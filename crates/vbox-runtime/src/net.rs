#![allow(non_camel_case_types)]

use std::cell::RefCell;
use parking_lot::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, Response, Headers, WebSocket};
use url::Url;
use vbox_core::{VBoxResult, VBoxError};

pub const RT_NET_AF_INET: i32 = 2;
pub const RT_NET_AF_INET6: i32 = 10;
pub const RT_NET_SOCK_STREAM: i32 = 1;
pub const RT_NET_SOCK_DGRAM: i32 = 2;
pub const RT_NET_INVALID_PORT: u16 = 0;

#[repr(C)]
pub struct RTNETADDR {
    pub af: i32,
    pub u16Port: u16,
    pub pvData: Vec<u8>,
    pub cbData: usize,
    pub pszAddress: String,
}

#[repr(C)]
pub struct RTNETREQ {
    pub u32ReqId: u32,
    pub pszUrl: String,
    pub cHeaders: usize,
    pub fAsync: bool,
    pub pvUser: usize,
}

#[repr(C)]
pub struct RTNETRES {
    pub u32ReqId: u32,
    pub u16Status: u16,
    pub pvData: Vec<u8>,
    pub cbData: usize,
    pub cHeaders: usize,
}

pub struct RTNetBackend {
    pending_requests: Mutex<Vec<PendingRequest>>,
}

struct PendingRequest {
    url: String,
    method: String,
    headers: Vec<(String, String)>,
    body: Option<Vec<u8>>,
}

impl RTNetBackend {
    pub fn new() -> Self {
        Self {
            pending_requests: Mutex::new(Vec::new()),
        }
    }

    pub async fn fetch(&self, url: &str, method: &str, body: Option<Vec<u8>>) -> VBoxResult<Vec<u8>> {
        let mut opts = js_sys::Object::new();
        js_sys::Reflect::set(&opts, &"method".into(), &method.into()).map_err(|_| VBoxError::InvalidParam)?;
        if let Some(ref body_data) = body {
            js_sys::Reflect::set(&opts, &"body".into(), &JsValue::from_serde(body_data).map_err(|_| VBoxError::InvalidParam)?).map_err(|_| VBoxError::InvalidParam)?;
        }
        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|_| VBoxError::InvalidParam)?;
        let window = web_sys::window().ok_or(VBoxError::NotFound)?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request)).await
            .map_err(|_| VBoxError::ConnectionRefused)?;
        let response: Response = resp_value.dyn_into().map_err(|_| VBoxError::Unknown("Response conversion failed".to_string()))?;
        let data = JsFuture::from(response.array_buffer().map_err(|_| VBoxError::Unknown("ArrayBuffer conversion failed".to_string()))?).await
            .map_err(|_| VBoxError::Unknown("Fetch await failed".to_string()))?;
        let array: js_sys::Uint8Array = js_sys::Uint8Array::new(&data);
        Ok(array.to_vec())
    }
}

pub struct RTNetwork {
    backend: RTNetBackend,
}

impl RTNetwork {
    pub fn new() -> Self {
        Self { backend: RTNetBackend::new() }
    }

    pub async fn get(&self, url: &str) -> VBoxResult<Vec<u8>> {
        self.backend.fetch(url, "GET", None).await
    }

    pub async fn post(&self, url: &str, body: Vec<u8>) -> VBoxResult<Vec<u8>> {
        self.backend.fetch(url, "POST", Some(body)).await
    }

    pub async fn put(&self, url: &str, body: Vec<u8>) -> VBoxResult<Vec<u8>> {
        self.backend.fetch(url, "PUT", Some(body)).await
    }

    pub async fn delete(&self, url: &str) -> VBoxResult<Vec<u8>> {
        self.backend.fetch(url, "DELETE", None).await
    }
}

impl Default for RTNetwork {
    fn default() -> Self {
        Self::new()
    }
}

pub struct RTSSLSocket {
    ws: Option<WebSocket>,
    url: String,
}

impl RTSSLSocket {
    pub fn new(url: &str) -> VBoxResult<Self> {
        Ok(Self {
            ws: None,
            url: url.to_string(),
        })
    }

    pub fn connect(&mut self) -> VBoxResult<()> {
        let mut socket = WebSocket::new(&self.url)
            .map_err(|_| VBoxError::ConnectionRefused)?;
        socket.set_onopen(Some(&Closure::new(|_e: web_sys::Event| {}).into_js_value().as_ref().unchecked_ref()));
        socket.set_onmessage(Some(&Closure::new(|_e: web_sys::MessageEvent| {}).into_js_value().as_ref().unchecked_ref()));
        socket.set_onerror(Some(&Closure::new(|_e: web_sys::Event| {}).into_js_value().as_ref().unchecked_ref()));
        self.ws = Some(socket);
        Ok(())
    }
}

pub fn rt_net_fetch(url: &str) -> VBoxResult<Vec<u8>> {
    wasm_bindgen_futures::spawn_local(async {
        let net = RTNetwork::new();
        match net.get(url).await {
            Ok(data) => {}
            Err(e) => {}
        }
    });
    Err(VBoxError::InProgress)
}

pub fn rt_net_get(url: &str) -> VBoxResult<Vec<u8>> {
    let net = RTNetwork::new();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = net.get(url).await;
    });
    Err(VBoxError::InProgress)
}

pub fn rt_net_post(url: &str, body: &[u8]) -> VBoxResult<Vec<u8>> {
    let net = RTNetwork::new();
    let body = body.to_vec();
    wasm_bindgen_futures::spawn_local(async move {
        let _ = net.post(url, body).await;
    });
    Err(VBoxError::InProgress)
}

pub fn rt_net_parse_url(url: &str) -> VBoxResult<Url> {
    Url::parse(url).map_err(|_| VBoxError::BadPath)
}

pub fn rt_net_is_valid_ip(addr: &str) -> bool {
    addr.parse::<std::net::IpAddr>().is_ok()
}

pub fn rt_net_get_local_ip() -> VBoxResult<String> {
    Ok("127.0.0.1".to_string())
}

pub fn rt_net_resolve(hostname: &str) -> VBoxResult<Vec<String>> {
    Ok(vec![hostname.to_string()])
}

pub fn rt_net_get_address_info(addr: &str, port: u16) -> VBoxResult<RTNETADDR> {
    Ok(RTNETADDR {
        af: RT_NET_AF_INET,
        u16Port: port,
        pvData: addr.as_bytes().to_vec(),
        cbData: addr.len(),
        pszAddress: addr.to_string(),
    })
}

pub fn rt_net_socket_create() -> VBoxResult<WebSocket> {
    WebSocket::new("ws://localhost").map_err(|_| VBoxError::ConnectionRefused)
}

pub fn rt_net_is_valid_ipv4(addr: &str) -> bool {
    addr.parse::<std::net::Ipv4Addr>().is_ok()
}

pub fn rt_net_is_valid_ipv6(addr: &str) -> bool {
    addr.parse::<std::net::Ipv6Addr>().is_ok()
}

pub fn rt_net_is_loopback(addr: &str) -> bool {
    addr == "127.0.0.1" || addr == "::1"
}

pub fn rt_net_is_link_local(addr: &str) -> bool {
    addr.starts_with("169.254.") || addr.starts_with("fe80::")
}
