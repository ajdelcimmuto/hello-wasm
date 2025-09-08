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
pub struct WasmHttpClient {
    base_url: Option<String>,
    default_headers: Headers,
    timeout_ms: u32
}

pub struct WasmResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: Headers,
    pub body: String,
    pub ok: bool,
}

impl WasmHttpClient {
    pub fn new() -> Self {
        Self {
            base_url: None,
            default_headers: Headers::new().unwrap(),
            timeout_ms: 10000
        }
    }

    pub fn with_base_url(&mut self, base_url: &str) {
        self.base_url = Some(base_url.to_string());
    }

    pub fn add_default_header(&mut self, key: &str, value: &str) {
        self.default_headers.append(key, value).unwrap();
    }

    pub fn set_timeout(&mut self, timeout_ms: u32) {
        self.timeout_ms = timeout_ms;
    }

    pub async fn get(&self) -> Result<WasmResponse, JsValue> {
        let opts = RequestInit::new();
        opts.set_method("GET");

        opts.set_headers(self.default_headers.as_ref());

        let url = match &self.base_url {
            Some(base) => base.as_str(),
            None => return Err(JsValue::from_str("No base URL set")),
        };

        let request = Request::new_with_str_and_init(url, &opts)?;
        let window = web_sys::window().unwrap();

        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
        let resp: web_sys::Response = resp_value.dyn_into()?;

        if !resp.ok() {
            return Err(JsValue::from_str(&format!("HTTP {}: {}",
                resp.status(), resp.status_text())));
        }

        let text = JsFuture::from(resp.text()?).await?;
        let text_string = text.as_string().unwrap();

        return Ok(WasmResponse {
            status: resp.status(),
            status_text: resp.status_text(),
            headers: resp.headers(),
            body: text_string,
            ok: true
        })
    }
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

        let mut wasm_http_client = WasmHttpClient::new();
        wasm_http_client.with_base_url(&self.base_url);
        wasm_http_client.add_default_header("Accept", "application/vnd.apple.mpegurl");
        let master_playlist_response: WasmResponse = wasm_http_client.get().await?;

        let master_playlist = self.parse_manifest(master_playlist_response.body).unwrap();

        match &master_playlist {
            Playlist::MasterPlaylist(pl) => {
                self.fetch_media_playlist(pl).await?;
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

        let base_url = "https://stream-fastly.castr.com/5b9352dbda7b8c769937e459/live_2361c920455111ea85db6911fe397b9e/";
        let mut wasm_http_client = WasmHttpClient::new();
        wasm_http_client.with_base_url(format!("{}{}", base_url.to_string(), master_playlist.variants[0].uri).as_str());
        wasm_http_client.add_default_header("Accept", "application/vnd.apple.mpegurl");
        let media_playlist_response: WasmResponse = wasm_http_client.get().await?;

        let media_playlist = self.parse_manifest(media_playlist_response.body).unwrap();

        match &media_playlist {
            Playlist::MediaPlaylist(pl) => {
                self.fetch_media_segment(pl).await?;
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
