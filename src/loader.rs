use hls_m3u8;
use js_sys::JsString;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

static HLS_URL: &str = "https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8";

// Fetch the HLS playlist from WASM
async fn get_playlist(url: &str) -> Result<hls_m3u8::MasterPlaylist, hls_m3u8::Error> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request =
        Request::new_with_str_and_init(&url, &opts).map_err(|x| hls_m3u8::Error::from(x))?;

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| hls_m3u8::Error::InvalidPlaylist)?;

    let resp: Response = resp_value.unchecked_into::<Response>();

    if resp.status() != 200 {
        return Err(hls_m3u8::Error::InvalidPlaylist);
    }

    let data = resp.as_string().unwrap();

    let playlist = data.as_ref().parse()?;
    
    Ok(playlist)
}