use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// The Buffer Pool frame id for internal use only. It is not associated with the page id.
pub type FrameId = u16;

pub struct Frame {
    /// How many threads are accessing this page. A page can only be evicted if pin_count is 0.
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
            data,
        }
    }
}

/// Wrapper for a RwLockReadGuard that decrements the frame pin count
pub struct PageReadGuard {
    frame: Arc<RwLock<Frame>>,
}

impl PageReadGuard {
    pub fn new(frame: Arc<RwLock<Frame>>) -> Self {
        {
            let mut frame = frame.write().unwrap_or_else(PoisonError::into_inner);
            frame.pin_count += 1;
        }
        PageReadGuard { frame }
    }

    pub fn read(&self) -> RwLockReadGuard<'_, Frame> {
        self.frame.read().unwrap()
    }
}

/// Wrapper for a RwLockWriteGuard that decrements the frame pin count
pub struct PageWriteGuard {
    frame: Arc<RwLock<Frame>>,
}

impl PageWriteGuard {
    pub fn new(frame: Arc<RwLock<Frame>>) -> Self {
        {
            let mut frame = frame.write().unwrap_or_else(PoisonError::into_inner);
            frame.pin_count += 1;
            frame.is_dirty = true;
        }
        PageWriteGuard { frame }
    }

    pub fn write(&self) -> RwLockWriteGuard<Frame> {
        self.frame.write().unwrap()
    }
}

impl Drop for PageWriteGuard {
    fn drop(&mut self) {
        let mut frame = self.frame.write().unwrap_or_else(PoisonError::into_inner);
        frame.pin_count -= 1;
    }
}
