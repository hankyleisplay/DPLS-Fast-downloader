use crate::engine::downloader::{DownloadTask, TaskSnapshot, TaskState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddTaskRequest {
    pub url: String,
    pub filename: Option<String>,
    pub output_dir: Option<String>,
    pub connections: Option<usize>,
    pub start_immediately: Option<bool>,
    pub speed_limit_bps: Option<u64>,
    pub collision_policy: Option<String>,
    pub expected_hash: Option<String>,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub cookies: Option<String>,
}

fn resolve_collision(dir: &std::path::Path, filename: &str, policy: Option<&str>) -> String {
    let policy = policy.unwrap_or("auto_rename");
    if policy == "overwrite" {
        return filename.to_string();
    }

    let target_path = dir.join(filename);
    if !target_path.exists() {
        return filename.to_string();
    }

    let path = std::path::Path::new(filename);
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or(filename);
    let ext = path.extension().and_then(|e| e.to_str());

    let mut counter = 1;
    loop {
        let new_name = match ext {
            Some(e) => format!("{} ({}).{}", stem, counter, e),
            None => format!("{} ({})", stem, counter),
        };
        if !dir.join(&new_name).exists() {
            return new_name;
        }
        counter += 1;
        if counter > 1000 {
            return new_name;
        }
    }
}

pub struct TaskManager {
    tasks: Arc<RwLock<HashMap<String, Arc<DownloadTask>>>>,
    pub default_download_dir: PathBuf,
    pub global_events_tx: broadcast::Sender<TaskSnapshot>,
}

impl TaskManager {
    pub fn new(default_download_dir: PathBuf) -> Self {
        let (global_events_tx, _) = broadcast::channel(256);
        Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            default_download_dir,
            global_events_tx,
        }
    }

    pub async fn add_task(
        &self,
        req: AddTaskRequest,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let id = Uuid::new_v4().to_string();
        let output_dir = req
            .output_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| self.default_download_dir.clone());

        let connections = req.connections.unwrap_or(16);
        let start_immediately = req.start_immediately.unwrap_or(true);

        let final_filename = req
            .filename
            .map(|f| resolve_collision(&output_dir, &f, req.collision_policy.as_deref()));

        let task = Arc::new(DownloadTask::new(
            id.clone(),
            req.url,
            output_dir,
            final_filename,
            connections,
            req.user_agent,
            req.referer,
            req.cookies,
        )?);

        // Forward task progress events to global channel
        let mut rx = task.progress_tx.subscribe();
        let global_tx = self.global_events_tx.clone();
        tokio::spawn(async move {
            while let Ok(snapshot) = rx.recv().await {
                let _ = global_tx.send(snapshot);
            }
        });

        // Register task
        self.tasks.write().await.insert(id.clone(), task.clone());

        if !start_immediately {
            task.is_paused.store(true, Ordering::SeqCst);
            *task.state.write().await = TaskState::Paused;
            let snap = task.snapshot().await;
            let _ = self.global_events_tx.send(snap);
        } else {
            // Spawn task runner
            tokio::spawn(async move {
                if let Err(e) = task.start().await {
                    eprintln!("Task error: {}", e);
                }
            });
        }

        Ok(id)
    }

    pub async fn get_all_snapshots(&self) -> Vec<TaskSnapshot> {
        let tasks = self.tasks.read().await;
        let mut snapshots = Vec::with_capacity(tasks.len());
        for task in tasks.values() {
            snapshots.push(task.snapshot().await);
        }
        snapshots
    }

    pub async fn get_task_snapshot(&self, id: &str) -> Option<TaskSnapshot> {
        let task = {
            let tasks = self.tasks.read().await;
            tasks.get(id).cloned()
        };
        if let Some(task) = task {
            Some(task.snapshot().await)
        } else {
            None
        }
    }

    pub async fn pause_task(&self, id: &str) -> bool {
        let task = {
            let tasks = self.tasks.read().await;
            tasks.get(id).cloned()
        };
        if let Some(task) = task {
            task.pause().await;
            true
        } else {
            false
        }
    }

    pub async fn resume_task(&self, id: &str) -> bool {
        let task = {
            let tasks = self.tasks.read().await;
            tasks.get(id).cloned()
        };
        if let Some(task) = task {
            tokio::spawn(async move {
                task.resume().await;
                let _ = task.start().await;
            });
            true
        } else {
            false
        }
    }

    pub async fn delete_task(&self, id: &str, delete_file: bool) -> bool {
        let task = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(id)
        };
        if let Some(task) = task {
            task.pause().await;
            if delete_file {
                let snap = task.snapshot().await;
                let _ = tokio::fs::remove_file(&snap.output_path).await;
                let meta_file = format!("{}.dpls.meta", snap.output_path);
                let _ = tokio::fs::remove_file(&meta_file).await;
            }
            true
        } else {
            false
        }
    }
}
