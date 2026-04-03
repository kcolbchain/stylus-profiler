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

pub struct SizeLimitCheck {
    pub uncompressed_size: usize,
    pub estimated_compressed: usize,
    pub uncompressed_ok: bool,
    pub compressed_ok: bool,
    pub headroom_uncompressed: usize,
    pub headroom_compressed: usize,
}
