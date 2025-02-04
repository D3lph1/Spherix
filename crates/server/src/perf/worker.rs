use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use flume::Receiver;
use rayon::{ThreadPool, ThreadPoolBuilder};

/// Task handler with statically-dispatched tasks.
pub trait StaticTaskHandle<T, S> {
    fn handle(&self, task: T, non_shared_state: S);
}

/// Task handler with dynamically-dispatched tasks. Much more convenient for usage
/// than [`StaticTaskHandle`], but with an additional indirection overhead.
pub trait DynamicTaskHandler {
    fn handle(&self, task: Box<dyn Any>);
}

pub struct DynamicTaskHandlerDelegate(pub HashMap<TypeId, Box<dyn DynamicTaskHandler + Send + Sync>>);

impl DynamicTaskHandler for DynamicTaskHandlerDelegate {
    fn handle(&self, task: Box<dyn Any>) {
        let type_id = (*task).type_id();

        let h = self.0.get(&type_id).unwrap();
        h.handle(task);
    }
}

pub struct StaticWorker<T, H, S> {
    pool: ThreadPool,
    handler: Arc<H>,
    thread_local_state: S,
    task_rx: Receiver<T>,
}

impl<T, H, S> StaticWorker<T, H, S>
    where
        T: Send + 'static,
        H: StaticTaskHandle<T, S> + Send + Sync + 'static,
        S: Send + Clone + 'static
{
    const TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new(
        handler: H,
        thread_local_state: S,
        task_rx: Receiver<T>,
        num_threads: usize
    ) -> Self {
        Self {
            pool: ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            thread_local_state,
            handler: Arc::new(handler),
            task_rx
        }
    }

    pub fn run(mut self) {
        loop {
            match self.task_rx.recv_timeout(Self::TIMEOUT) {
                Ok(task ) => {
                    let handler = self.handler.clone();
                    let thread_local_state = self.thread_local_state.clone();
                    self.pool.spawn(move || {
                        handler.handle(task, thread_local_state);
                    })
                },
                Err(_) => {}
            }
        }
    }
}

pub struct DynamicWorker<H: DynamicTaskHandler + Sync + 'static> {
    pool: ThreadPool,
    handler: Arc<H>,
    task_rx: Receiver<Box<dyn Any + Send>>,
}

impl<H> DynamicWorker<H>
    where H: DynamicTaskHandler + Send + Sync + 'static
{
    const TIMEOUT: Duration = Duration::from_secs(10);

    pub fn new(
        handler: H,
        task_rx: Receiver<Box<dyn Any + Send>>,
        num_threads: usize
    ) -> Self {
        Self {
            pool: ThreadPoolBuilder::new()
                .num_threads(num_threads)
                .build()
                .unwrap(),
            handler: Arc::new(handler),
            task_rx
        }
    }

    pub fn run(mut self) {
        loop {
            match self.task_rx.recv_timeout(Self::TIMEOUT) {
                Ok(task ) => {
                    let handler = self.handler.clone();
                    self.pool.spawn(move || {
                        handler.handle(task);
                    })
                },
                Err(_) => {}
            }
        }
    }
}

/// Unsafe wrapper for [`T`] that forcely implement [`Send`] trait.
/// Can be used to move a structure that does not implement [`Send`] trait to another
/// thread (provided that there is a guarantee that this structure will only be used
/// by the new thread (so, be a thread-local) and that there are no thread-unsafe
/// references to this structure left on other threads).
#[derive(Clone)]
pub struct ForceSend<T>(T);

/// All methods of the wrapper structure are marked as unsafe in order to draw attention
/// to the nature of the wrapper in a code that calls them.
impl<T> ForceSend<T> {
    #[inline]
    pub unsafe fn new(val: T) -> Self {
        Self(val)
    }

    /// Just move the inner value out of the wrapper.
    #[inline]
    pub unsafe fn into_inner(self) -> T {
        self.0
    }
}

unsafe impl<T> Send for ForceSend<T> {}
