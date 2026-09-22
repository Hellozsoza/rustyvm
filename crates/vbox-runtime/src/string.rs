#[derive(Debug, Clone)]
pub struct RTStr {
    inner: String,
}

impl RTStr {
    pub fn from_str(s: &str) -> Self {
        RTStr { inner: s.to_string() }
    }

    pub fn to_utf16(&self) -> Vec<u16> {
        let mut result = Vec::with_capacity(self.inner.len() + 1);
        for ch in self.inner.encode_utf16() {
            result.push(ch);
        }
        result.push(0);
        result
    }

    pub fn cmp(&self, other: &RTStr) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }

    pub fn icmp(&self, other: &RTStr) -> std::cmp::Ordering {
        self.inner.to_lowercase().cmp(&other.inner.to_lowercase())
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtstr_from_str() {
        let s = RTStr::from_str("hello");
        assert_eq!(s.len(), 5);
    }

    #[test]
    fn test_rtstr_cmp() {
        let a = RTStr::from_str("abc");
        let b = RTStr::from_str("xyz");
        assert_eq!(a.cmp(&b), std::cmp::Ordering::Less);
    }

    #[test]
    fn test_rtstr_to_utf16() {
        let s = RTStr::from_str("Hi");
        let utf16 = s.to_utf16();
        assert_eq!(utf16, vec![72, 105, 0]);
    }
}
