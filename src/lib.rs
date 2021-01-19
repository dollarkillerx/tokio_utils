mod wait_group;
mod pool;

// pub use
pub use wait_group::WaitGroup;
pub use pool::TaskPool;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    use super::*;
    use tokio_test;
    use tokio::task;

    #[test]
    fn async_wait_group() {
        tokio_test::block_on(async {
            let wg = WaitGroup::new().await;

            for i in 0..10 {
                let wg = wg.clone();
                wg.add(1).await;
                task::spawn(async move {
                    println!("Hello World {} ", i);
                    wg.done().await.unwrap();
                });
            }

            wg.wait().await;
        })
    }

    #[test]
    fn pool() {
        tokio_test::block_on(async {
            // let mut pool = TaskPool::new(async move {
            //     println!("Hello World");
            // }).await;
            //
            // pool.hello_world().await;
            //
            // pool.run().await;

            // let r = TaskPool::new(100, async move {
            //     println!("Over");
            // }).await;
        })
    }

    use tokio::sync::mpsc;
    use tokio::sync::Mutex;
    use std::sync::Arc;
    use tokio::sync::mpsc::{Sender, Receiver};

    #[test]
    fn test_ch() {
        tokio_test::block_on(async {
            let (sen, rx): (Sender<usize>, Receiver<usize>) = mpsc::channel(10);

            let rxc = Arc::new(Mutex::new(rx));
            let rxc1 = rxc.clone();
            // let r = rxc1.lock().await.recv().await;
            // println!("{:#?}",r);
            tokio::spawn(async move {
                'a:
                loop {
                    let mut r = rxc1.lock().await;
                    tokio::select! {
                        val = r.recv() => {
                           if let Some(data) = val {
                                println!("r: {}",data);
                                continue;
                           }
                           break 'a;
                        }
                    }
                }
            });

            sen.send(1).await.unwrap();
            // s.closed().await;
            // drop(s);
            sen.send(2).await.unwrap();

            drop(sen);

            println!("rx is {:#?}", rxc.lock().await.recv().await);
            println!("rx is {:#?}", rxc.lock().await.recv().await);
            println!("rx is {:#?}", rxc.lock().await.recv().await);  // is None
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });
    }

    use tokio::sync::oneshot;

    #[test]
    fn test_ch2() {
        tokio_test::block_on(async {
            let (tx1, rx1) = oneshot::channel();
            let (tx2, rx2) = oneshot::channel();

            tokio::spawn(async {
                let _ = tx1.send("one");
            });

            tokio::spawn(async {
                let _ = tx2.send("two");
            });

            tokio::select! {
                val = rx1 => {
                    println!("rx1 completed first with {:?}", val);
                }
                val = rx2 => {
                    println!("rx2 completed first with {:?}", val);
                }
            }
        })
    }

    #[test]
    fn test_ch3() {
        tokio_test::block_on(async {
            let r = TaskPool::new(10, async {
                println!("hello world 1314");
            }).await;

            r.send_task(async {
                println!("hello world 1314");
            }).await;

            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        })
    }
}
