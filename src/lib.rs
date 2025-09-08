use wasm_bindgen::prelude::*;
use wasm_logger;
use log;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Headers};
use m3u8_rs::{Playlist};

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn set_url(url: &str) {
    alert(&format!("URL, {}!", url));
}

#[wasm_bindgen]
pub struct StreamingClient {
    base_url: String
}

#[wasm_bindgen]
impl StreamingClient {
    #[wasm_bindgen(constructor)]
    pub fn new(base_url: String) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        Self {
            base_url
        }
    }

    pub async fn fetch_manifest(&mut self) -> Result<String, JsValue> {
        log::info!("fetch_manifest");

        let opts = RequestInit::new();
        opts.set_method("GET");

        let headers = Headers::new()?;
        headers.set("Accept", "application/vnd.apple.mpegurl")?;
        opts.set_headers(&headers);

        let request = Request::new_with_str_and_init(&self.base_url, &opts)?;
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}: {}",
                resp.status(), resp.status_text())));
        }

        let text = JsFuture::from(resp.text()?).await?;

        let text_string: String = text.as_string().unwrap();
        let text_bytes: &[u8] = text_string.as_bytes();

        let playlist = m3u8_rs::parse_playlist_res(text_bytes);

        match playlist {
            Ok(Playlist::MasterPlaylist(pl)) => log::info!("Master playlist:\n{:?}", pl),
            Ok(Playlist::MediaPlaylist(pl)) => log::info!("Media playlist:\n{:?}", pl),
            Err(e) => println!("Error: {:?}", e),
        }

        Ok(text_string)
    }

    // async fn fetch_segments(&mut self, playlist: &m3u8_rs::Playlist) -> Result<Vec<Vec<u8>>, JsValue> {
    //     let playlist_seg_iter = playlist.get_segment_url_vec().iter();
    //     let http_base = playlist.get_http_base();
    //     let mut seg_data: Vec<Vec<u8>> = Vec::new();
    //     let url = http_base.replace("{SEG_PREFIX}", playlist.get_segment_url_vec()[0].as_str());
    //     dbg!(&url);

    //     let headers = Headers::new()?;
    //     headers.set("Accept", "application/vnd.apple.mpegurl")?;
    //     opts.set_headers(&headers);

    //     let request = Request::new_with_str_and_init(&self.base_url, &opts)?;
    //     let window = web_sys::window().unwrap();

    //     let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    //     let resp: web_sys::Response = resp_value.dyn_into()?;

    //     if !resp.ok() {
    //         return Err(JsValue::from_str(&format!("HTTP {}: {}",
    //             resp.status(), resp.status_text())));
    //     }

    //     let text = JsFuture::from(resp.text()?).await?;

    //     Ok(seg_data)
    // }
}
