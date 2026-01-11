use super::free_space;
use crate::utils::prettify_bytes;
use futures_util::StreamExt;
use reqwest::StatusCode;
use reqwest::header::{RANGE, USER_AGENT};
use reqwest_middleware::ClientWithMiddleware;
use reqwest_retry::RetryTransientMiddleware;
use reqwest_retry::policies::ExponentialBackoff;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::atomic::{AtomicU64, Ordering as AtomicOrdering};
use std::time::Duration;
use std::time::Instant;
use thiserror::Error;
use tokio::io::AsyncSeekExt;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;

/// Thread-safe speed tracker using Exponential Moving Average for smooth readings.
/// This aggregates download speeds across multiple parallel downloads and provides
/// a smoothed speed value that doesn't fluctuate wildly.
pub struct SpeedTracker {
    total_bytes: AtomicU64,
    last_bytes: AtomicU64,
    last_update: StdMutex<Instant>,
    ema_speed: AtomicU64, // Stored as integer (bytes/s) for atomicity
}

impl SpeedTracker {
    pub fn new() -> Self {
        Self {
            total_bytes: AtomicU64::new(0),
            last_bytes: AtomicU64::new(0),
            last_update: StdMutex::new(Instant::now()),
            ema_speed: AtomicU64::new(0),
        }
    }

    /// Add bytes downloaded by a worker. Returns the current smoothed speed.
    pub fn add_bytes(&self, bytes: u64) -> u64 {
        self.total_bytes.fetch_add(bytes, AtomicOrdering::SeqCst);
        self.ema_speed.load(AtomicOrdering::SeqCst)
    }

    /// Update the EMA speed calculation. Call this periodically (e.g., every 200ms).
    /// Returns the smoothed speed in bytes/second.
    pub fn update(&self) -> u64 {
        const EMA_ALPHA: f64 = 0.5; // More responsive at slower update frequency
        const MIN_UPDATE_MS: u128 = 150; // Minimum time between updates

        let now = Instant::now();
        let mut last_update = self.last_update.lock().unwrap();
        let elapsed = now.duration_since(*last_update);

        if elapsed.as_millis() < MIN_UPDATE_MS {
            return self.ema_speed.load(AtomicOrdering::SeqCst);
        }

        let current_bytes = self.total_bytes.load(AtomicOrdering::SeqCst);
        let last_bytes = self.last_bytes.swap(current_bytes, AtomicOrdering::SeqCst);
        let bytes_diff = current_bytes.saturating_sub(last_bytes);

        // Calculate instantaneous speed
        let elapsed_secs = elapsed.as_secs_f64();
        let instant_speed = if elapsed_secs > 0.0 {
            (bytes_diff as f64 / elapsed_secs) as u64
        } else {
            0
        };

        // Apply EMA smoothing
        let prev_ema = self.ema_speed.load(AtomicOrdering::SeqCst);
        let new_ema = if prev_ema == 0 {
            instant_speed
        } else {
            ((EMA_ALPHA * instant_speed as f64) + ((1.0 - EMA_ALPHA) * prev_ema as f64)) as u64
        };

        self.ema_speed.store(new_ema, AtomicOrdering::SeqCst);
        *last_update = now;

        new_ema
    }

    /// Get the current smoothed speed without updating.
    pub fn get_speed(&self) -> u64 {
        self.ema_speed.load(AtomicOrdering::SeqCst)
    }

    /// Reset the tracker for reuse.
    pub fn reset(&self) {
        self.total_bytes.store(0, AtomicOrdering::SeqCst);
        self.last_bytes.store(0, AtomicOrdering::SeqCst);
        self.ema_speed.store(0, AtomicOrdering::SeqCst);
        *self.last_update.lock().unwrap() = Instant::now();
    }
}

impl Default for SpeedTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct DownloadRateLimiterState {
    window_start: tokio::time::Instant,
    used_in_window: u64,
}

#[derive(Debug)]
pub struct DownloadRateLimiter {
    limit_bps: AtomicU64,
    state: Mutex<DownloadRateLimiterState>,
}

impl DownloadRateLimiter {
    fn new() -> Self {
        Self {
            limit_bps: AtomicU64::new(0),
            state: Mutex::new(DownloadRateLimiterState {
                window_start: tokio::time::Instant::now(),
                used_in_window: 0,
            }),
        }
    }

    fn set_limit_bps(&self, limit_bps: u64) {
        self.limit_bps.store(limit_bps, AtomicOrdering::Relaxed);
    }

    async fn consume(&self, mut bytes: u64) {
        let limit_bps = self.limit_bps.load(AtomicOrdering::Relaxed);
        if limit_bps == 0 || bytes == 0 {
            return;
        }

        // Fixed-window limiter to keep the "instantaneous" rate close to the configured cap.
        // This avoids long sleep / burst patterns that can show up as spikes in Task Manager.
        const WINDOW_MS: u64 = 100;
        let window = Duration::from_millis(WINDOW_MS);

        // Integer budget for this window; at least 1 byte to avoid deadlock at tiny limits.
        let window_budget = ((limit_bps as u128 * WINDOW_MS as u128) / 1000u128).max(1) as u64;

        while bytes > 0 {
            let now = tokio::time::Instant::now();
            let mut sleep_until: Option<tokio::time::Instant> = None;
            let mut allowed_now = 0u64;

            {
                let mut state = self.state.lock().await;
                if now.duration_since(state.window_start) >= window {
                    state.window_start = now;
                    state.used_in_window = 0;
                }

                let remaining = window_budget.saturating_sub(state.used_in_window);
                if remaining == 0 {
                    sleep_until = Some(state.window_start + window);
                } else {
                    allowed_now = remaining.min(bytes);
                    state.used_in_window = state.used_in_window.saturating_add(allowed_now);
                }
            }

            if let Some(until) = sleep_until {
                tokio::time::sleep_until(until).await;
                continue;
            }

            bytes = bytes.saturating_sub(allowed_now);
        }
    }
}

static GLOBAL_DOWNLOAD_RATE_LIMITER: OnceLock<DownloadRateLimiter> = OnceLock::new();

fn global_download_rate_limiter() -> &'static DownloadRateLimiter {
    GLOBAL_DOWNLOAD_RATE_LIMITER.get_or_init(DownloadRateLimiter::new)
}

/// Sets the global download speed limit in KiB/s. Set to 0 for unlimited.
///
/// This limiter is shared across all concurrent downloads running inside this process.
pub fn set_global_download_speed_limit_kib(kib_per_sec: u64) {
    let bps = kib_per_sec.saturating_mul(1024);
    global_download_rate_limiter().set_limit_bps(bps);
}

pub const DEFAULT_CHUNK_SIZE: usize = 128 * 1024; // 128 KiB
const RATE_LIMIT_SLICE_SIZE: usize = 8 * 1024; // 8 KiB

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownloadingError {
    #[error("Path is not mounted: {0:?}")]
    PathNotMounted(PathBuf),
    #[error("No free space available for specified path: {0:?} (requires {}, available {})", prettify_bytes(*.1), prettify_bytes(*.2))]
    NoSpaceAvailable(PathBuf, u64, u64),
    #[error("Failed to create output file {0:?}: {1}")]
    OutputFileError(PathBuf, String),
    #[error("Failed to read metadata of the output file {0:?}: {1}")]
    OutputFileMetadataError(PathBuf, String),
    #[error("Request error: {0}")]
    Reqwest(String),
    #[error("Download cancelled")]
    Cancelled,
}

impl From<reqwest::Error> for DownloadingError {
    fn from(error: reqwest::Error) -> Self {
        DownloadingError::Reqwest(error.to_string())
    }
}

#[derive(Debug)]
pub struct AsyncDownloader {
    uri: String,
    length: Option<u64>,
    pub chunk_size: usize,
    pub continue_downloading: bool,
    pub check_free_space: bool,
    client: Arc<ClientWithMiddleware>,
    cancel_token: Option<Arc<AtomicBool>>,
}

impl AsyncDownloader {
    pub async fn setup_client() -> ClientWithMiddleware {
        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(30);
        let c = reqwest::Client::builder()
            .read_timeout(std::time::Duration::from_secs(30))
            .use_native_tls()
            .no_brotli()
            .no_gzip()
            .no_deflate()
            .no_zstd()
            .build()
            .unwrap();
        reqwest_middleware::ClientBuilder::new(c)
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build()
    }

    pub async fn new<T: AsRef<str>>(
        client: Arc<ClientWithMiddleware>,
        uri: T,
    ) -> Result<Self, reqwest::Error> {
        let uri = uri.as_ref();

        let header = client
            .head(uri)
            .header(USER_AGENT, "lib/fischl-rs")
            .send()
            .await
            .unwrap();
        let length = header.headers().get("content-length").map(|len| {
            len.to_str()
                .unwrap()
                .parse()
                .expect("Requested site's content-length is not a number")
        });

        Ok(Self {
            uri: uri.to_owned(),
            length,
            chunk_size: DEFAULT_CHUNK_SIZE,
            continue_downloading: true,
            check_free_space: true,
            cancel_token: None,
            client: client,
        })
    }

    #[inline]
    pub fn with_chunk_size(mut self, chunk_size: usize) -> Self {
        self.chunk_size = chunk_size;
        self
    }

    #[inline]
    pub fn with_continue_downloading(mut self, continue_downloading: bool) -> Self {
        self.continue_downloading = continue_downloading;
        self
    }

    #[inline]
    pub fn with_free_space_check(mut self, check_free_space: bool) -> Self {
        self.check_free_space = check_free_space;
        self
    }

    #[inline]
    pub fn with_cancel_token(mut self, cancel_token: Option<Arc<AtomicBool>>) -> Self {
        self.cancel_token = cancel_token;
        self
    }
    pub fn length(&self) -> Option<u64> {
        self.length
    }

    pub async fn get_filename(&self) -> &str {
        if let Some(pos) = self.uri.replace('\\', "/").rfind(|c| c == '/') {
            if !self.uri[pos + 1..].is_empty() {
                return &self.uri[pos + 1..];
            }
        }
        "index.html"
    }

    pub async fn download(
        &mut self,
        path: impl Into<PathBuf>,
        mut progress: impl FnMut(u64, u64, u64, u64) + Send + Sync + 'static,
    ) -> Result<(), DownloadingError> {
        let path = path.into();
        let mut downloaded = 0;

        // Open or create output file
        let file = if path.exists() && self.continue_downloading {
            let mut file = tokio::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(&path)
                .await;

            // Continue downloading if the file exists and can be opened
            if let Ok(file) = &mut file {
                match file.metadata().await {
                    Ok(metadata) => {
                        // Stop the process if the file is already downloaded
                        if let Some(length) = self.length() {
                            match metadata.len().cmp(&length) {
                                std::cmp::Ordering::Less => (),
                                std::cmp::Ordering::Equal => return Ok(()),
                                // Trim downloaded file to prevent future issues (e.g. with extracting the archive)
                                std::cmp::Ordering::Greater => {
                                    if let Err(err) = file.set_len(length).await {
                                        return Err(DownloadingError::OutputFileError(
                                            path,
                                            err.to_string(),
                                        ));
                                    }
                                    return Ok(());
                                }
                            }
                        }

                        if let Err(err) =
                            file.seek(tokio::io::SeekFrom::Start(metadata.len())).await
                        {
                            return Err(DownloadingError::OutputFileError(path, err.to_string()));
                        }
                        downloaded = metadata.len() as usize;
                    }

                    Err(err) => {
                        return Err(DownloadingError::OutputFileMetadataError(
                            path,
                            err.to_string(),
                        ));
                    }
                }
            }

            file
        } else {
            let base_folder = path.parent().unwrap();
            if !base_folder.exists() {
                if let Err(err) = tokio::fs::create_dir_all(base_folder).await {
                    return Err(DownloadingError::OutputFileError(path, err.to_string()));
                }
            }
            tokio::fs::File::create(&path).await
        };

        // Check available free space
        if self.check_free_space {
            match free_space::available(&path) {
                Some(space) => {
                    if let Some(required) = self.length() {
                        let required = required.checked_sub(downloaded as u64).unwrap_or_default();
                        if space < required {
                            return Err(DownloadingError::NoSpaceAvailable(path, required, space));
                        }
                    }
                }
                None => return Err(DownloadingError::PathNotMounted(path)),
            }
        }

        // Download data
        match file {
            Ok(mut file) => {
                let request = self
                    .client
                    .head(&self.uri)
                    .header(RANGE, format!("bytes={downloaded}-"))
                    .header(USER_AGENT, "lib/fischl-rs")
                    .send()
                    .await
                    .unwrap();

                // Request content range (downloaded + remained content size)
                // If finished or overcame: bytes */10611646760
                // If not finished: bytes 10611646759-10611646759/10611646760
                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Range
                if let Some(range) = request.headers().get("content-range") {
                    // Finish downloading if header says that we've already downloaded all the data
                    if range.to_str().unwrap().contains("*/") {
                        progress(
                            self.length.unwrap_or(downloaded as u64),
                            self.length.unwrap_or(downloaded as u64),
                            0,
                            0,
                        );
                        return Ok(());
                    }
                }

                let request = self
                    .client
                    .get(&self.uri)
                    .header(RANGE, format!("bytes={downloaded}-"))
                    .header(USER_AGENT, "lib/fischl-rs")
                    .send()
                    .await
                    .unwrap();

                // HTTP 416 = provided range is overcame actual content length (means file is downloaded)
                // I check this here because HEAD request can return 200 OK while GET - 416
                // https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/416
                if request.status() == StatusCode::RANGE_NOT_SATISFIABLE {
                    progress(
                        self.length.unwrap_or(downloaded as u64),
                        self.length.unwrap_or(downloaded as u64),
                        0,
                        0,
                    );
                    return Ok(());
                }

                let mut stream = request.bytes_stream();
                let net_tracker = SpeedTracker::new();
                let disk_tracker = SpeedTracker::new();
                let mut last_update = Instant::now();
                let mut written_bytes = downloaded as u64;

                while let Some(chunk) = stream.next().await {
                    if let Some(token) = &self.cancel_token {
                        if token.load(Ordering::Relaxed) {
                            return Err(DownloadingError::Cancelled);
                        }
                    }
                    let data = chunk?;
                    net_tracker.add_bytes(data.len() as u64);

                    for part in data.chunks(RATE_LIMIT_SLICE_SIZE) {
                        if let Some(token) = &self.cancel_token {
                            if token.load(Ordering::Relaxed) {
                                return Err(DownloadingError::Cancelled);
                            }
                        }

                        global_download_rate_limiter()
                            .consume(part.len() as u64)
                            .await;
                        file.write_all(part).await.unwrap();
                        written_bytes += part.len() as u64;
                        disk_tracker.add_bytes(part.len() as u64);

                        let now = Instant::now();
                        if now.duration_since(last_update).as_millis() >= 500 {
                            let net_speed = net_tracker.update();
                            let disk_speed = disk_tracker.update();
                            progress(
                                written_bytes,
                                self.length.unwrap_or(written_bytes),
                                net_speed,
                                disk_speed,
                            );
                            last_update = now;
                        }
                    }
                }

                if let Err(err) = file.flush().await {
                    return Err(DownloadingError::OutputFileError(path, err.to_string()));
                }
                drop(file);
                Ok(())
            }
            Err(err) => Err(DownloadingError::OutputFileError(path, err.to_string())),
        }
    }
}
