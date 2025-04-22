use super::frame::{Frame, FrameId, PageReadGuard, PageWriteGuard};
use crate::config::PAGE_SIZE;
use crate::storage::disk::disk_scheduler::DiskScheduler;
use crate::storage::PageId;

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::rc::Rc;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::thread;

/// # Design principles
///
/// - Data locality: a page only stores tuples that are in the same table.
/// - Simplicity: no page directory is neede because page ids represent offsets in the DB file.
pub struct BufferPool<R: Read + Write + Seek> {
    /// The size of the buffer pool in number of frames
    pool_size: usize,
    /// Stores the metadata of the pages in the buffer pool
    /// The buffer pool must guarantee that all entries here are loaded in memory.
    /// TODO: since this vec is readonly, should i delete the rwlock and use inmutable borrows?
    frames: Arc<RwLock<Vec<Rc<RwLock<Frame>>>>>,
    /// Maps page id to buffer pool frame id. Returns None if the page is not in the buffer pool.
    page_table: Arc<RwLock<HashMap<PageId, FrameId>>>,
    /// The list of available frames for allocation. Getting a free frame is O(1).
    free_list: Vec<FrameId>,
    disk_scheduler: DiskScheduler<R>,
}

impl<R: Read + Write + Seek> BufferPool<R> {
    /// Creates a new buffer pool manager with the given size
    pub fn new(pool_size: usize, reader: R) -> Self {
        let mut frames = Vec::with_capacity(pool_size);
        let page_table = HashMap::with_capacity(pool_size);

        //  TODO: log to the console that the database is allocating the buffer pool
        for _ in 0..pool_size {
            let data = vec![0u8; PAGE_SIZE].into_boxed_slice();
            frames.push(Rc::new(RwLock::new(Frame::new(data))));
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

    pub fn get_page_read(&mut self, page_id: PageId) -> PageReadGuard {
        // self.disk_scheduler.schedule_read(page_id);
        // self.disk_scheduler
        //     .schedule_read(page_id, thread::current());

        todo!("copy from get_page_write")
    }

    /// Returns a write (exclusive) guard for a frame, efectively pinning it.
    /// If no free frame is available, it will ask the replacer to evict a frame.
    /// If no frame can be evicted, it will block until a frame is available.
    pub fn get_page_write(&mut self, page_id: PageId) -> PageWriteGuard {
        let frame_id = {
            // TODO: should this remain blocked? or should we release de lock inmediatly
            let page_table = self.page_table.read().expect("page table was poisoned");
            page_table.get(&page_id).cloned()
        };

        match frame_id {
            Some(frame_id) => {
                assert!(
                    frame_id < self.pool_size as FrameId,
                    "Frame id out of bounds",
                );
                let frames = self.frames.read().unwrap();
                let frame = frames.get(frame_id as usize).unwrap();

                let cloned_frame = frame.clone();
                PageWriteGuard::new(cloned_frame)
            }
            None => {
                // TODO: load the page from disk first
                // We initially lock the buffer pool so we can safely pin the frame.
                let free_frame_id: FrameId = self
                    .free_list
                    .pop()
                    .expect("No free frame found. You better work in your eviction algorithm");

                self.load_page_from_disk(page_id, free_frame_id);

                let frames = self.frames.write().unwrap();
                let frame = frames
                    .get(free_frame_id as usize)
                    .expect(format!("Frame id={free_frame_id} out of bounds").as_str());
                let cloned_frame = frame.clone();

                PageWriteGuard::new(cloned_frame)
            }
        }
    }

    fn load_page_from_disk(&mut self, page_id: PageId, frame_id: FrameId) {
        let frames = self.frames.read().unwrap();
        let frame = frames
            .get(frame_id as usize)
            .expect(format!("Frame id={frame_id} out of bounds").as_str());

        let writable_frame = frame.write().unwrap();
        let buffer = writable_frame.data.clone();

        self.disk_scheduler
            .schedule_read(page_id, buffer, thread::current());

        thread::park();

        todo!("Call the scheduler and wait for it to finish")
    }

    // fn get_free_frame(&mut self) -> Rc<RwLockWriteGuard<'_, Frame>> {}

    // pub fn get_frame_mut(&mut self, frame_id: FrameId) -> &mut Frame {

    //     frames
    //         .get_mut(&frame_id)
    //         .expect("Frame not found in buffer pool")
    // }

    pub fn load_free_page(&mut self) {
        // TODO: ask for table kind and look in the catalog
        todo!()
    }

    /// Returns the number of allocated frames in the buffer pool in O(n)
    pub fn len(&self) -> usize {
        self.frames
            .read()
            .unwrap()
            .iter()
            .filter(|f| f.read().unwrap().pin_count > 0)
            .count()
    }

    pub fn free_list(&self) -> &Vec<FrameId> {
        &self.free_list
    }
}
