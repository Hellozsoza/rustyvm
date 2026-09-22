use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::{Storage, IndexedDb, IDBCursor, File, FileList, FileSystemFileHandle, Blob, Url};
use uuid::Uuid;

#[wasm_bindgen]
pub struct BrowserStorage {
    local_storage: Option<Storage>,
    indexed_db: Option<IndexedDb>,
}

#[wasm_bindgen]
impl BrowserStorage {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<BrowserStorage, JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let local_storage = window.local_storage().map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;

        Ok(BrowserStorage {
            local_storage,
            indexed_db: None,
        })
    }

    pub fn save_vm_state(&self, uuid: Uuid, data: &[u8]) -> Result<(), JsValue> {
        let key = format!("vm_{}_state", uuid);
        if let Some(ref storage) = self.local_storage {
            let array = js_sys::Uint8Array::from(data);
            storage.set_item(&key, &array.buffer()).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        }
        Ok(())
    }

    pub fn load_vm_state(&self, uuid: Uuid) -> Result<Option<Vec<u8>>, JsValue> {
        let key = format!("vm_{}_state", uuid);
        if let Some(ref storage) = self.local_storage {
            if let Some(data) = storage.get_item(&key).map_err(|e| JsValue::from_str(&format!("{:?}", e)))? {
                return Ok(Some(data.into_bytes()));
            }
        }
        Ok(None)
    }

    pub fn save_vm_to_file(&self, uuid: Uuid, data: &[u8]) -> Result<(), JsValue> {
        let blob = Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::of1(&js_sys::Uint8Array::from(data).buffer()),
            web_sys::BlobPropertyBag::new().type_("application/octet-stream"),
        ).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let url = Url::create_object_url_with_blob(&blob).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("No window"))?;
        let document = window.document().ok_or_else(|| JsValue::from_str("No document"))?;
        let a = document.create_element("a").map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        a.set_attribute("href", &url).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        a.set_attribute("download", &format!("{}.vbox", uuid)).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let body = document.body().ok_or_else(|| JsValue::from_str("No body"))?;
        body.append_child(&a).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        a.click();
        body.remove_child(&a).map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        let _ = Url::revoke_object_url(&url).map_err(|e| JsValue::from_str(&format!("{:?}", e)));
        Ok(())
    }

    pub fn import_disk_image(&self) -> Result<Vec<u8>, JsValue> {
        Ok(vec![])
    }

    pub fn export_disk_image(&self, data: &[u8], name: &str) -> Result<(), JsValue> {
        let uuid = Uuid::new_v4();
        self.save_vm_to_file(uuid, data)?;
        Ok(())
    }
}

impl Default for BrowserStorage {
    fn default() -> Self {
        BrowserStorage::new().unwrap()
    }
}
