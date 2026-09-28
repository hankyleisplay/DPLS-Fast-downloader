use crate::engine::downloader::{DownloadTask, TaskSnapshot, TaskState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
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
    pub scheduled_start_time: Option<u64>,
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
    pub max_concurrent_tasks: Arc<AtomicUsize>,
    queue_trigger_tx: mpsc::Sender<()>,
    queue_trigger_rx: Arc<tokio::sync::Mutex<Option<mpsc::Receiver<()>>>>,
    scheduled_tasks: Arc<RwLock<HashMap<String, u64>>>,
    queue_worker_started: Arc<std::sync::atomic::AtomicBool>,
}

impl TaskManager {
    pub fn new(default_download_dir: PathBuf) -> Self {
        let (global_events_tx, _) = broadcast::channel(256);
        let tasks = Arc::new(RwLock::new(HashMap::new()));
        let max_concurrent_tasks = Arc::new(AtomicUsize::new(3));
        let scheduled_tasks = Arc::new(RwLock::new(HashMap::new()));
        let (queue_trigger_tx, queue_trigger_rx) = mpsc::channel::<()>(128);

        Self {
            tasks,
            default_download_dir,
            global_events_tx,
            max_concurrent_tasks,
            queue_trigger_tx,
            queue_trigger_rx: Arc::new(tokio::sync::Mutex::new(Some(queue_trigger_rx))),
            scheduled_tasks,
            queue_worker_started: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn ensure_queue_worker_started(&self) {
        if !self.queue_worker_started.swap(true, Ordering::SeqCst) {
            let tasks_ref = self.tasks.clone();
            let max_ref = self.max_concurrent_tasks.clone();
            let sched_ref = self.scheduled_tasks.clone();
            let global_tx = self.global_events_tx.clone();
            let trigger_tx = self.queue_trigger_tx.clone();
            let rx_mutex = self.queue_trigger_rx.clone();

            tokio::spawn(async move {
                let mut rx_opt = rx_mutex.lock().await;
                if let Some(mut rx) = rx_opt.take() {
                    drop(rx_opt);
                    let mut interval = tokio::time::interval(std::time::Duration::from_millis(1000));
                    loop {
                        tokio::select! {
                            _ = rx.recv() => {
                                Self::process_queue(&tasks_ref, &max_ref, &sched_ref, &global_tx, &trigger_tx).await;
                            }
                            _ = interval.tick() => {
                                let now = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                let mut ready = false;
                                {
                                    let mut sched = sched_ref.write().await;
                                    let due: Vec<String> = sched.iter()
                                        .filter(|(_, &ts)| now >= ts)
                                        .map(|(id, _)| id.clone())
                                        .collect();
                                    for id in due {
                                        sched.remove(&id);
                                        ready = true;
                                    }
                                }

                                if ready {
                                    Self::process_queue(&tasks_ref, &max_ref, &sched_ref, &global_tx, &trigger_tx).await;
                                }
                            }
                        }
                    }
                }
            });
        }
    }

    async fn process_queue(
        tasks: &Arc<RwLock<HashMap<String, Arc<DownloadTask>>>>,
        max_concurrent: &Arc<AtomicUsize>,
        scheduled_tasks: &Arc<RwLock<HashMap<String, u64>>>,
        global_tx: &broadcast::Sender<TaskSnapshot>,
        trigger_tx: &mpsc::Sender<()>,
    ) {
        let max = max_concurrent.load(Ordering::Relaxed).max(1);
        let task_list: Vec<Arc<DownloadTask>> = {
            let map = tasks.read().await;
            map.values().cloned().collect()
        };

        let mut active_count = 0;
        let mut queued_candidates = Vec::new();

        for task in task_list {
            let state = task.state.read().await.clone();
            if state == TaskState::Downloading || state == TaskState::Probing {
                active_count += 1;
            } else if state == TaskState::Queued {
                queued_candidates.push(task);
            }
        }

        let sched_map = scheduled_tasks.read().await;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let available_slots = max.saturating_sub(active_count);
        let mut launched = 0;

        for task in queued_candidates {
            if launched >= available_slots {
                break;
            }

            // If task is scheduled for future, don't start yet
            if let Some(&due) = sched_map.get(&task.id) {
                if now < due {
                    continue;
                }
            }

            // Set state to Downloading and start
            {
                let mut state = task.state.write().await;
                *state = TaskState::Downloading;
            }
            task.is_paused.store(false, Ordering::SeqCst);

            let snap = task.snapshot().await;
            let _ = global_tx.send(snap);

            let t = task.clone();
            let trig = trigger_tx.clone();
            tokio::spawn(async move {
                if let Err(e) = t.start().await {
                    eprintln!("Task error: {}", e);
                }
                let _ = trig.send(()).await;
            });

            launched += 1;
        }
    }

    pub async fn add_task(
        &self,
        req: AddTaskRequest,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        self.ensure_queue_worker_started();
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

        if let Some(ts) = req.scheduled_start_time {
            *task.scheduled_at.write().await = Some(ts);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            if ts > now {
                self.scheduled_tasks.write().await.insert(id.clone(), ts);
            }
        }

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
            let snap = task.snapshot().await;
            let _ = self.global_events_tx.send(snap);
            let _ = self.queue_trigger_tx.send(()).await;
        }

        Ok(id)
    }

    pub async fn batch_add(
        &self,
        reqs: Vec<AddTaskRequest>,
    ) -> Vec<Result<String, String>> {
        let mut results = Vec::with_capacity(reqs.len());
        for req in reqs {
            match self.add_task(req).await {
                Ok(id) => results.push(Ok(id)),
                Err(e) => results.push(Err(e.to_string())),
            }
        }
        results
    }

    pub fn get_max_concurrent_tasks(&self) -> usize {
        self.max_concurrent_tasks.load(Ordering::Relaxed)
    }

    pub async fn set_max_concurrent_tasks(&self, val: usize) {
        self.max_concurrent_tasks.store(val.max(1), Ordering::Relaxed);
        let _ = self.queue_trigger_tx.send(()).await;
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
            let _ = self.queue_trigger_tx.send(()).await;
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
            task.is_paused.store(false, Ordering::SeqCst);
            *task.state.write().await = TaskState::Queued;
            self.scheduled_tasks.write().await.remove(id);
            let snap = task.snapshot().await;
            let _ = self.global_events_tx.send(snap);
            let _ = self.queue_trigger_tx.send(()).await;
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
        self.scheduled_tasks.write().await.remove(id);
        if let Some(task) = task {
            task.pause().await;
            if delete_file {
                let snap = task.snapshot().await;
                let _ = tokio::fs::remove_file(&snap.output_path).await;
                let meta_file = format!("{}.dpls.meta", snap.output_path);
                let _ = tokio::fs::remove_file(&meta_file).await;
            }
            let _ = self.queue_trigger_tx.send(()).await;
            true
        } else {
            false
        }
    }
}
