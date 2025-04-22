use std::fs;
use std::io::{Read, Seek};

use crate::config::BUFFER_POOL_N_FRAMES;
use crate::storage::BufferPool;

pub struct Database<R: Read + Seek> {
    /// The filename of the database file. None if the database is in memory.
    filename: Option<String>,
    buffer_pool: BufferPool<R>,
}

impl<R: Read + Seek> Database<R> {
    pub fn from_reader(reader: R) -> Self {
        Database {
            filename: None,
            buffer_pool: BufferPool::new(BUFFER_POOL_N_FRAMES, reader),
        }
    }

    // TODO: return Result instead
    pub fn from_file(filename: String) -> Database<fs::File> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&filename)
            .unwrap_or_else(|_| {
                panic!("Failed to open database file: {:?}", filename);
            });

        Database {
            filename: Some(filename),
            buffer_pool: BufferPool::new(BUFFER_POOL_N_FRAMES, file),
        }
    }
}

mod test {
    const TEST_CONCURRENCY: usize = 16;
    use super::*;
    use std::{io::Cursor, sync::atomic::AtomicUsize};

    #[test]
    fn test_create_database_from_reader() {
        let database = vec![0u8; 4096];
        let reader = Cursor::new(database);

        let db = Database::from_reader(reader);
        assert!(db.filename.is_none());
        assert_eq!(db.buffer_pool.len(), 0);
        assert_eq!(db.buffer_pool.free_list().len(), BUFFER_POOL_N_FRAMES);
    }

    #[test]
    fn test_database_multiple_readers() {
        let database = vec![0u8; 4096];
        let reader = Cursor::new(database);

        let mut db = Database::from_reader(reader);

        let mut threads = Vec::with_capacity(TEST_CONCURRENCY);

        let n_bytes_read = AtomicUsize::new(0);

        for _ in 0..TEST_CONCURRENCY {
            let t = std::thread::spawn(|| {
                // RwLockReadGuard<'_, Box<[u8]>>
                let page = db.buffer_pool.get_page_read(0);
                let n_bytes = page.len();
                n_bytes_read.fetch_add(n_bytes, std::sync::atomic::Ordering::SeqCst);
            });
            threads.push(t);
        }

        for t in threads {
            t.join();
        }
    }
}
