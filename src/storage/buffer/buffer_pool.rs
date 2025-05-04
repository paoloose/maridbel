use super::eviction::EvictionPolicy;
use super::frame::{Frame, FrameId, PageReadGuard, PageWriteGuard};
use super::lruk_eviction::LRUKEvictionPolicy;
use crate::config::{LRU_K, PAGE_SIZE};
use crate::storage::disk::disk_scheduler::DiskScheduler;
use crate::storage::PageId;

use std::collections::HashMap;
use std::io::{Read, Seek, Write};
use std::sync::{Arc, RwLock};
use std::thread;

/// # Design principles
///
/// - Data locality: a page only stores tuples that are in the same table.
/// - Simplicity: no page directory is neede because page ids represent offsets in the DB file.
pub struct BufferPool {
    /// The size of the buffer pool in number of frames
    pool_size: usize,
    /// Stores the metadata of the pages in the buffer pool
    /// The buffer pool must guarantee that all entries here are loaded in memory.
    frames: Vec<Arc<RwLock<Frame>>>,
    /// Maps page id to buffer pool frame id. Returns None if the page is not in the buffer pool.
    page_table: Arc<RwLock<HashMap<PageId, FrameId>>>,
    /// The list of available frames for allocation. Getting a free frame is O(1).
    free_list: Arc<RwLock<Vec<FrameId>>>,
    /// The disk scheduler that will handle the underlying IO operations. The buffer pool
    /// has no details over how the data is read and written to disk.
    disk_scheduler: DiskScheduler,
    /// The eviction policy to use when the buffer pool is full.
    eviction_policy: Arc<dyn EvictionPolicy + Send + Sync>,
}

impl BufferPool {
    /// Creates a new buffer pool manager with the given size
    pub fn new<R>(pool_size: usize, reader: R) -> Self
    where
        R: Read + Write + Seek + Send + 'static,
    {
        let mut frames = Vec::with_capacity(pool_size);
        let page_table = HashMap::with_capacity(pool_size);

        //  TODO: log to the console that the database is allocating the buffer pool
        for _ in 0..pool_size {
            let data = vec![0u8; PAGE_SIZE].into_boxed_slice();
            frames.push(Arc::new(RwLock::new(Frame::new(data))));
        }

        let free_list = (0..pool_size as FrameId).collect();
        let disk_scheduler = DiskScheduler::new(reader);

        BufferPool {
            pool_size,
            frames,
            free_list: Arc::new(RwLock::new(free_list)),
            page_table: Arc::new(RwLock::new(page_table)),
            eviction_policy: Arc::new(LRUKEvictionPolicy::new(LRU_K, pool_size)),
            disk_scheduler,
        }
    }

    // TODO: This function can fail for the following reasons
    //       - buffer pool is full and there is no frame to evict
    //       - the disk scheduler panicked
    // TODO: Acquiring a full lock over the page_table is a bad design choice. get_page_read
    //       should be possible to be called multiple times at the same time for different page ids
    pub fn get_page_read(&self, page_id: PageId) -> PageReadGuard {
        // We acquire exclusive lock over the page table because we may potentially write to
        // it in the "None" branch
        let mut page_table = self.page_table.write().expect("page table was poisoned");
        let maybe_frame_id = page_table.get(&page_id).cloned();

        match maybe_frame_id {
            Some(frame_id) => {
                // We are not writing to the page table so release the lock inmediatly.
                drop(page_table);

                println!("Found page_id={page_id} in frame_id={frame_id}");
                assert!(
                    frame_id < self.pool_size as FrameId,
                    "Frame id out of bounds",
                );
                let frame = self.frames.get(frame_id as usize).unwrap();
                PageReadGuard::new(frame_id, frame.clone(), self.eviction_policy.clone())
            }
            None => {
                println!("Page id={page_id} not found in buffer pool. Fetching from disk");
                let free_frame_id = self
                    .try_get_free_frane()
                    .expect("Buffer pool is full. No free frame found.");

                page_table.insert(page_id, free_frame_id);

                println!("Found empty frame frame_id={free_frame_id}. Loading page id={page_id}");
                self.load_page_from_disk(page_id, free_frame_id);
                println!("Loaded page id={page_id} into frame_id={free_frame_id}");

                let frame = self
                    .frames
                    .get(free_frame_id as usize)
                    .unwrap_or_else(|| panic!("Frame id={free_frame_id} out of bounds"));
                PageReadGuard::new(free_frame_id, frame.clone(), self.eviction_policy.clone())
            }
        }
    }

    /// Returns a write (exclusive) guard for a frame, efectively pinning it.
    /// If no free frame is available, it will ask the replacer to evict a frame.
    /// If no frame can be evicted, it will block until a frame is available.
    pub fn get_page_write(&self, page_id: PageId) -> PageWriteGuard {
        // We acquire exclusive lock over the page because we may potentially write to
        // the table in the "None" branch
        let mut page_table = self.page_table.write().expect("page table was poisoned");
        let maybe_frame_id = page_table.get(&page_id).cloned();

        match maybe_frame_id {
            Some(frame_id) => {
                // We are not writing to the page table so release the lock inmediatly.
                drop(page_table);
                assert!(
                    frame_id < self.pool_size as FrameId,
                    "Frame id out of bounds",
                );
                let frame = self.frames.get(frame_id as usize).unwrap();
                PageWriteGuard::new(frame_id, frame.clone(), self.eviction_policy.clone())
            }
            None => {
                println!("Page id={page_id} not found in buffer pool. Fetching from disk");
                let free_frame_id = self
                    .try_get_free_frane()
                    .expect("Buffer pool is full. No free frame found.");

                page_table.insert(page_id, free_frame_id);

                self.load_page_from_disk(page_id, free_frame_id);

                let frame = self
                    .frames
                    .get(free_frame_id as usize)
                    .unwrap_or_else(|| panic!("Frame id={free_frame_id} out of bounds"));
                PageWriteGuard::new(free_frame_id, frame.clone(), self.eviction_policy.clone())
            }
        }
    }

    fn load_page_from_disk(&self, page_id: PageId, frame_id: FrameId) {
        let frame = self
            .frames
            .get(frame_id as usize)
            .unwrap_or_else(|| panic!("Frame id={frame_id} out of bounds"));

        self.disk_scheduler
            .schedule_read(page_id, frame.clone(), thread::current());

        println!("Parking thread waiting for page id={page_id} to be read");
        thread::park();

        // TODO: SHOULD BE 7_u8 7_u8 7_u8 7_u8
        println!(
            "🌟 Done. First byte is {}",
            frame.read().unwrap().data.first().unwrap()
        );
        // for byte in frame.read().unwrap().data.iter() {
        //     print!("{byte} ");
        // }
    }

    fn try_get_free_frane(&self) -> Option<FrameId> {
        match self.free_list.write().unwrap().pop() {
            Some(free_frame_id) => Some(free_frame_id),
            _ => self.eviction_policy.evict(),
        }
    }

    pub fn load_free_page(&self) {
        // TODO: ask for table kind and look in the catalog
        todo!()
    }

    /// Returns the number of allocated frames in the buffer pool in O(n)
    pub fn len(&self) -> usize {
        self.page_table.read().unwrap().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
