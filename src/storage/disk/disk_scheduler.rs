use crate::config::PAGE_SIZE;
use crate::storage::page::THE_EMPTY_PAGE;
use crate::storage::{Frame, PageId};
use std::io::{Read, Seek, SeekFrom, Write};
use std::panic;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::{JoinHandle, Thread};
use std::time::Duration;

enum QueueRequest {
    Read {
        page_id: PageId,
        buffer: Arc<RwLock<Frame>>,
        thread: Thread,
        // callback: Box<dyn FnOnce()>,
    },
    Write {
        page_id: PageId,
        data: Arc<RwLock<Frame>>,
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

        let handle = std::thread::spawn(move || {
            let hook = panic::take_hook();
            panic::set_hook(Box::new(move |info| {
                hook(info);
                println!("Disk scheduler thread panicked: {:#?}", info);
                panic!();
            }));

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
                        buffer,
                        thread,
                    }) => {
                        reader
                            .seek(SeekFrom::Start(page_id_to_file_offset(page_id)))
                            .unwrap();

                        let mut buffer = buffer.write().expect("could not lock buffer for reading");
                        match reader.read_exact(&mut buffer.data) {
                            Ok(_) => {
                                thread.unpark();
                            }
                            // catch eof
                            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                                reader.write_all(&THE_EMPTY_PAGE).unwrap();
                                buffer.data.copy_from_slice(&THE_EMPTY_PAGE);
                                thread.unpark();
                            }
                            Err(e) => {
                                println!("Error reading from disk: {e}");
                            }
                        }
                    }
                    Some(QueueRequest::Write {
                        page_id,
                        data,
                        thread: _,
                    }) => {
                        println!("writing data {data:?} into page_id={page_id}");
                        todo!("writing not implemented");
                        // thread.unpark();
                    }
                    None => {
                        // No requests in the queue, sleep for a while
                        std::thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        });

        DiskScheduler {
            // disk_manager, // TODO: move this manager here, or go without it
            requests_queue: queue.clone(),
            handle,
        }
    }

    pub fn schedule_read(&self, page_id: PageId, buffer: Arc<RwLock<Frame>>, thread: Thread) {
        if self.handle.is_finished() {
            panic!("Disk scheduler thread has finished");
        }
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Read {
                page_id,
                buffer,
                thread,
            });
    }

    pub fn schedule_write(&self, page_id: PageId, data: Arc<RwLock<Frame>>, thread: Thread) {
        if self.handle.is_finished() {
            panic!("Disk scheduler thread has finished");
        }
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
