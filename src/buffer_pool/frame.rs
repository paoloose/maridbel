use std::sync::RwLock;

/// The Buffer Pool frame id for internal use only. It is not associated with the page id.
pub type FrameId = u16;

pub struct PageMetadata {
    /// How many threads are accessing this page. A page can only be evicted if pin_count is 0.
    pub pin_count: u32,
    pub is_dirty: bool,
}

// pub struct Page {
//     /// If a page is loaded, it will contain the page metadata
//     metadata: PageMetadata,
//     /// Heap allocated frame of size PAGE_SIZE
//     data: RwLock<Box<[u8]>>,
// }

// impl Page {
//     fn new(data: RwLock<Box<[u8]>>, metadata: PageMetadata) -> Self {
//         Page { metadata, data }
//     }
// }

pub struct Frame {
    /// If a page is loaded, it will contain the Page metadata
    pub page_metadata: Option<PageMetadata>,
    /// Heap allocated frame of size PAGE_SIZE.
    /// It is only guaranteed to contain valid page data if page_metadata is Some.
    pub data: RwLock<Box<[u8]>>,
}

impl Frame {
    pub fn new(data: Box<[u8]>) -> Self {
        Frame {
            page_metadata: None,
            data: RwLock::new(data),
        }
    }
}
