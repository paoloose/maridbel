use std::{
    rc::Rc,
    sync::{PoisonError, RwLock, RwLockWriteGuard},
};

/// The Buffer Pool frame id for internal use only. It is not associated with the page id.
pub type FrameId = u16;

pub struct PageMetadata {
    /// How many threads are accessing this page. A page can only be evicted if pin_count is 0.
    pub pin_count: u32,
    pub is_dirty: bool,
}

pub struct Frame {
    /// If a page is loaded, it will contain the Page metadata
    pub pin_count: u32,
    pub is_dirty: bool,
    /// Heap allocated frame of size PAGE_SIZE.
    /// It is only guaranteed to contain valid page data if page_metadata is Some.
    pub data: Box<[u8]>,
}

impl Frame {
    pub fn new(data: Box<[u8]>) -> Self {
        Frame {
            pin_count: 0,
            is_dirty: false,
            data: data,
        }
    }
}

/// Wrapper for a RwLockReadGuard that decrements the frame pin count
pub struct PageReadGuard<'a> {
    frame: &'a Frame,
}

/// Wrapper for a RwLockWriteGuard that decrements the frame pin count
pub struct PageWriteGuard {
    frame: Rc<RwLock<Frame>>,
}

impl PageWriteGuard {
    pub fn new(frame: Rc<RwLock<Frame>>) -> Self {
        {
            let mut frame = frame.write().unwrap_or_else(PoisonError::into_inner);
            frame.pin_count += 1;
            frame.is_dirty = true;
        }
        PageWriteGuard { frame }
    }

    pub fn write_guard(&self) -> RwLockWriteGuard<Frame> {
        self.frame.write().unwrap()
    }
}

impl Drop for PageWriteGuard {
    fn drop(&mut self) {
        let mut frame = self.frame.write().unwrap_or_else(PoisonError::into_inner);
        frame.pin_count -= 1;
    }
}
