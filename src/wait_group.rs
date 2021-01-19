use tokio::sync::Mutex;
use tokio::time;
use std::sync::Arc;
use std::cell::Cell;
use super::*;

/// A WaitGroup waits for a collection of routines to finish.
///
/// The main routine calls Add to set the number of routines to wait for.
/// Then each of the routines runs and calls Done when finished. At the same time,
/// Wait can be used to block until all routines have finished.
/// A WaitGroup must not be copied after first use.
///
/// # Examples
/// ```
/// let wg = WaitGroup::new().await;
///
/// for i in 0..10 {
/// let wg = wg.clone();
///   wg.add(1).await;
///   task::spawn(async move {
///   println!("Hello World {} ",i);
///   wg.done().await.unwrap();
///   });
/// }
///
/// wg.wait().await;
/// ```
pub struct WaitGroup {
    task_number: Mutex<Cell<u64>>
}

impl WaitGroup {
    pub async fn new() -> Arc<WaitGroup> {
        Arc::new(WaitGroup {
            task_number: Mutex::new(Cell::new(0))
        })
    }

    /// The main routine calls Add to set the number of routines to wait for.
    pub async fn add(&self, delta: u64) {
        let task_number = self.task_number.lock().await;
        task_number.set(task_number.get() + delta);
    }

    /// Then each of the routines runs and calls Done when finished. At the same time,
    pub async fn done(&self) -> Result<()> {
        let task_number = self.task_number.lock().await;
        if task_number.get() <= 0 {
            return Err("No running tasks".into());
        }
        task_number.set(task_number.get() - 1);

        Ok(())
    }

    /// Wait can be used to block until all routines have finished.
    pub async fn wait(&self) {
        loop {
            // 又没有更好的解决办法? select ?
            time::sleep(time::Duration::from_millis(50)).await;

            let task_num = self.task_number.lock().await;
            if task_num.get() == 0 {
                break;
            }
        }
    }
}

