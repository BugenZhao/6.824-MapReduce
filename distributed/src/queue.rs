use std::sync::atomic::{AtomicUsize, Ordering};

use tokio::sync::{broadcast, Mutex};

#[derive(Debug)]
pub struct Queue<T: Clone> {
    tx: broadcast::Sender<T>,
    rx: Mutex<broadcast::Receiver<T>>,
    capacity: usize,
    size: AtomicUsize,
}

impl<T: Clone> Queue<T> {
    pub fn new(capacity: usize) -> Self {
        let (tx, rx) = broadcast::channel(capacity);
        Self {
            tx,
            rx: Mutex::new(rx),
            capacity,
            size: AtomicUsize::new(0),
        }
    }

    pub fn push(&self, value: T) -> bool {
        if self.len() >= self.capacity {
            return false;
        }

        match self.tx.send(value) {
            Ok(_) => {
                self.size.fetch_add(1, Ordering::SeqCst);
                true
            }
            Err(_) => false,
        }
    }

    pub async fn pop(&self) -> Option<T> {
        match self.rx.lock().await.try_recv() {
            Ok(value) => {
                self.size.fetch_sub(1, Ordering::SeqCst);
                Some(value)
            }
            Err(e) => match e {
                broadcast::error::TryRecvError::Empty => None,
                e => panic!("{}", e.to_string()),
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        self.size.load(Ordering::Acquire)
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;

    #[tokio::test]
    async fn queue() {
        let queue = Queue::new(2);
        assert_eq!(queue.len(), 0);
        queue.push(0);
        assert_eq!(queue.len(), 1);
        queue.push(1);
        assert_eq!(queue.len(), 2);

        assert_eq!(queue.push(2), false); // failed
        assert_eq!(queue.len(), 2);

        assert_eq!(queue.pop().await, Some(0));
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.pop().await, Some(1));
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop().await, None);
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.pop().await, None);
        assert_eq!(queue.len(), 0);
    }
}
