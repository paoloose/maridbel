use std::io::{Read, Seek};

use crate::{config::PAGE_SIZE, storage::PageId};

pub struct DiskManager<R: Read + Seek> {
    reader: R,
}

impl<R: Read + Seek> DiskManager<R> {
    pub fn new(reader: R) -> Self {
        DiskManager { reader }
    }
}

/* Utils */

fn page_id_to_file_offset(id: PageId) -> usize {
    id as usize * PAGE_SIZE
}
