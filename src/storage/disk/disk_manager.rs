use std::io::{Read, Seek, Write};

use crate::{config::PAGE_SIZE, storage::PageId};

pub struct DiskManager<R: Read + Write + Seek> {
    reader: R,
}

impl<R: Read + Write + Seek> DiskManager<R> {
    pub fn new(reader: R) -> Self {
        DiskManager { reader }
    }
}

/* Utils */

fn page_id_to_file_offset(id: PageId) -> usize {
    id as usize * PAGE_SIZE
}
