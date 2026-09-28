use percent_encoding::percent_decode_str;
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_RANGE, ETAG,
    LAST_MODIFIED, RANGE, USER_AGENT,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::Path;

const DEFAULT_USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 DPLS-Fast/1.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeInfo {
    #[allow(dead_code)]
    pub url: String,
    pub filename: String,
    pub total_size: Option<u64>,
    pub supports_range: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

pub fn create_http_client(custom_headers: Option<HeaderMap>) -> Result<Client, reqwest::Error> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(DEFAULT_USER_AGENT));

    if let Some(custom) = custom_headers {
        for (k, v) in custom {
            if let Some(key) = k {
                headers.insert(key, v);
            }
        }
    }

    Client::builder()
        .default_headers(headers)
        .tcp_keepalive(std::time::Duration::from_secs(30))
        .tcp_nodelay(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
}

pub async fn probe_url(client: &Client, url: &str) -> Result<ProbeInfo, Box<dyn std::error::Error + Send + Sync>> {
    // 1. Try HEAD request first
    let head_res = client.head(url).send().await;

    let mut filename = None;
    let mut total_size = None;
    let mut supports_range = false;
    let mut etag = None;
    let mut last_modified = None;

    let mut need_get = false;

    match head_res {
        Ok(res) if res.status().is_success() => {
            let headers = res.headers();

            if let Some(disposition) = headers.get(CONTENT_DISPOSITION) {
                if let Ok(disp_str) = disposition.to_str() {
                    filename = extract_filename_from_content_disposition(disp_str);
                }
            }

            if let Some(len) = headers.get(CONTENT_LENGTH) {
                if let Ok(len_str) = len.to_str() {
                    total_size = len_str.parse::<u64>().ok();
                }
            }

            if let Some(ar) = headers.get(ACCEPT_RANGES) {
                if let Ok(ar_str) = ar.to_str() {
                    if ar_str.to_lowercase().contains("bytes") {
                        supports_range = true;
                    }
                }
            }

            if let Some(et) = headers.get(ETAG) {
                etag = et.to_str().ok().map(|s| s.to_string());
            }

            if let Some(lm) = headers.get(LAST_MODIFIED) {
                last_modified = lm.to_str().ok().map(|s| s.to_string());
            }

            // If range support wasn't declared in HEAD, test with Range: bytes=0-0
            if !supports_range && total_size.is_some() {
                need_get = true;
            }
        }
        _ => {
            need_get = true;
        }
    }

    // 2. If HEAD failed or did not confirm range support, issue a lightweight Range GET: bytes=0-0
    if need_get {
        let get_res = client
            .get(url)
            .header(RANGE, "bytes=0-0")
            .send()
            .await?;

        let status = get_res.status();
        let headers = get_res.headers();

        if status == reqwest::StatusCode::PARTIAL_CONTENT {
            supports_range = true;

            if let Some(cr) = headers.get(CONTENT_RANGE) {
                if let Ok(cr_str) = cr.to_str() {
                    // Content-Range: bytes 0-0/146515
                    if let Some(total_str) = cr_str.split('/').last() {
                        if let Ok(parsed_len) = total_str.trim().parse::<u64>() {
                            total_size = Some(parsed_len);
                        }
                    }
                }
            }
        } else if status.is_success() {
            supports_range = false;
            if let Some(len) = headers.get(CONTENT_LENGTH) {
                if let Ok(len_str) = len.to_str() {
                    total_size = len_str.parse::<u64>().ok();
                }
            }
        }

        if filename.is_none() {
            if let Some(disposition) = headers.get(CONTENT_DISPOSITION) {
                if let Ok(disp_str) = disposition.to_str() {
                    filename = extract_filename_from_content_disposition(disp_str);
                }
            }
        }

        if etag.is_none() {
            if let Some(et) = headers.get(ETAG) {
                etag = et.to_str().ok().map(|s| s.to_string());
            }
        }

        if last_modified.is_none() {
            if let Some(lm) = headers.get(LAST_MODIFIED) {
                last_modified = lm.to_str().ok().map(|s| s.to_string());
            }
        }
    }

    // Fallback filename from URL
    let resolved_filename = filename.unwrap_or_else(|| extract_filename_from_url(url));

    Ok(ProbeInfo {
        url: url.to_string(),
        filename: resolved_filename,
        total_size,
        supports_range,
        etag,
        last_modified,
    })
}

fn extract_filename_from_content_disposition(disp: &str) -> Option<String> {
    for part in disp.split(';') {
        let part = part.trim();
        if let Some(name) = part.strip_prefix("filename*=") {
            // e.g. UTF-8''encoded_name.ext
            let cleaned = name.trim_matches('"');
            if let Some(utf8_idx) = cleaned.to_lowercase().find("utf-8''") {
                let encoded = &cleaned[utf8_idx + 7..];
                if let Ok(decoded) = percent_decode_str(encoded).decode_utf8() {
                    return Some(sanitize_filename(&decoded));
                }
            }
        } else if let Some(name) = part.strip_prefix("filename=") {
            let cleaned = name.trim().trim_matches('"');
            if !cleaned.is_empty() {
                return Some(sanitize_filename(cleaned));
            }
        }
    }
    None
}

fn extract_filename_from_url(url: &str) -> String {
    if let Ok(parsed) = reqwest::Url::parse(url) {
        if let Some(segments) = parsed.path_segments() {
            if let Some(last) = segments.filter(|s| !s.is_empty()).last() {
                let decoded = percent_decode_str(last).decode_utf8_lossy().to_string();
                let sanitized = sanitize_filename(&decoded);
                if !sanitized.is_empty() {
                    return sanitized;
                }
            }
        }
    }
    "download.bin".to_string()
}

fn sanitize_filename(filename: &str) -> String {
    let name = Path::new(filename)
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or("download.bin");

    let clean: String = name
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect();

    if clean.trim().is_empty() {
        "download.bin".to_string()
    } else {
        clean
    }
}
