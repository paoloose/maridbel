use std::error::Error;

#[derive(Debug)]
pub enum ScheduleError {
    IOError(std::io::Error),
    UnexpectedEof,
    Unknown,
}

#[derive(Debug)]
pub enum BufferPoolError {
    /// The buffer pool is full and cannot accommodate more pages.
    BufferPoolFull,
    /// The requested page is not found in the buffer pool.
    PageNotFound,
    /// The requested page is dirty and cannot be evicted.
    PageDirty,
    /// The requested page is not pinned.
    PageNotPinned,
    /// Derived error from the scheduler
    SchedulerError(ScheduleError),
}

impl std::fmt::Display for BufferPoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BufferPoolError::BufferPoolFull => write!(f, "Buffer pool is full"),
            BufferPoolError::PageNotFound => write!(f, "Page not found in buffer pool"),
            BufferPoolError::PageDirty => write!(f, "Page is dirty and cannot be evicted"),
            BufferPoolError::PageNotPinned => write!(f, "Page is not pinned"),
            BufferPoolError::SchedulerError(schedule_error) => {
                write!(f, "Scheduler error: {:?}", schedule_error)
            }
        }
    }
}

impl std::fmt::Display for ScheduleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScheduleError::IOError(err) => write!(f, "IO error: {}", err),
            ScheduleError::UnexpectedEof => write!(f, "Unexpected EOF"),
            ScheduleError::Unknown => write!(f, "Unknown error"),
        }
    }
}

impl std::convert::From<BufferPoolError> for std::io::Error {
    fn from(err: BufferPoolError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }
}

impl std::convert::From<ScheduleError> for BufferPoolError {
    fn from(err: ScheduleError) -> Self {
        BufferPoolError::SchedulerError(err)
    }
}

impl Error for BufferPoolError {}
impl Error for ScheduleError {}
