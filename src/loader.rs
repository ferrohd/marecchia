use m3u8_rs;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};

static HLS_URL: &str = "https://demo.unified-streaming.com/k8s/features/stable/video/tears-of-steel/tears-of-steel.ism/.m3u8";

// Fetch the HLS playlist from WASM
async fn get_playlist(url: &str) -> Result<m3u8_rs::MasterPlaylist, ()> {
    let mut opts = RequestInit::new();
    opts.method("GET");
    opts.mode(RequestMode::Cors);

    let request =
        Request::new_with_str_and_init(&url, &opts).unwrap();

    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request))
        .await
        .unwrap();

    let resp: Response = resp_value.unchecked_into::<Response>();

    if resp.status() != 200 {
        return Err(());
    }

    let data = resp.as_string().unwrap();

    let playlist = m3u8_rs::parse_master_playlist_res(data.as_bytes()).unwrap();
    
    Ok(playlist)
}