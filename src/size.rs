/// Stylus contract size limits (as of ArbOS 32)
pub const MAX_UNCOMPRESSED_WASM: usize = 128 * 1024;  // 128 KB
pub const MAX_COMPRESSED_WASM: usize = 24 * 1024;      // 24 KB (brotli compressed)

pub fn check_limits(file_size: usize) -> SizeLimitCheck {
    let uncompressed_ok = file_size <= MAX_UNCOMPRESSED_WASM;
    // Rough compressed estimate: brotli typically achieves 3-5x compression on WASM
    let estimated_compressed = file_size / 4; // conservative 4x estimate
    let compressed_ok = estimated_compressed <= MAX_COMPRESSED_WASM;

    SizeLimitCheck {
        uncompressed_size: file_size,
        estimated_compressed: estimated_compressed,
        uncompressed_ok,
        compressed_ok,
        headroom_uncompressed: if uncompressed_ok {
            MAX_UNCOMPRESSED_WASM - file_size
        } else {
            0
        },
        headroom_compressed: if compressed_ok {
            MAX_COMPRESSED_WASM - estimated_compressed
        } else {
            0
        },
    }
}

#[allow(dead_code)]
pub struct SizeLimitCheck {
    pub uncompressed_size: usize,
    pub estimated_compressed: usize,
    pub uncompressed_ok: bool,
    pub compressed_ok: bool,
    pub headroom_uncompressed: usize,
    pub headroom_compressed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_contract() {
        let check = check_limits(10 * 1024); // 10KB
        assert!(check.uncompressed_ok);
        assert!(check.compressed_ok);
    }

    #[test]
    fn test_oversized_uncompressed_contract() {
        let check = check_limits(130 * 1024); // 130KB (>128KB)
        assert!(!check.uncompressed_ok);
        // It might be compressed OK depending on the 4x estimate, but uncompressed fails
    }

    #[test]
    fn test_oversized_compressed_contract() {
        // If file is 100KB, estimated compressed is 25KB (>24KB)
        let check = check_limits(100 * 1024);
        assert!(check.uncompressed_ok);
        assert!(!check.compressed_ok);
    }

    #[test]
    fn flags_contracts_over_compressed_limit() {
        let oversized = (MAX_COMPRESSED_WASM * 4) + 4;

        let check = check_limits(oversized);

        assert_eq!(check.uncompressed_size, oversized);
        assert!(check.uncompressed_ok);
        assert_eq!(check.estimated_compressed, MAX_COMPRESSED_WASM + 1);
        assert!(!check.compressed_ok);
        assert_eq!(check.headroom_compressed, 0);
    }

    #[test]
    fn reports_headroom_for_small_contracts() {
        let check = check_limits(1024);

        assert!(check.uncompressed_ok);
        assert!(check.compressed_ok);
        assert_eq!(check.headroom_uncompressed, MAX_UNCOMPRESSED_WASM - 1024);
        assert_eq!(check.headroom_compressed, MAX_COMPRESSED_WASM - 256);
    }
}
