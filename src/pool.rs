use std::future::Future;
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio::sync::mpsc::{Sender, Receiver};
use crate::WaitGroup;
use std::sync::Arc;

/// Asynchronous task pools
pub struct TaskPool<T>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
{
    /// Pool
    pool: Option<Sender<T>>,
}

impl<T> TaskPool<T>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
{
    /// Initialisation Asynchronous task pools
    pub async fn new(size: usize, close_func: T) -> TaskPool<T> {
        /// Control the number of concurrent
        let (limit_sender, limit_receiver): (Sender<bool>, Receiver<bool>) = mpsc::channel(size);

        /// Pool
        let (pool, pool_receiver): (Sender<T>, Receiver<T>) = mpsc::channel(size);

        /// CORE
        task::spawn(Self::core(limit_sender, limit_receiver, pool_receiver, close_func));

        TaskPool {
            pool: Some(pool)
        }
    }

    /// Issuance of specific tasks
    pub async fn send_task(&self, task: T) {
        if let Some(channel) = &self.pool {
            channel.send(task).await;
        }
    }

    /// Missions issued
    pub async fn over(&mut self) {
        let r = self.pool.take();
        if let Some(rr) = r {
            drop(rr);
        }
    }

    /// core
    async fn core(limit_sender: Sender<bool>, limit_receiver: Receiver<bool>, mut pool_receiver: Receiver<T>, close_fn: T) {
        let wg = WaitGroup::new().await;

        let limit_receiver = Arc::new(Mutex::new(limit_receiver));

        'lp:
        loop {
            tokio::select! {
                val = pool_receiver.recv() => {
                    match val {
                        Some(fun) => {
                            fun.await;
                        }
                        None => {
                            break 'lp;
                        }
                    }
                }
            }
        }

        wg.wait().await;
        close_fn.await;
    }
}