use std::io::{Read, Seek, Write};

enum DataSource {
    Memory,
    File(String),
}

pub struct DiskManager<R: Read + Write + Seek> {
    reader: R,
}

impl<R: Read + Write + Seek> DiskManager<R> {
    pub fn new(reader: R) -> Self {
        DiskManager { reader }
    }
}
