use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{WebSocket, Request};

#[wasm_bindgen]
pub struct BrowserNetwork {
    websocket: Option<WebSocket>,
    fetch_client: Option<Request>,
    connected: bool,
}

#[wasm_bindgen]
impl BrowserNetwork {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BrowserNetwork, JsValue> {
        Ok(BrowserNetwork {
            websocket: None,
            fetch_client: None,
            connected: false,
        })
    }

    pub fn connect(&mut self, url: &str) -> Result<(), JsValue> {
        let socket = WebSocket::new(url).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        self.websocket = Some(socket);
        self.connected = true;
        Ok(())
    }

    pub fn send(&self, data: &[u8]) -> Result<(), JsValue> {
        if let Some(ref socket) = self.websocket {
            let array = js_sys::Uint8Array::from(data);
            socket.send_with_u8_array(&array).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn receive(&mut self) -> Result<Option<Vec<u8>>, JsValue> {
        if let Some(ref socket) = self.websocket {
            if let Some(msg) = socket.data() {
                if let Ok(s) = msg.as_string() {
                    return Ok(Some(s.into_bytes()));
                }
            }
        }
        Ok(None)
    }

    pub fn fetch(&self, url: &str, method: &str, body: &[u8]) -> Result<Vec<u8>, JsValue> {
        let mut opts = js_sys::Object::new();
        js_sys::Reflect::set(&opts, &"method".into(), &method.into()).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        if !body.is_empty() {
            js_sys::Reflect::set(&opts, &"body".into(), &js_sys::Uint8Array::from(body).buffer()).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let _window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        Ok(vec![])
    }

    pub fn is_connected(&self) -> bool { self.connected }
}

impl Default for BrowserNetwork {
    fn default() -> Self {
        BrowserNetwork::new().unwrap()
    }
}
