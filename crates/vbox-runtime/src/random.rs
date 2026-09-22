pub fn random_u32() -> u32 {
    let mut buf = [0u8; 4];
    getrandom::getrandom(&mut buf).unwrap_or(());
    u32::from_le_bytes(buf)
}

pub fn random_u64() -> u64 {
    let mut buf = [0u8; 8];
    getrandom::getrandom(&mut buf).unwrap_or(());
    u64::from_le_bytes(buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_u32() {
        let a = random_u32();
        let b = random_u32();
        let _ = (a, b);
    }
}
