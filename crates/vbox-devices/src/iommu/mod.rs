/// IOMMU (Input/Output Memory Management Unit) emulation.
pub struct IommuDevice {
    /// Number of IOMMU domains.
    pub domains: u32,
    /// IOMMU base address.
    pub base_address: u64,
    /// Whether IOMMU is enabled.
    pub enabled: bool,
}

impl IommuDevice {
    /// Creates a new IOMMU device.
    pub fn new() -> Self {
        Self {
            domains: 1,
            base_address: 0xFED00000,
            enabled: false,
        }
    }

    /// Enables the IOMMU.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disables the IOMMU.
    pub fn disable(&mut self) {
        self.enabled = false;
    }
}

impl Default for IommuDevice {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iommu_new() {
        let iommu = IommuDevice::new();
        assert!(!iommu.enabled);
        assert_eq!(iommu.domains, 1);
    }
}
