use std::{
    io::{Read, Seek, Write},
    sync::{Arc, Mutex},
    thread::{spawn, JoinHandle, Thread},
};

use crate::storage::{DiskManager, PageId};

enum QueueRequest {
    Read {
        page_id: PageId,
        buffer: Box<[u8]>,
        thread: Thread,
        // callback: Box<dyn FnOnce()>,
    },
    Write {
        page_id: PageId,
        data: Box<[u8]>,
        thread: Thread,
        // callback: Box<dyn FnOnce()>,
    },
}

pub struct DiskScheduler<R: Read + Write + Seek> {
    requests_queue: Arc<Mutex<Vec<QueueRequest>>>,
    handle: JoinHandle<()>,
    disk_manager: DiskManager<R>,
}

impl<R: Read + Write + Seek> DiskScheduler<R> {
    pub fn new(reader: R) -> Self {
        let queue = Arc::new(Mutex::new(Vec::new()));
        let disk_manager = DiskManager::new(reader);

        let moved_queue = queue.clone();
        let handle = std::thread::spawn(move || {
            let queue = moved_queue;

            // TODO: how to send this read/write result?
            //       start with reads, and writes should be easier
            //       also, start to think where io_uring will fit here

            loop {
                let mut queue = queue.lock().unwrap();
                match queue.pop() {
                    Some(QueueRequest::Read {
                        page_id,
                        buffer,
                        thread,
                    }) => {
                        thread.unpark();
                    }
                    Some(QueueRequest::Write {
                        page_id,
                        data,
                        thread,
                    }) => {
                        thread.unpark();
                    }
                    None => todo!(),
                }
            }
        });

        DiskScheduler {
            disk_manager,
            requests_queue: queue.clone(),
            handle,
        }
    }

    pub fn schedule_read(&mut self, page_id: PageId, buffer: Box<[u8]>, thread: Thread) {
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Read {
                page_id,
                buffer,
                thread,
            });
    }

    pub fn schedule_write(&mut self, page_id: PageId, data: Box<[u8]>, thread: Thread) {
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Write {
                page_id,
                data,
                thread,
            });
    }
}
