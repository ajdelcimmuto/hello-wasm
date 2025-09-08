use wasm_bindgen::prelude::*;
use wasm_logger;
use log;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Headers};
use m3u8_rs::{MasterPlaylist, MediaPlaylist, Playlist};

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
    pub fn segment_info(s: &[u8]);
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
        let text_string = text.as_string().unwrap();

        let master_playlist = self.parse_manifest(text_string).unwrap();

        match &master_playlist {
            Playlist::MasterPlaylist(pl) => {
                let media_playlist = self.fetch_media_playlist(pl).await?;
            },
            _ => {}
        }

        Ok(String::new())
    }

    fn parse_manifest(&mut self, manifest: String) -> Result<m3u8_rs::Playlist, String> {
        let text_bytes: &[u8] = manifest.as_bytes();

        let playlist = m3u8_rs::parse_playlist_res(text_bytes).expect("Manifest could not be parsed.");

        match &playlist {
            Playlist::MasterPlaylist(pl) => log::info!("Master playlist:\n{:?}", pl),
            Playlist::MediaPlaylist(pl) => log::info!("Media playlist:\n{:?}", pl)
        }

        Ok(playlist)
    }

    async fn fetch_media_playlist(&mut self, master_playlist: &MasterPlaylist) -> Result<String, JsValue> {
        log::info!("fetch_media_playlist");

        let opts = RequestInit::new();
        opts.set_method("GET");

        let headers = Headers::new()?;
        headers.set("Accept", "application/vnd.apple.mpegurl")?;
        opts.set_headers(&headers);

        // Fetch first variant for POC purposes
        let base_url = "https://stream-fastly.castr.com/5b9352dbda7b8c769937e459/live_2361c920455111ea85db6911fe397b9e/";
        let url = format!("{}{}", base_url.to_string(), master_playlist.variants[0].uri);
        let request = Request::new_with_str_and_init(url.as_str(), &opts)?;
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}: {}",
                resp.status(), resp.status_text())));
        }

        let text = JsFuture::from(resp.text()?).await?;
        let text_string = text.as_string().unwrap();

        let media_playlist = self.parse_manifest(text_string).unwrap();

        match &media_playlist {
            Playlist::MediaPlaylist(pl) => {
                let media_playlist = self.fetch_media_segment(pl).await?;
            },
            _ => {}
        }

        Ok(String::new())
    }

    async fn fetch_media_segment(&mut self, media_playlist: &MediaPlaylist) -> Result<String, JsValue> {
        log::info!("fetch_media_segment");

        let opts = RequestInit::new();
        opts.set_method("GET");

        let headers = Headers::new()?;
        opts.set_headers(&headers);

        let base_url = "https://stream-fastly.castr.com/5b9352dbda7b8c769937e459/live_2361c920455111ea85db6911fe397b9e/tracks-v3/";
        // Fetch the init segment
        let segment = &media_playlist.segments[0];
        let map = segment.map.as_ref().unwrap();
        let url = format!("{}{}", base_url.to_string(), map.uri);
        let request = Request::new_with_str_and_init(url.as_str(), &opts)?;
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}: {}",
                resp.status(), resp.status_text())));
        }

        let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);

        log::info!("Segment: {} bytes", bytes.len());
        segment_info(&bytes);

        // Fetch first variant for POC purposes
        let url = format!("{}{}", base_url.to_string(), &media_playlist.segments[0].uri);
        let request = Request::new_with_str_and_init(url.as_str(), &opts)?;
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}: {}",
                resp.status(), resp.status_text())));
        }

        let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let mut bytes = vec![0; uint8_array.length() as usize];
        uint8_array.copy_to(&mut bytes);

        log::info!("Segment: {} bytes", bytes.len());
        segment_info(&bytes);

        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_parse_manifest() {
        let manifest = "#EXTM3U\n#EXT-X-VERSION:7\n#EXT-X-TARGETDURATION:6\n#EXTINF:5.009,\nhttps://media.example.com/first.ts\n#EXTINF:5.009,\nhttps://media.example.com/second.ts\n#EXTINF:3.003,\nhttps://media.example.com/third.ts\n#EXT-X-ENDLIST";

        let manifest_string = manifest.to_string();
        let mut streaming_client = StreamingClient::new(String::new());
        let manifest = streaming_client.parse_manifest(manifest_string).expect("manifest failed to parse");

        match &manifest {
            Playlist::MasterPlaylist(pl) => {
                log::info!("Master playlist:\n{:?}", pl);

                assert!(!pl.variants.is_empty());
            },
            Playlist::MediaPlaylist(pl) => {
                log::info!("Media playlist:\n{:?}", pl);
            }
        }
    }
}
