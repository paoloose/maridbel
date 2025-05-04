use std::{
    cell::{RefCell, UnsafeCell},
    error::Error,
    fmt::Display,
    sync::{Arc, Condvar, Mutex},
};

pub struct OneshotChannel<T> {
    data: UnsafeCell<Option<T>>,
    sync_pair: Arc<(Mutex<bool>, Condvar)>,
}

pub struct OneshotChannelSender<T> {
    data: UnsafeCell<Option<T>>,
    sync_pair: Arc<(Mutex<bool>, Condvar)>,
}

pub struct OneshotChannelReceiver<T> {
    data: RefCell<Option<T>>,
    sync_pair: Arc<(Mutex<bool>, Condvar)>,
}

impl<T> OneshotChannelSender<T> {
    pub fn send(self, data: T) {
        let (mutex, condvar) = &*self.sync_pair;
        let __ = mutex.lock().unwrap();
        *self.data.borrow_mut() = Some(data);
        condvar.notify_one();
    }
}

impl<T> OneshotChannelReceiver<T> {
    pub fn recv(self) -> Result<T, ReceiveError> {
        let (mutex, condvar) = &*self.sync_pair;
        let mut guard = mutex
            .lock()
            .map_err(|err| ReceiveError::Other(err.to_string()))?;

        while self.data.borrow().is_none() {
            guard = condvar
                .wait(guard)
                .map_err(|err| ReceiveError::Other(err.to_string()))?;
        }
        Ok(self.data.take().expect("msg"))
    }
}

pub fn channel<T>() -> (OneshotChannelSender<T>, OneshotChannelReceiver<T>) {
    let data = UnsafeCell::new(None);

    let sync_pair1 = Arc::new((Mutex::new(false), Condvar::new()));
    let sync_pair2 = sync_pair1.clone();

    (
        OneshotChannelSender {
            data,
            sync_pair: sync_pair1,
        },
        OneshotChannelReceiver {
            data: data,
            sync_pair: sync_pair2,
        },
    )
}

unsafe impl<T> Sync for OneshotChannel<T> {}

impl<T> OneshotChannel<T> {
    pub fn new() -> Self {
        OneshotChannel {
            data: RefCell::new(None),
            sync_pair: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    pub fn send(&self, data: T) {
        let (mutex, condvar) = &*self.sync_pair;
        let __ = mutex.lock().unwrap();
        *self.data.borrow_mut() = Some(data);
        condvar.notify_one();
    }

    pub fn recv(&self) -> Result<T, ReceiveError> {
        let (mutex, condvar) = &*self.sync_pair;
        let mut guard = mutex
            .lock()
            .map_err(|err| ReceiveError::Other(err.to_string()))?;

        while self.data.borrow().is_none() {
            guard = condvar
                .wait(guard)
                .map_err(|err| ReceiveError::Other(err.to_string()))?;
        }
        Ok(self.data.take().expect("msg"))
    }
}

#[derive(Debug)]
pub enum ReceiveError {
    Closed,
    Other(String),
}

impl Error for ReceiveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for ReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiveError::Closed => write!(f, "Channel closed"),
            ReceiveError::Other(msg) => write!(f, "Cannot wait for message: {}", msg),
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test_oneshot_two_threads() {
        let channel = Arc::new(OneshotChannel::<u64>::new());
        let cloned_channel = channel.clone();

        thread::spawn(move || {
            cloned_channel.clone().send(69);
        });

        let data = match channel.recv() {
            Ok(num) => num,
            Err(_) => unreachable!(),
        };
        assert_eq!(data, 69);
    }
}
