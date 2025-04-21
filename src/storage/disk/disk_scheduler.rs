use crate::storage::PageId;

pub struct DiskScheduler {}

impl DiskScheduler {
    pub fn new() {
        // TODO: no queue for now, just instantly handle all schedule requests
    }

    pub fn schedule_read(self, page_id: PageId) {}
    pub fn schedule_write(self, page_id: PageId, data: Box<[u8]>) {}
}
