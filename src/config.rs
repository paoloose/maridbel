use crate::macros::static_assert;

/// The size (in bytes) of a page in the buffer pool
pub const PAGE_SIZE: usize = 4096;

/// The number of frames in the buffer pool.
/// The more frames, the more pages we can cache in memory. Increasing this value
/// will generally improve performance, but will also increase memory usage.
pub const BUFFER_POOL_N_FRAMES: usize = 69;

static_assert!(PAGE_SIZE % 8 == 0);
