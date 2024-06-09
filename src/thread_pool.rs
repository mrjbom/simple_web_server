use std::thread;
use std::{sync, sync::atomic, sync::mpsc};

pub struct ThreadPool {
    threads_handlers: Vec<thread::JoinHandle<()>>,
    threads_number: u8,
    active_jobs_counter: sync::Arc<atomic::AtomicU8>,
    job_sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() -> () + Send + 'static>;

impl ThreadPool {
    /// Creates a ThreadPool and starts threads_number of threads ready for Jobs.
    pub fn new(threads_number: u8) -> Self {
        assert!(threads_number > 0);
        let mut threads_handlers: Vec<thread::JoinHandle<()>> =
            Vec::with_capacity(threads_number as usize);

        // Atomic counter will be increased when a Job is sent to the Thread Pool and will be decreased after it is executed by the thread.
        let active_jobs_counter = sync::Arc::new(atomic::AtomicU8::new(0));

        // Each Job will be sent to a channel from which it will be read by a free thread and executed.
        let (job_sender, job_receiver) = mpsc::channel::<Job>();
        let job_receiver_mutex = sync::Arc::new(sync::Mutex::new(job_receiver));

        // Create and start threads
        for thread_id in 0..threads_number {
            // Active threads counter for this thread
            let active_jobs_counter = sync::Arc::clone(&active_jobs_counter);
            // Job receiver for thread
            let job_receiver_mutex = sync::Arc::clone(&job_receiver_mutex);
            // Create and start thread
            let thread_handler = thread::spawn(move || {
                let thread_id = thread_id;
                let active_jobs_counter = active_jobs_counter;
                println!("Starting thread {thread_id}");
                loop {
                    // Get job receiver mutex guard
                    let job_receiver_mutex_guard = job_receiver_mutex.lock().unwrap();
                    println!("Thread {thread_id} waiting Job");
                    let result = job_receiver_mutex_guard.recv();
                    // Unlock mutex
                    drop(job_receiver_mutex_guard);
                    if let Err(_error) = result {
                        // The sending side has disconnected and will no longer send work,
                        // which means the Thread Pool is no longer working and this thread can be terminated.
                        // Shutdown thread
                        break;
                    }
                    let job = result.unwrap();
                    // Execute Job
                    active_jobs_counter.fetch_add(1, atomic::Ordering::SeqCst);
                    println!("Thread {thread_id} execute Job...");
                    job();
                    active_jobs_counter.fetch_sub(1, atomic::Ordering::SeqCst);
                }
                println!("Finishing thread {thread_id}");
            });
        }

        Self {
            threads_handlers,
            threads_number,
            active_jobs_counter,
            job_sender,
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
