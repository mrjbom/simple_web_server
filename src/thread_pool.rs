use std::thread;
use std::{sync, sync::atomic};

pub struct ThreadPool {
    threads_handlers: Vec<thread::JoinHandle<()>>,
    threads_number: u8,
    active_jobs_counter: sync::Arc<atomic::AtomicU8>,
}

type Job = Box<dyn FnOnce() -> () + 'static>;

impl ThreadPool {
    /// Creates a ThreadPool and starts threads_number of threads ready for Jobs.
    pub fn new(threads_number: u8) -> Self {
        assert!(threads_number > 0);
        let mut threads_handlers: Vec<thread::JoinHandle<()>> =
            Vec::with_capacity(threads_number as usize);

        // Create and start threads
        for thread_id in 0..threads_number {
            let thread_handler = thread::spawn(move || {
                let thread_id = thread_id;
                // Execute some Job
            });
        }

        // Atomic counter will be increased when a Job is sent to the Thread Pool and will be decreased after it is executed by the thread.
        let active_jobs_counter = sync::Arc::new(atomic::AtomicU8::new(0));

        Self {
            threads_handlers,
            threads_number,
            active_jobs_counter,
        }
    }

    /// Sends a Job to be executed in some thread.
    pub fn send_job(&self, job: Job) {
        assert!(self.threads_handlers.len() > 0);
        self.active_jobs_counter
            .fetch_add(1, atomic::Ordering::SeqCst);
    }

    /// Checks if any Job is currently being executed.
    pub fn has_some_job(&self) -> bool {
        self.active_jobs_counter.load(atomic::Ordering::SeqCst) > 0
    }

    /// Blocks the current thread and waits for all Jobs to be finished.
    pub fn wait_for_jobs_finish(&self) {
        while self.has_some_job() {}
    }

    /// Blocks the current thread, waits for all Jobs to complete, and destroys the threads.
    /// Called by drop()
    pub fn destroy(&self) {
        assert!(self.threads_handlers.len() > 0);
        self.wait_for_jobs_finish();
    }
}

impl Drop for ThreadPool {
    /// Blocks the current thread, waits for all Jobs to complete, and destroys the threads.
    fn drop(&mut self) {
        self.destroy();
    }
}
