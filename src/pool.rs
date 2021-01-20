use super::*;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::{mpsc, Mutex};
use tokio::task;
use tokio::sync::mpsc::{Sender, Receiver};
use std::sync::Arc;


/// Asynchronous task pools
pub struct TaskPool
{
    /// Pool
    pool: Option<Sender<Pin<Box<dyn Future<Output=()>>>>>,
}

impl TaskPool
{
    /// Initialisation Asynchronous task pools
    pub async fn new(size: usize, close_func: Pin<Box<dyn Future<Output=()>>>) -> TaskPool {
        // Control the number of concurrent
        let (limit_sender, limit_receiver): (Sender<bool>, Receiver<bool>) = mpsc::channel(size);

        // Pool
        let (pool, pool_receiver): (Sender<Pin<Box<dyn Future<Output=()>>>>, Receiver<Pin<Box<dyn Future<Output=()>>>>) = mpsc::channel(size);

        // CORE
        task::spawn(Self::core(limit_sender, limit_receiver, pool_receiver, close_func));

        TaskPool {
            pool: Some(pool)
        }
    }

    /// Issuance of specific tasks
    pub async fn send_task(&self, task: Pin<Box<dyn Future<Output=()>>>) {
        if let Some(channel) = &self.pool {
            channel.send(task).await.unwrap_err();
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
    async fn core(limit_sender: Sender<bool>, limit_receiver: Receiver<bool>, mut pool_receiver: Receiver<Pin<Box<dyn Future<Output=()>>>>, close_fn: Pin<Box<dyn Future<Output=()>>>) {
        let wg = WaitGroup::new().await;

        // Restricted flow
        let limit_receiver = Arc::new(Mutex::new(limit_receiver));

        'lp:
        loop {
            tokio::select! {
                val = pool_receiver.recv() => {
                    match val {
                        Some(fun) => {
                            wg.add(1).await;
                            limit_sender.send(true).await.unwrap();

                            let limit_receiver = limit_receiver.clone();
                            let tg = wg.clone();
                            task::spawn(async move {
                                fun.await;

                                limit_receiver.lock().await.recv().await;
                                tg.done().await.unwrap();
                            });
                            // future created by async block is not `Send`
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