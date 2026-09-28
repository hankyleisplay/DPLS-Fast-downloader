use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use tokio::sync::{mpsc, oneshot};

#[allow(dead_code)]
pub enum WriteMessage {
    Write {
        offset: u64,
        data: Vec<u8>,
        confirm: Option<oneshot::Sender<()>>,
    },
    Flush {
        reply: oneshot::Sender<()>,
    },
    Close {
        reply: oneshot::Sender<()>,
    },
}

#[derive(Clone)]
pub struct FileWriterHandle {
    sender: mpsc::Sender<WriteMessage>,
    #[allow(dead_code)]
    pub file_path: PathBuf,
}

impl FileWriterHandle {
    pub async fn create(
        path: &Path,
        preallocate_size: Option<u64>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Ensure parent directories exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)
            .await?;

        if let Some(size) = preallocate_size {
            if size > 0 {
                let current_len = file.metadata().await?.len();
                if current_len < size {
                    file.set_len(size).await?;
                }
            }
        }

        let (sender, mut receiver) = mpsc::channel::<WriteMessage>(1024);
        let path_clone = path.to_path_buf();

        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                match msg {
                    WriteMessage::Write {
                        offset,
                        data,
                        confirm,
                    } => {
                        if let Err(e) = write_at_offset(&mut file, offset, &data).await {
                            eprintln!("Error writing at offset {}: {}", offset, e);
                        }
                        if let Some(c) = confirm {
                            let _ = c.send(());
                        }
                    }
                    WriteMessage::Flush { reply } => {
                        let _ = file.flush().await;
                        let _ = reply.send(());
                    }
                    WriteMessage::Close { reply } => {
                        let _ = file.flush().await;
                        let _ = file.sync_all().await;
                        let _ = reply.send(());
                        break;
                    }
                }
            }
        });

        Ok(Self {
            sender,
            file_path: path_clone,
        })
    }

    pub async fn write(&self, offset: u64, data: Vec<u8>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.sender
            .send(WriteMessage::Write {
                offset,
                data,
                confirm: None,
            })
            .await
            .map_err(|e| format!("Writer channel closed: {}", e).into())
    }

    #[allow(dead_code)]
    pub async fn flush(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(WriteMessage::Flush { reply: tx })
            .await
            .map_err(|e| format!("Writer channel closed: {}", e))?;
        rx.await.map_err(|e| format!("Flush response dropped: {}", e).into())
    }

    pub async fn close(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        self.sender
            .send(WriteMessage::Close { reply: tx })
            .await
            .map_err(|e| format!("Writer channel closed: {}", e))?;
        rx.await.map_err(|e| format!("Close response dropped: {}", e).into())
    }
}

async fn write_at_offset(
    file: &mut File,
    offset: u64,
    data: &[u8],
) -> Result<(), std::io::Error> {
    file.seek(SeekFrom::Start(offset)).await?;
    file.write_all(data).await?;
    Ok(())
}
