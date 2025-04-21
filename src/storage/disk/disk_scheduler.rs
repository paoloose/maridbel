use std::{
    sync::{Arc, Mutex},
    thread::spawn,
};

use crate::storage::PageId;

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

pub struct DiskScheduler {
    requests_queue: Arc<Mutex<Vec<QueueRequest>>>,
}

impl DiskScheduler {
    pub fn new() -> Self {
        let queue = Arc::new(Mutex::new(Vec::new()));

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
