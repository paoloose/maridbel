use crate::config::PAGE_SIZE;
use crate::storage::{DiskManager, PageId};
use std::io::{Read, Seek, SeekFrom, Write};
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

pub struct DiskScheduler {
    requests_queue: Arc<Mutex<Vec<QueueRequest>>>,
    handle: JoinHandle<()>,
    // disk_manager: DiskManager<R>,
}

impl DiskScheduler {
    pub fn new<R>(mut reader: R) -> Self
    where
        R: Read + Write + Seek + Send + 'static,
    {
        let queue = Arc::new(Mutex::new(Vec::new()));

        let moved_queue = queue.clone();
        // we make the reader safe to move to the worker thread
        // let mut reader = disk_manager.reader;

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
                        reader.seek(SeekFrom::Start(page_id_to_file_offset(0)));
                        reader.read_exact(buffer.as_mut()).unwrap();
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
            // disk_manager,
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

/* Utils */

fn page_id_to_file_offset(id: PageId) -> u64 {
    id as u64 * PAGE_SIZE as u64
}
