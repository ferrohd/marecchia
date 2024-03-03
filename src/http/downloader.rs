use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

#[derive(Debug)]
pub enum DownloadError {
    // The Network is unreachable
    NetworkError,
    // The HTTP request failed with status code
    HttpError(u16),
    // Cannot parse the response as bytes
    DataError,
    // Window variable is not defined
    WindowError,
}

pub async fn download_segment(url: &str) -> Result<Vec<u8>, DownloadError> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request =
        Request::new_with_str_and_init(url, &opts).map_err(|_| DownloadError::NetworkError)?;

    let window = web_sys::window().ok_or(DownloadError::WindowError)?;
    let resp = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| DownloadError::NetworkError)?
        .dyn_into::<Response>()
        .map_err(|_| DownloadError::DataError)?;

    if resp.status() != 200 {
        return Err(DownloadError::HttpError(resp.status()));
    }

    let data = resp
        .array_buffer()
        .map(JsFuture::from)
        .map_err(|_| DownloadError::DataError)?
        .await
        .map_err(|_| DownloadError::DataError)?;
    //.unchecked_into::<ArrayBuffer>();

    let data = js_sys::Uint8Array::new(&data).to_vec();

    Ok(data)
}
