use std::fs;
use std::io::{Read, Seek, Write};
use std::sync::Arc;

use crate::config::BUFFER_POOL_N_FRAMES;
use crate::storage::BufferPool;

pub struct Database {
    /// The filename of the database file. None if the database is in memory.
    filename: Option<String>,
    buffer_pool: Arc<BufferPool>,
}

impl Database {
    pub fn from_buffer<R>(reader: R) -> Self
    where
        R: Read + Write + Seek + Send + 'static,
    {
        Database {
            filename: None,
            buffer_pool: Arc::new(BufferPool::new(BUFFER_POOL_N_FRAMES, reader)),
        }
    }

    // TODO: return Result instead
    pub fn from_file(filename: String) -> Database {
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
            buffer_pool: Arc::new(BufferPool::new(BUFFER_POOL_N_FRAMES, file)),
        }
    }
}

mod test {
    use super::*;
    use crate::config::PAGE_SIZE;
    use std::io::Cursor;
    use std::sync::{atomic::AtomicUsize, Arc};

    const TEST_CONCURRENCY: usize = 24;

    #[test]
    fn test_create_database_from_reader() {
        let database = vec![0u8; PAGE_SIZE];
        let reader = Cursor::new(database);

        let db = Database::from_buffer(reader);
        assert!(db.filename.is_none());
        assert_eq!(db.buffer_pool.len(), 0);
        // assert_eq!(db.buffer_pool.free_list().len(), BUFFER_POOL_N_FRAMES);
    }

    #[test]
    fn test_database_multiple_readers() {
        let data = vec![7u8; PAGE_SIZE];
        let reader = Cursor::new(data);

        let db = Database::from_buffer(reader);
        let mut threads = Vec::with_capacity(TEST_CONCURRENCY);

        let n_bytes_read = Arc::new(AtomicUsize::new(0));

        for _ in 0..TEST_CONCURRENCY {
            let cloned_n_bytes_read = n_bytes_read.clone();
            let cloned_buffer_pool = db.buffer_pool.clone();

            let t = std::thread::spawn(move || {
                let page = cloned_buffer_pool.get_page_read(0);
                let data = &page.read().data;

                assert_eq!(data[0], 7);
                assert_eq!(data.last(), Some(&7));

                let n_bytes = data.len();
                cloned_n_bytes_read.fetch_add(n_bytes, std::sync::atomic::Ordering::SeqCst);
            });
            threads.push(t);
        }

        for t in threads {
            t.join().unwrap();
        }

        assert_eq!(
            n_bytes_read.load(std::sync::atomic::Ordering::Relaxed),
            TEST_CONCURRENCY * PAGE_SIZE
        );
        assert_eq!(db.buffer_pool.len(), 1);
    }

    #[test]
    fn test_database_multiple_writers_and_reader() {
        let data = vec![]; // empty database
        let reader = Cursor::new(data);

        let db = Database::from_buffer(reader);
        let mut threads = Vec::with_capacity(TEST_CONCURRENCY);

        for i in 0..TEST_CONCURRENCY {
            let cloned_buffer_pool = db.buffer_pool.clone();

            let t = std::thread::spawn(move || {
                let page = cloned_buffer_pool.get_page_write(0);
                page.write().data = vec![i as u8; PAGE_SIZE].into();
            });
            threads.push(t);
        }

        for t in threads {
            t.join().unwrap();
        }

        assert_eq!(db.buffer_pool.len(), 1);
        let page = db.buffer_pool.get_page_read(0);
        let data = &page.read().data;
        let first_byte = data[0];
        // the same first byte should be written in all the page
        assert_eq!(data[..], vec![first_byte; PAGE_SIZE]);
    }
}
