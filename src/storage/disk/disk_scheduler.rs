use crate::config::PAGE_SIZE;
use crate::errors::ScheduleError;
use crate::storage::page::THE_EMPTY_PAGE;
use crate::storage::{Frame, PageId};
use oneshot::{OneshotChannelReceiver, OneshotChannelSender};
use std::io::{Read, Seek, SeekFrom, Write};
use std::panic;
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

pub type ScheduleResult = Result<(), ScheduleError>;

enum QueueRequest {
    Read {
        page_id: PageId,
        buffer: Arc<RwLock<Frame>>,
        channel: OneshotChannelSender<ScheduleResult>,
    },
    Write {
        page_id: PageId,
        data: Arc<RwLock<Frame>>,
        channel: OneshotChannelSender<ScheduleResult>,
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
                if let Some(payload) = info.payload().downcast_ref::<&str>() {
                    println!("Payload: {}", payload);
                } else if let Some(payload) = info.payload().downcast_ref::<String>() {
                    println!("Payload: {}", payload);
                } else {
                    println!("Payload: <unknown>");
                }
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
                        channel,
                    }) => {
                        reader
                            .seek(SeekFrom::Start(page_id_to_file_offset(page_id)))
                            .unwrap();

                        println!("reading page_id={page_id} into buffer");

                        let mut buffer = buffer.write().expect("could not lock buffer for reading");
                        match reader.read_exact(&mut buffer.data) {
                            Ok(_) => {
                                // Unwrapped because the caller must not drop the receiver
                                channel.send(Ok(())).unwrap();
                            }
                            // EOF are not errors.W e interpret this as the buffer pool wanting
                            // to read an empty page
                            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                                reader.write_all(&THE_EMPTY_PAGE).unwrap();
                                buffer.data.copy_from_slice(&THE_EMPTY_PAGE);
                                channel.send(Ok(())).unwrap();
                            }
                            Err(e) => {
                                channel.send(Err(ScheduleError::IOError(e))).unwrap();
                            }
                        }
                    }
                    Some(QueueRequest::Write {
                        page_id,
                        data,
                        channel,
                    }) => {
                        println!("writing data {data:?} into page_id={page_id}");
                        channel.send(Ok(())).unwrap();
                        todo!("writing not implemented");
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

    pub fn schedule_read(
        &self,
        page_id: PageId,
        buffer: Arc<RwLock<Frame>>,
    ) -> OneshotChannelReceiver<ScheduleResult> {
        let (tx, rx) = oneshot::channel::<ScheduleResult>();

        if self.handle.is_finished() {
            panic!("Disk scheduler thread has finished");
        }
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Read {
                page_id,
                buffer,
                channel: tx,
            });

        rx
    }

    pub fn schedule_write(
        &self,
        page_id: PageId,
        data: Arc<RwLock<Frame>>,
    ) -> OneshotChannelReceiver<ScheduleResult> {
        let (tx, rx) = oneshot::channel::<ScheduleResult>();

        if self.handle.is_finished() {
            panic!("Disk scheduler thread has finished");
        }
        self.requests_queue
            .lock()
            .unwrap()
            .push(QueueRequest::Write {
                page_id,
                data,
                channel: tx,
            });

        rx
    }
}

/* Utils */

fn page_id_to_file_offset(id: PageId) -> u64 {
    id as u64 * PAGE_SIZE as u64
}
