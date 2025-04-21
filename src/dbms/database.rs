use std::fs;
use std::io::{Read, Seek};

use crate::buffer_pool::BufferPool;

pub struct Database<R: Read + Seek> {
    /// The filename of the database file. None if the database is in memory.
    filename: Option<String>,
    buffer_pool: BufferPool,
    reader: R,
}

impl<R: Read + Seek> Database<R> {
    pub fn from_reader(reader: R) -> Self {
        Database {
            filename: None,
            buffer_pool: BufferPool::default(),
            reader,
        }
    }

    // TODO: return Result instead
    pub fn from_file<S: Into<String>>(filename: S) -> Database<fs::File> {
        let filename = filename.into();

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
            buffer_pool: BufferPool::default(),
            reader: file,
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
        assert_eq!(db.reader.position(), 0);
    }
}
