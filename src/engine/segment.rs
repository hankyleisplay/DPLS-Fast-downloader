use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: usize,
    pub start: u64,
    pub end: u64, // inclusive
    pub downloaded: u64,
    pub status: SegmentStatus,
}

impl Segment {
    pub fn new(id: usize, start: u64, end: u64) -> Self {
        Self {
            id,
            start,
            end,
            downloaded: 0,
            status: SegmentStatus::Pending,
        }
    }

    pub fn total_bytes(&self) -> u64 {
        if self.end >= self.start {
            self.end - self.start + 1
        } else {
            0
        }
    }

    pub fn remaining_bytes(&self) -> u64 {
        let total = self.total_bytes();
        if self.downloaded < total {
            total - self.downloaded
        } else {
            0
        }
    }

    pub fn current_offset(&self) -> u64 {
        self.start + self.downloaded
    }

    pub fn is_completed(&self) -> bool {
        self.downloaded >= self.total_bytes()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadMetadata {
    pub url: String,
    pub filename: String,
    pub total_size: Option<u64>,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub segments: Vec<Segment>,
    pub completed: bool,
}

impl DownloadMetadata {
    pub fn new(
        url: String,
        filename: String,
        total_size: Option<u64>,
        supports_range: bool,
        etag: Option<String>,
        last_modified: Option<String>,
    ) -> Self {
        Self {
            url,
            filename,
            total_size,
            supports_range,
            etag,
            last_modified,
            segments: Vec::new(),
            completed: false,
        }
    }

    /// Split a file of `total_size` into `num_segments` initial segments
    pub fn initialize_segments(&mut self, num_segments: usize) {
        self.segments.clear();

        let total = match self.total_size {
            Some(s) if s > 0 && self.supports_range => s,
            _ => {
                // If unknown size or range not supported, use a single stream segment
                self.segments.push(Segment::new(0, 0, u64::MAX));
                return;
            }
        };

        let count = num_segments.max(1).min(total as usize);
        let chunk_size = total / count as u64;
        let mut start = 0;

        for i in 0..count {
            let end = if i == count - 1 {
                total - 1
            } else {
                start + chunk_size - 1
            };
            self.segments.push(Segment::new(i, start, end));
            start = end + 1;
        }
    }

    pub fn total_downloaded(&self) -> u64 {
        self.segments.iter().map(|s| s.downloaded).sum()
    }

    pub fn progress_ratio(&self) -> f64 {
        match self.total_size {
            Some(total) if total > 0 => {
                let downloaded = self.total_downloaded();
                (downloaded as f64 / total as f64).min(1.0)
            }
            _ => 0.0,
        }
    }

    /// Dynamic splitting: find the segment with the largest remaining bytes (> min_split_bytes)
    /// and split its remaining portion into two.
    pub fn try_split_largest_segment(&mut self, min_split_bytes: u64) -> Option<Segment> {
        if !self.supports_range {
            return None;
        }

        // Find candidate with max remaining bytes
        let candidate_idx = self
            .segments
            .iter()
            .enumerate()
            .filter(|(_, s)| s.status == SegmentStatus::Downloading && s.remaining_bytes() >= min_split_bytes)
            .max_by_key(|(_, s)| s.remaining_bytes())
            .map(|(idx, _)| idx)?;

        let target = &mut self.segments[candidate_idx];
        let remaining = target.remaining_bytes();
        let split_offset = remaining / 2;

        if split_offset < (min_split_bytes / 2).max(1024 * 512) {
            return None;
        }

        let old_end = target.end;
        let new_end = target.current_offset() + split_offset - 1;
        let new_start = new_end + 1;

        // Truncate target segment end
        target.end = new_end;

        // Create new segment
        let new_id = self.segments.len();
        let new_segment = Segment::new(new_id, new_start, old_end);
        self.segments.push(new_segment.clone());

        Some(new_segment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_segments() {
        let mut meta = DownloadMetadata::new(
            "http://example.com/test.zip".into(),
            "test.zip".into(),
            Some(1000),
            true,
            None,
            None,
        );
        meta.initialize_segments(4);
        assert_eq!(meta.segments.len(), 4);
        assert_eq!(meta.segments[0].start, 0);
        assert_eq!(meta.segments[0].end, 249);
        assert_eq!(meta.segments[3].start, 750);
        assert_eq!(meta.segments[3].end, 999);
    }

    #[test]
    fn test_dynamic_split() {
        let mut meta = DownloadMetadata::new(
            "http://example.com/test.zip".into(),
            "test.zip".into(),
            Some(100 * 1024 * 1024), // 100MB
            true,
            None,
            None,
        );
        meta.initialize_segments(2);
        meta.segments[0].status = SegmentStatus::Downloading;
        meta.segments[0].downloaded = 10 * 1024 * 1024; // 10MB down, 40MB remaining

        let new_seg = meta.try_split_largest_segment(2 * 1024 * 1024);
        assert!(new_seg.is_some());
        let new_seg = new_seg.unwrap();
        assert_eq!(meta.segments.len(), 3);
        assert_eq!(new_seg.id, 2);
        assert!(new_seg.start > meta.segments[0].current_offset());
        assert_eq!(new_seg.end, 50 * 1024 * 1024 - 1);
    }
}
