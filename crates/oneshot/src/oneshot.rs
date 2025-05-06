use std::cell::UnsafeCell;
use std::sync::{Arc, Condvar, Mutex};

use crate::errors::SendError;
use crate::ReceiveError;

pub struct OneshotChannelSender<T> {
    data: Arc<UnsafeCell<Option<T>>>,
    sync_pair: Arc<(Mutex<bool>, Condvar)>,
}

pub struct OneshotChannelReceiver<T> {
    data: Arc<UnsafeCell<Option<T>>>,
    sync_pair: Arc<(Mutex<bool>, Condvar)>,
}

// SAFETY: UnsafeCell<Option<T>> is not safe to send to another thread,
// but we guarantee safety by synchronizing access to the data using the sync_pair condvar.
unsafe impl<T> Send for OneshotChannelSender<T> {}
unsafe impl<T> Send for OneshotChannelReceiver<T> {}

impl<T> OneshotChannelSender<T> {
    /// Sends the oneshot data to the receiver.
    /// It never blocks. Always returns inmediatly.
    ///
    /// The sender is consumed so the resources are released
    /// and no other thread can send data.
    pub fn send(self, data: T) -> Result<(), SendError> {
        let (mutex, condvar) = &*self.sync_pair;
        let __ = mutex.lock().unwrap();

        match Arc::try_unwrap(self.data) {
            Ok(_) => Err(SendError::Closed),
            Err(shared_data) => {
                // SAFETY: when this block is reached, we have exclusive access
                // over the shared mutex.
                unsafe {
                    *shared_data.get() = Some(data);
                }
                condvar.notify_one();

                Ok(())
            }
        }
    }
}

impl<T> OneshotChannelReceiver<T> {
    /// Blocks until ther is data available.
    /// The data is made available by the sender when send is called.
    pub fn recv(self) -> Result<T, ReceiveError> {
        let (mutex, condvar) = &*self.sync_pair;
        let mut guard = mutex
            .lock()
            .map_err(|err| ReceiveError::Other(err.to_string()))?;

        match Arc::try_unwrap(self.data) {
            Ok(_) => Err(ReceiveError::Closed),
            Err(shared_data) => {
                // SAFETY: when this block is reached, we have exclusive access
                // over the shared mutex.
                unsafe {
                    let data = &mut *shared_data.get();

                    while data.is_none() {
                        guard = condvar
                            .wait(guard)
                            .map_err(|err| ReceiveError::Other(err.to_string()))?;
                    }
                    Ok(data.take().expect("msg"))
                }
            }
        }
    }
}

/// Creates a oneshot channel. The channel is composed of a sender and a receiver.
/// Both the sender and receiver become invalid after the first send/receive.
///
/// ## Example
///
/// ```
/// use std::thread;
/// use oneshot::channel;
///
/// let (tx, rx) = channel::<u64>();
///
/// thread::spawn(move || {
///     tx.send(69).unwrap();
/// });
///
/// let data = match rx.recv() {
///     Ok(num) => num,
///     Err(_) => unreachable!(),
/// };
///
/// assert_eq!(data, 69);
/// ```

pub fn channel<T>() -> (OneshotChannelSender<T>, OneshotChannelReceiver<T>) {
    let data1 = Arc::new(UnsafeCell::new(None));
    let data2 = data1.clone();

    let sync_pair1 = Arc::new((Mutex::new(false), Condvar::new()));
    let sync_pair2 = sync_pair1.clone();

    (
        OneshotChannelSender {
            data: data1,
            sync_pair: sync_pair1,
        },
        OneshotChannelReceiver {
            data: data2,
            sync_pair: sync_pair2,
        },
    )
}

#[cfg(test)]
mod test {
    use std::thread;

    use super::*;

    #[test]
    fn test_oneshot_send_in_other_thread() {
        let (tx, rx) = channel::<u64>();

        thread::spawn(move || {
            tx.send(69).unwrap();
        });

        let data = match rx.recv() {
            Ok(num) => num,
            Err(_) => unreachable!(),
        };

        assert_eq!(data, 69);
    }

    #[test]
    fn test_oneshot_receive_in_other_thread() {
        let (tx, rx) = channel::<u64>();

        thread::spawn(move || {
            let data = match rx.recv() {
                Ok(num) => num,
                Err(_) => unreachable!(),
            };

            assert_eq!(data, 69);
        });

        tx.send(69).unwrap();
    }

    #[test]
    fn test_oneshot_handle_receiver_drop() {
        let (tx, rx) = channel::<u64>();

        drop(rx);
        assert_eq!(tx.send(69).unwrap_err(), SendError::Closed);
    }

    #[test]
    fn test_oneshot_handle_sender_drop() {
        let (tx, rx) = channel::<u64>();

        drop(tx);
        assert_eq!(rx.recv().unwrap_err(), ReceiveError::Closed);
    }

    #[test]
    fn test_oneshot_handle_sender_and_receiver_drop() {
        let (tx, rx) = channel::<u64>();
        drop(tx);
        drop(rx);
    }
}
