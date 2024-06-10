use std::thread;
use std::{sync, sync::atomic, sync::mpsc};

pub struct ThreadPool {
    threads_handlers: Vec<thread::JoinHandle<()>>,
    _threads_number: u8,
    active_threads_number: sync::Arc<atomic::AtomicU8>,
    active_jobs_counter: sync::Arc<atomic::AtomicU8>,
    jobs_queue_size: sync::Arc<atomic::AtomicU8>,
    job_sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() -> () + Send + 'static>;

impl ThreadPool {
    /// Creates a ThreadPool and starts threads_number of threads ready for Jobs.
    pub fn new(threads_number: u8) -> Self {
        assert!(threads_number > 0);

        let mut threads_handlers: Vec<thread::JoinHandle<()>> =
            Vec::with_capacity(threads_number as usize);

        // Atomic counter will be decreased during the thread finishing.
        let active_threads_number = sync::Arc::new(atomic::AtomicU8::new(threads_number));

        // Atomic counter will be increased before Job executing by thread and will be decreased after it is executed by the thread.
        let active_jobs_counter = sync::Arc::new(atomic::AtomicU8::new(0));

        // Atomic counter will be increased when sending a Job to the Thread Pool and decrease when the thread takes the Job for execution.
        let jobs_queue_size = sync::Arc::new(atomic::AtomicU8::new(0));

        // Each Job will be sent to a channel from which it will be read by a free thread and executed.
        let (job_sender, job_receiver) = mpsc::channel::<Job>();
        let job_receiver_mutex = sync::Arc::new(sync::Mutex::new(job_receiver));

        // Create and start threads
        for thread_id in 0..threads_number {
            let active_threads_number = sync::Arc::clone(&active_threads_number);
            let active_jobs_counter = sync::Arc::clone(&active_jobs_counter);
            let jobs_queue_size = sync::Arc::clone(&jobs_queue_size);
            let job_receiver_mutex = sync::Arc::clone(&job_receiver_mutex);
            // Create and start thread
            let thread_handler = thread::spawn(move || {
                let _thread_id = thread_id;
                //println!("Starting thread {thread_id}");
                loop {
                    // Get job receiver mutex guard
                    let job_receiver_mutex_guard = job_receiver_mutex.lock().unwrap();
                    //println!("Thread {thread_id} waiting Job");
                    // Receive a Job from channel
                    let result = job_receiver_mutex_guard.recv();
                    // Unlock mutex
                    drop(job_receiver_mutex_guard);
                    if let Err(_error) = result {
                        // The sending side has disconnected and will no longer send work,
                        // which means the Thread Pool is no longer working and this thread can be terminated.
                        // Shutdown thread
                        break;
                    }
                    // Job received
                    jobs_queue_size.fetch_sub(1, atomic::Ordering::SeqCst);
                    let job = result.unwrap();
                    // Execute Job
                    active_jobs_counter.fetch_add(1, atomic::Ordering::SeqCst);
                    //println!("Thread {thread_id} starts Job executing...");
                    job();
                    active_jobs_counter.fetch_sub(1, atomic::Ordering::SeqCst);
                }
                active_threads_number.fetch_sub(1, atomic::Ordering::SeqCst);
                //println!("Finishing thread {thread_id}");
            });
            threads_handlers.push(thread_handler);
        }

        Self {
            threads_handlers,
            _threads_number: threads_number,
            active_threads_number,
            active_jobs_counter,
            jobs_queue_size,
            job_sender: Some(job_sender),
        }
    }

    /// Sends a Job to be executed in some thread.
    pub fn send_job(&self, job: Job) {
        assert!(self.threads_handlers.len() > 0);
        // Send Job to the channel
        self.jobs_queue_size.fetch_add(1, atomic::Ordering::SeqCst);
        let result = self.job_sender.as_ref().unwrap().send(job);
        if let Err(_error) = result {
            // If an error has occurred, it means that the threads cannot accept Job (they have been destroyed)
            self.jobs_queue_size.fetch_sub(1, atomic::Ordering::SeqCst);
            panic!("An attempt to send a Job to the Thread Pool when all threads are destroyed.");
        }
    }

    /// Checks if the threads has Job's that it is executing or can execute
    pub fn has_some_job(&self) -> bool {
        self.active_jobs_counter.load(atomic::Ordering::SeqCst) > 0
            && self.jobs_queue_size.load(atomic::Ordering::SeqCst) == 0
    }

    /// Blocks the current thread and waits for all Jobs to be finished.
    pub fn wait_for_jobs_finish(&self) {
        while self.has_some_job() {}
    }
}

impl Drop for ThreadPool {
    /// Blocks the current thread, waits for all threads finishing, and destroys the threads.
    fn drop(&mut self) {
        // Wait for jobs finish
        self.wait_for_jobs_finish();
        // Destroying the Sender causes all threads to finish
        let sender = self.job_sender.take().unwrap();
        drop(sender);
        // Wait for threads finish
        while self.active_threads_number.load(atomic::Ordering::SeqCst) > 0 {}
    }
}
