use std::fs;
use std::io::{Read, Seek};

use crate::buffer_pool::BufferPool;
use crate::config::BUFFER_POOL_N_FRAMES;

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
    use std::io::Cursor;

    use super::*;

    #[test]
    fn test_database_from_reader() {
        let database = vec![0u8; 4096];
        let reader = Cursor::new(database);

        let db = Database::from_reader(reader);
        assert!(db.filename.is_none());
        assert_eq!(db.buffer_pool.len(), 0);
        assert_eq!(db.buffer_pool.free_list().len(), BUFFER_POOL_N_FRAMES);
    }
}
