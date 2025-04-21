use crate::config::{BUFFER_POOL_N_FRAMES, PAGE_SIZE};
use std::collections::HashMap;

type PageId = u16;
type FrameId = u16;

pub struct PageMetadata {
    rf: u32,
    frame: FrameId,
    dirty: bool,
}

pub struct BufferPool {
    page_table: HashMap<PageId, PageMetadata>,
    free_list: Vec<FrameId>,
    pages: Box<[u8]>,
}

impl BufferPool {
    /// Creates a new buffer pool manager with the given size
    pub fn new(pool_size: usize) -> Self {
        let page_table = HashMap::with_capacity(pool_size);
        let pages = vec![0u8; pool_size * PAGE_SIZE].into_boxed_slice();
        let free_list = (0..pool_size as FrameId).collect();

        BufferPool {
            page_table,
            pages,
            free_list,
        }
    }

    /// Returns the number of frames in the buffer pool
    pub fn len(&self) -> usize {
        self.page_table.len()
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        BufferPool::new(BUFFER_POOL_N_FRAMES)
    }
}
