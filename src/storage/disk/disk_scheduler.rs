use std::{
    io::{Read, Seek},
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::storage::{DiskManager, PageId};

enum QueueRequest {
    Read {
        page_id: PageId,
        // callback: Box<dyn FnOnce()>,
    },
    Write {
        page_id: PageId,
        data: Box<[u8]>,
        // callback: Box<dyn FnOnce()>,
    },
}

pub struct DiskScheduler<R: Read + Seek> {
    requests_queue: Arc<Mutex<Vec<QueueRequest>>>,
    disk_manager: DiskManager<R>,
}

impl<R: Read + Seek> DiskScheduler<R> {
    pub fn new(reader: R) -> Self {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let disk_manager = DiskManager::new(reader);

        let moved_queue = queue.clone();
        std::thread::spawn(move || {
            let queue = moved_queue;

            loop {
                let mut queue = queue.lock().unwrap();
                match queue.pop() {
                    Some(_) => todo!(),
                    None => todo!(),
                }
            }
        });

        DiskScheduler {
            disk_manager,
            requests_queue: queue.clone(),
        }
    }

    pub fn schedule_read(&mut self, page_id: PageId) {
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Read { page_id });
    }

    pub fn schedule_write(&mut self, page_id: PageId, data: Box<[u8]>) {
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Write { page_id, data });
    }
}
