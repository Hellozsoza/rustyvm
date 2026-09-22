#[derive(Debug, Clone)]
pub struct RTUuid(uuid::Uuid);

impl RTUuid {
    pub fn new_v4() -> Self {
        RTUuid(uuid::Uuid::new_v4())
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        self.0.as_bytes().clone()
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        RTUuid(uuid::Uuid::from_bytes(bytes))
    }

    pub fn as_uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtuuid_new_v4() {
        let u = RTUuid::new_v4();
        assert_eq!(u.as_bytes().len(), 16);
    }

    #[test]
    fn test_rtuuid_roundtrip() {
        let u1 = RTUuid::new_v4();
        let bytes = u1.as_bytes();
        let u2 = RTUuid::from_bytes(bytes);
        assert_eq!(u1.as_bytes(), u2.as_bytes());
    }
}
