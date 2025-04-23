use crate::storage::{DiskManager, PageId};
use std::io::{Read, Seek, Write};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, Thread};
use std::time::Duration;

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

            // TODO: where io_uring will fit here

            loop {
                let maybe_request = {
                    let mut queue = queue.lock().unwrap();
                    queue.pop()
                };

                match maybe_request {
                    Some(QueueRequest::Read {
                        page_id,
                        mut buffer,
                        thread,
                    }) => {
                        let buffer = buffer.as_mut();
                        for i in 0..buffer.len() {
                            buffer[i] = 7_u8;
                        }
                        thread.unpark();
                    }
                    Some(QueueRequest::Write {
                        page_id,
                        data,
                        thread,
                    }) => {
                        println!("writing data");
                        thread.unpark();
                    }
                    None => {
                        // No requests in the queue, sleep for a while
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        });

        DiskScheduler {
            disk_manager,
            requests_queue: queue.clone(),
            handle,
        }
    }

    pub fn schedule_read(&self, page_id: PageId, buffer: Box<[u8]>, thread: Thread) {
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
