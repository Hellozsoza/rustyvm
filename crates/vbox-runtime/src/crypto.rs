use sha2::{Sha256, Digest};

pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

pub fn generate_random(length: usize) -> Vec<u8> {
    getrandom::getrandom(&mut vec![0u8; length]).unwrap_or(());
    let mut data = vec![0u8; length];
    getrandom::getrandom(&mut data).unwrap_or(());
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256() {
        let hash = sha256(b"hello");
        assert_eq!(hash.len(), 32);
    }
}
