use vbox_core::VBoxError;

#[derive(Debug)]
pub struct RTSocket {
    url: String,
    connected: bool,
}

impl RTSocket {
    pub fn connect(url: &str) -> Result<Self, VBoxError> {
        Ok(RTSocket { url: url.to_string(), connected: true })
    }

    pub fn send(&mut self, data: &[u8]) -> Result<usize, VBoxError> {
        Ok(data.len())
    }

    pub fn recv(&mut self, buf: &mut [u8]) -> Result<usize, VBoxError> {
        Ok(buf.len())
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}
