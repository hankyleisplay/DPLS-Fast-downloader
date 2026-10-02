use crate::engine::client::{create_http_client, probe_url};
use crate::engine::segment::{DownloadMetadata, Segment, SegmentStatus};
use crate::engine::writer::FileWriterHandle;
use futures_util::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, COOKIE, RANGE, REFERER, USER_AGENT};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{broadcast, Mutex, RwLock};

pub const MIN_SPLIT_BYTES: u64 = 2 * 1024 * 1024; // 2 MB minimum to split

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskState {
    Queued,
    Probing,
    Downloading,
    Paused,
    Completed,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSnapshot {
    pub id: String,
    pub url: String,
    pub filename: String,
    pub output_path: String,
    pub total_size: Option<u64>,
    pub downloaded_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: Option<u64>,
    pub state: TaskState,
    pub progress_ratio: f64,
    pub connections: usize,
    pub segments: Vec<Segment>,
    pub supports_range: bool,
    pub scheduled_at: Option<u64>,
}

pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub output_dir: PathBuf,
    pub custom_filename: Option<String>,
    pub connections: usize,
    pub metadata: Arc<RwLock<DownloadMetadata>>,
    pub state: Arc<RwLock<TaskState>>,
    pub downloaded_bytes: Arc<AtomicU64>,
    pub speed_bps: Arc<AtomicU64>,
    pub is_paused: Arc<AtomicBool>,
    pub progress_tx: broadcast::Sender<TaskSnapshot>,
    pub scheduled_at: Arc<RwLock<Option<u64>>>,
    client: Client,
    final_output_path: Arc<RwLock<PathBuf>>,
}

impl DownloadTask {
    pub fn new(
        id: String,
        url: String,
        output_dir: PathBuf,
        custom_filename: Option<String>,
        connections: usize,
        user_agent: Option<String>,
        referer: Option<String>,
        cookies: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let mut custom_headers = HeaderMap::new();
        if let Some(ua) = user_agent {
            if !ua.trim().is_empty() {
                if let Ok(val) = HeaderValue::from_str(ua.trim()) {
                    custom_headers.insert(USER_AGENT, val);
                }
            }
        }
        if let Some(ref_url) = referer {
            if !ref_url.trim().is_empty() {
                if let Ok(val) = HeaderValue::from_str(ref_url.trim()) {
                    custom_headers.insert(REFERER, val);
                }
            }
        }
        if let Some(cookie_str) = cookies {
            if !cookie_str.trim().is_empty() {
                if let Ok(val) = HeaderValue::from_str(cookie_str.trim()) {
                    custom_headers.insert(COOKIE, val);
                }
            }
        }

        let headers_opt = if custom_headers.is_empty() {
            None
        } else {
            Some(custom_headers)
        };

        let client = create_http_client(headers_opt)?;
        let initial_name = custom_filename.clone().unwrap_or_else(|| "download.bin".into());
        let initial_path = output_dir.join(&initial_name);

        let metadata = Arc::new(RwLock::new(DownloadMetadata::new(
            url.clone(),
            initial_name,
            None,
            false,
            None,
            None,
        )));

        let (progress_tx, _) = broadcast::channel(128);

        Ok(Self {
            id,
            url,
            output_dir,
            custom_filename,
            connections: connections.max(1),
            metadata,
            state: Arc::new(RwLock::new(TaskState::Queued)),
            downloaded_bytes: Arc::new(AtomicU64::new(0)),
            speed_bps: Arc::new(AtomicU64::new(0)),
            is_paused: Arc::new(AtomicBool::new(false)),
            progress_tx,
            scheduled_at: Arc::new(RwLock::new(None)),
            client,
            final_output_path: Arc::new(RwLock::new(initial_path)),
        })
    }

    pub fn from_persisted(
        id: String,
        url: String,
        filename: String,
        output_path: PathBuf,
        total_size: Option<u64>,
        downloaded_bytes: u64,
        state: TaskState,
        connections: usize,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let client = create_http_client(None)?;
        let output_dir = output_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let (progress_tx, _) = broadcast::channel(128);
        let mut meta = DownloadMetadata::new(
            url.clone(),
            filename.clone(),
            total_size,
            true,
            None,
            None,
        );
        if state == TaskState::Completed {
            meta.completed = true;
        }
        let metadata = Arc::new(RwLock::new(meta));

        Ok(Self {
            id,
            url,
            output_dir,
            custom_filename: Some(filename),
            connections: connections.max(1),
            metadata,
            state: Arc::new(RwLock::new(state)),
            downloaded_bytes: Arc::new(AtomicU64::new(downloaded_bytes)),
            speed_bps: Arc::new(AtomicU64::new(0)),
            is_paused: Arc::new(AtomicBool::new(false)),
            progress_tx,
            scheduled_at: Arc::new(RwLock::new(None)),
            client,
            final_output_path: Arc::new(RwLock::new(output_path)),
        })
    }

    pub async fn get_meta_path(&self) -> PathBuf {
        let path = self.final_output_path.read().await;
        PathBuf::from(format!("{}.dpls.meta", path.display()))
    }

    pub async fn snapshot(&self) -> TaskSnapshot {
        let meta = self.metadata.read().await;
        let state = self.state.read().await.clone();
        let path = self.final_output_path.read().await.to_string_lossy().to_string();
        let downloaded = self.downloaded_bytes.load(Ordering::Relaxed);
        let speed = self.speed_bps.load(Ordering::Relaxed);

        let eta = if speed > 0 && meta.total_size.is_some() {
            let total = meta.total_size.unwrap();
            if total > downloaded {
                Some((total - downloaded) / speed)
            } else {
                Some(0)
            }
        } else {
            None
        };

        let scheduled = *self.scheduled_at.read().await;

        TaskSnapshot {
            id: self.id.clone(),
            url: self.url.clone(),
            filename: meta.filename.clone(),
            output_path: path,
            total_size: meta.total_size,
            downloaded_bytes: downloaded,
            speed_bps: speed,
            eta_seconds: eta,
            state,
            progress_ratio: meta.progress_ratio(),
            connections: self.connections,
            segments: meta.segments.clone(),
            supports_range: meta.supports_range,
            scheduled_at: scheduled,
        }
    }

    pub async fn pause(&self) {
        self.is_paused.store(true, Ordering::SeqCst);
        {
            let mut state = self.state.write().await;
            if *state == TaskState::Downloading {
                *state = TaskState::Paused;
            }
        }
        self.save_metadata().await;
        let snap = self.snapshot().await;
        let _ = self.progress_tx.send(snap);
    }

    pub async fn resume(&self) {
        self.is_paused.store(false, Ordering::SeqCst);
        let mut meta = self.metadata.write().await;
        for seg in meta.segments.iter_mut() {
            if !seg.is_completed() {
                seg.status = SegmentStatus::Pending;
            }
        }
    }

    async fn save_metadata(&self) {
        let meta = self.metadata.read().await;
        if let Ok(json) = serde_json::to_string_pretty(&*meta) {
            let meta_p = self.get_meta_path().await;
            let _ = tokio::fs::write(&meta_p, json).await;
        }
    }

    pub async fn start(self: Arc<Self>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            let mut state = self.state.write().await;
            *state = TaskState::Probing;
        }
        self.is_paused.store(false, Ordering::SeqCst);
        let snap = self.snapshot().await;
        let _ = self.progress_tx.send(snap);

        // 1. Check if metadata file exists for resume
        let meta_p = self.get_meta_path().await;
        let mut resumed = false;
        if meta_p.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&meta_p).await {
                if let Ok(loaded_meta) = serde_json::from_str::<DownloadMetadata>(&content) {
                    if loaded_meta.url == self.url {
                        let total_dl = loaded_meta.total_downloaded();
                        *self.metadata.write().await = loaded_meta;
                        self.downloaded_bytes.store(total_dl, Ordering::SeqCst);
                        resumed = true;
                    }
                }
            }
        }

        // 2. If not resumed, probe URL
        if !resumed {
            let probe = match probe_url(&self.client, &self.url).await {
                Ok(p) => p,
                Err(e) => {
                    let err_msg = format!("Probe failed: {}", e);
                    {
                        let mut state = self.state.write().await;
                        *state = TaskState::Error(err_msg.clone());
                    }
                    let snap = self.snapshot().await;
                    let _ = self.progress_tx.send(snap);
                    return Err(e);
                }
            };

            let filename = self
                .custom_filename
                .clone()
                .unwrap_or(probe.filename);
            let target_path = self.output_dir.join(&filename);
            *self.final_output_path.write().await = target_path.clone();

            let real_meta_p = self.get_meta_path().await;
            if real_meta_p.exists() {
                if let Ok(content) = tokio::fs::read_to_string(&real_meta_p).await {
                    if let Ok(loaded_meta) = serde_json::from_str::<DownloadMetadata>(&content) {
                        if loaded_meta.url == self.url {
                            let total_dl = loaded_meta.total_downloaded();
                            *self.metadata.write().await = loaded_meta;
                            self.downloaded_bytes.store(total_dl, Ordering::SeqCst);
                            resumed = true;
                        }
                    }
                }
            }

            if !resumed {
                let mut meta = self.metadata.write().await;
                meta.filename = filename;
                meta.total_size = probe.total_size;
                meta.supports_range = probe.supports_range;
                meta.etag = probe.etag;
                meta.last_modified = probe.last_modified;
                meta.initialize_segments(self.connections);
            }
        }

        // 3. Initialize file writer
        let final_path = self.final_output_path.read().await.clone();
        let total_size = self.metadata.read().await.total_size;
        let writer = FileWriterHandle::create(&final_path, total_size).await?;

        {
            let mut state = self.state.write().await;
            *state = TaskState::Downloading;
        }

        // 4. Start background speed calculator and progress broadcaster
        let this = self.clone();
        let speed_task = tokio::spawn(async move {
            let mut last_bytes = this.downloaded_bytes.load(Ordering::Relaxed);
            let mut last_instant = Instant::now();

            loop {
                tokio::time::sleep(Duration::from_millis(500)).await;
                let current_state = this.state.read().await.clone();
                if current_state != TaskState::Downloading {
                    if current_state != TaskState::Paused {
                        this.speed_bps.store(0, Ordering::Relaxed);
                    }
                    if current_state == TaskState::Completed || matches!(current_state, TaskState::Error(_)) {
                        break;
                    }
                }

                let current_bytes = this.downloaded_bytes.load(Ordering::Relaxed);
                let elapsed = last_instant.elapsed().as_secs_f64();
                if elapsed > 0.1 {
                    let diff = current_bytes.saturating_sub(last_bytes);
                    let speed = (diff as f64 / elapsed) as u64;
                    this.speed_bps.store(speed, Ordering::Relaxed);
                    last_bytes = current_bytes;
                    last_instant = Instant::now();
                }

                this.save_metadata().await;
                let _ = this.progress_tx.send(this.snapshot().await);
            }
        });

        // Reset any uncompleted segments to Pending so workers can claim them
        {
            let mut meta = self.metadata.write().await;
            for seg in meta.segments.iter_mut() {
                if !seg.is_completed() {
                    seg.status = SegmentStatus::Pending;
                }
            }
        }

        // 5. Worker pool execution
        let active_workers = Arc::new(Mutex::new(0usize));
        let num_workers = self.connections;

        let mut join_handles = Vec::new();
        for worker_id in 0..num_workers {
            let task_ref = self.clone();
            let writer_ref = writer.clone();
            let active_ref = active_workers.clone();

            let handle = tokio::spawn(async move {
                loop {
                    if task_ref.is_paused.load(Ordering::Relaxed) {
                        break;
                    }

                    // Pick next pending or incomplete segment
                    let mut seg_to_download: Option<Segment> = None;
                    {
                        let mut meta = task_ref.metadata.write().await;
                        // First find any pending segment
                        if let Some(s) = meta.segments.iter_mut().find(|s| s.status == SegmentStatus::Pending && !s.is_completed()) {
                            s.status = SegmentStatus::Downloading;
                            seg_to_download = Some(s.clone());
                        } else {
                            // If no pending segment, try dynamic splitting!
                            if let Some(new_seg) = meta.try_split_largest_segment(MIN_SPLIT_BYTES) {
                                if let Some(target) = meta.segments.iter_mut().find(|s| s.id == new_seg.id) {
                                    target.status = SegmentStatus::Downloading;
                                    seg_to_download = Some(target.clone());
                                }
                            }
                        }
                    }

                    let segment = match seg_to_download {
                        Some(s) => s,
                        None => {
                            // No work available
                            break;
                        }
                    };

                    {
                        let mut act = active_ref.lock().await;
                        *act += 1;
                    }

                    // Download this segment
                    let res = download_segment_stream(
                        &task_ref,
                        segment.id,
                        &writer_ref,
                    ).await;

                    {
                        let mut act = active_ref.lock().await;
                        *act = act.saturating_sub(1);
                    }

                    if let Err(e) = res {
                        eprintln!("Worker {} failed on segment {}: {}", worker_id, segment.id, e);
                        let mut meta = task_ref.metadata.write().await;
                        if let Some(s) = meta.segments.iter_mut().find(|s| s.id == segment.id) {
                            if !s.is_completed() {
                                s.status = SegmentStatus::Pending; // Re-queue
                            }
                        }
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                }
            });
            join_handles.push(handle);
        }

        // Wait for all workers
        for handle in join_handles {
            let _ = handle.await;
        }

        // Check completion
        let is_completed = {
            let meta = self.metadata.read().await;
            if let Some(total) = meta.total_size {
                meta.total_downloaded() >= total
            } else {
                meta.segments.iter().all(|s| s.is_completed())
            }
        };

        if is_completed {
            writer.close().await?;
            {
                let mut state = self.state.write().await;
                *state = TaskState::Completed;
            }
            {
                let mut meta = self.metadata.write().await;
                meta.completed = true;
            }
            // Clean up metadata file on successful finish
            let meta_p = self.get_meta_path().await;
            let _ = tokio::fs::remove_file(&meta_p).await;
        } else if self.is_paused.load(Ordering::Relaxed) {
            let _ = writer.close().await;
            {
                let mut state = self.state.write().await;
                *state = TaskState::Paused;
            }
            self.save_metadata().await;
        }

        speed_task.abort();
        let snap = self.snapshot().await;
        let _ = self.progress_tx.send(snap);
        Ok(())
    }
}

async fn download_segment_stream(
    task: &DownloadTask,
    seg_id: usize,
    writer: &FileWriterHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (url, start_offset, end_offset, supports_range) = {
        let meta = task.metadata.read().await;
        let s = meta.segments.iter().find(|s| s.id == seg_id).ok_or("Segment not found")?;
        (meta.url.clone(), s.current_offset(), s.end, meta.supports_range)
    };

    if start_offset > end_offset {
        let mut meta = task.metadata.write().await;
        if let Some(s) = meta.segments.iter_mut().find(|s| s.id == seg_id) {
            s.status = SegmentStatus::Completed;
        }
        return Ok(());
    }

    let mut req = task.client.get(&url);
    if supports_range && end_offset < u64::MAX {
        req = req.header(RANGE, format!("bytes={}-{}", start_offset, end_offset));
    }

    let resp = req.send().await?;
    if !resp.status().is_success() && resp.status() != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(format!("Bad HTTP status: {}", resp.status()).into());
    }

    let mut stream = resp.bytes_stream();
    let mut current_pos = start_offset;

    while let Some(chunk_res) = stream.next().await {
        if task.is_paused.load(Ordering::Relaxed) {
            break;
        }

        let chunk = chunk_res?;
        let chunk_len = chunk.len() as u64;

        writer.write(current_pos, chunk.to_vec()).await?;
        current_pos += chunk_len;

        task.downloaded_bytes.fetch_add(chunk_len, Ordering::Relaxed);

        // Update segment downloaded bytes
        {
            let mut meta = task.metadata.write().await;
            if let Some(s) = meta.segments.iter_mut().find(|s| s.id == seg_id) {
                s.downloaded += chunk_len;
                // Check if segment was dynamically resized
                if s.current_offset() > s.end && s.end < u64::MAX {
                    s.status = SegmentStatus::Completed;
                    break;
                }
            }
        }
    }

    let mut meta = task.metadata.write().await;
    if let Some(s) = meta.segments.iter_mut().find(|s| s.id == seg_id) {
        if s.is_completed() {
            s.status = SegmentStatus::Completed;
        } else {
            s.status = SegmentStatus::Pending;
        }
    }

    Ok(())
}
