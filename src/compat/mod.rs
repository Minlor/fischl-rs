#[cfg(feature = "compat")]
use std::path::PathBuf;
#[cfg(feature = "compat")]
use crate::utils::downloader::AsyncDownloader;
#[cfg(feature = "compat")]
use crate::utils::extract_archive_with_progress;
#[cfg(feature = "compat")]
use wincompatlib::wine::Wine;

#[cfg(feature = "compat")]
pub mod prefix;
#[cfg(feature = "compat")]
pub mod dxvk;

#[cfg(feature = "compat")]
pub struct Compat {
    pub wine: Wine,
}

#[cfg(feature = "compat")]
pub async fn download_steamrt(path: PathBuf, dest: PathBuf, edition: String, branch: String, progress: impl FnMut(u64, u64, u64, u64) + Send + Sync + 'static, extract_progress: impl Fn(u64, u64) + Send + 'static) -> bool {
    if !path.exists() || edition.is_empty() || branch.is_empty() { false } else {
        let code = if edition.as_str() == "steamrt3" { "sniper" } else if edition.as_str() == "steamrt4" { "4" } else { return false };
        let archive = if cfg!(target_arch = "aarch64") { format!("SteamLinuxRuntime_{code}-arm64.tar.xz") } else if cfg!(target_arch = "x86_64") { format!("SteamLinuxRuntime_{code}.tar.xz") } else { return false; };

        let token = crate::utils::url_safe_token(22); // Bypass Cloudflare cache
        let url = format!("https://repo.steampowered.com/{edition}/images/{branch}/{archive}?versions={token}");
        let cl = AsyncDownloader::setup_client().await;
        let dl = AsyncDownloader::new(std::sync::Arc::new(cl), url).await;
        if dl.is_ok() {
            let mut d = dl.unwrap();
            let p = path.join(archive);
            let dld = d.download(p.as_path(), progress).await;
            if dld.is_ok() {
                let ext = extract_archive_with_progress(p.to_str().unwrap().to_string(), dest.to_str().unwrap().to_string(), true, extract_progress);
                if ext { true } else { false }
            } else { false }
        } else { false }
    }
}

#[cfg(feature = "compat")]
pub fn check_steamrt_update(edition: String, branch: String) -> Option<String> {
    if edition.is_empty() || branch.is_empty() { None } else {
        let token = crate::utils::url_safe_token(22); // Bypass Cloudflare cache
        let url = format!("https://repo.steampowered.com/{edition}/images/{branch}/VERSION.txt?versions={token}");
        let req = reqwest::blocking::get(url);
        match req {
            Ok(response) => {
                match response.text() {
                    Ok(version_text) => { Some(version_text) },
                    Err(_) => { None },
                }
            }
            Err(_) => { None },
        }
    }
}