use m3u8_rs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

use crate::http::downloader::DownloadError;

pub static HLS_URL: &str = "https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8";

// Fetch the HLS playlist from WASM
async fn get_playlist(url: &str) -> Result<m3u8_rs::MasterPlaylist, DownloadError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request = Request::new_with_str_and_init(&url, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let resp = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| DownloadError::NetworkError)?
        .dyn_into::<Response>()
        .map_err(|_| DownloadError::DataError)?;

    if resp.status() != 200 {
        return Err(DownloadError::HttpError(resp.status()));
    }

    let data = resp
        .text()
        .and_then(|p| Ok(JsFuture::from(p)))
        .map_err(|_| DownloadError::DataError)?
        .await
        .map_err(|_| DownloadError::DataError)?
        .as_string()
        .ok_or(DownloadError::DataError)?;


    let playlist = m3u8_rs::parse_master_playlist_res(data.as_bytes()).map_err(|_| DownloadError::DataError)?;

    Ok(playlist)
}
