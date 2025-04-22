use super::frame::{Frame, FrameId};
use crate::config::PAGE_SIZE;
use crate::storage::disk::disk_scheduler::DiskScheduler;
use crate::storage::PageId;

use std::collections::HashMap;
use std::io::{Read, Seek};
use std::sync::{Arc, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// # Design principles
///
/// - Data locality: a page only stores tuples that are in the same table.
/// - Simplicity: no page directory is neede because page ids represent offsets in the DB file.
pub struct BufferPool<R: Read + Seek> {
    pool_size: usize,
    /// Stores the metadata of the pages in the buffer pool
    /// The buffer pool must guarantee that all entries here are loaded in memory.
    frames: Arc<RwLock<HashMap<FrameId, Frame>>>,
    /// Maps page id to buffer pool frame id. Returns None if the page is not in the buffer pool.
    page_table: Arc<RwLock<HashMap<PageId, FrameId>>>,
    free_list: Vec<FrameId>,
    // disk_manager: DiskManager<R>,
    disk_scheduler: DiskScheduler<R>,
}

impl<R: Read + Seek> BufferPool<R> {
    /// Creates a new buffer pool manager with the given size
    pub fn new(pool_size: usize, reader: R) -> Self {
        let mut frames = HashMap::with_capacity(pool_size);
        let page_table = HashMap::with_capacity(pool_size);

        //  TODO: log to the console that the database is allocating the buffer pool
        for i in 0..pool_size {
            let data = vec![0u8; PAGE_SIZE].into_boxed_slice();
            frames.insert(i as FrameId, Frame::new(data));
        }

        let free_list = (0..pool_size as FrameId).collect();
        let disk_scheduler = DiskScheduler::new(reader);

        BufferPool {
            pool_size,
            frames: Arc::new(RwLock::new(frames)),
            free_list,
            page_table: Arc::new(RwLock::new(page_table)),
            disk_scheduler,
        }
    }

    pub fn get_page_read(&mut self, page_id: PageId) -> RwLockReadGuard<'_, Box<[u8]>> {
        self.disk_scheduler.schedule_read(page_id);
        todo!("copy from get_page_write")
    }

    /// Will potentially block if another thread is reading the page
    pub fn get_page_write(&mut self, page_id: PageId) -> RwLockWriteGuard<'_, Box<[u8]>> {
        let page_table = self.page_table.read().expect("page table was poisoned");

        match page_table.get(&page_id) {
            Some(frame_id) => {
                // NOTE: Look for race conditions here
                let frame = self.get_frame_mut(*frame_id);
                assert!(
                    frame.page_metadata.is_some(),
                    "Page with id={} is in the page table but not in the buffer pool",
                    page_id,
                );
                // unwrap is safe because of the assert above
                let metadata = frame.page_metadata.as_mut().unwrap();
                metadata.is_dirty = true;
                metadata.pin_count += 1;
                frame.data.write().unwrap_or_else(PoisonError::into_inner)
            }
            None => {
                let frame = self.load_page_from_disk(page_id);
                frame.data.write().unwrap_or_else(PoisonError::into_inner)
            }
        }
    }

    fn load_page_from_disk(&mut self, page_id: PageId) -> &Frame {
        todo!("Call the scheduler and wait for it to finish")
    }

    pub fn get_frame_mut(&mut self, frame_id: FrameId) -> &mut Frame {
        assert!(
            frame_id < self.pool_size as FrameId,
            "Frame id out of bounds",
        );
        self.frames
            .get_mut(&frame_id)
            .expect("Frame not found in buffer pool")
    }

    pub fn load_free_page(&mut self) {
        // TODO: ask for table kind and look in the catalog
        todo!()
    }

    /// Returns the number of frames in the buffer pool in O(n)
    pub fn len(&self) -> usize {
        self.frames
            .iter()
            .filter(|f| f.1.page_metadata.is_some())
            .count()
    }

    pub fn free_list(&self) -> &Vec<FrameId> {
        &self.free_list
    }
}
