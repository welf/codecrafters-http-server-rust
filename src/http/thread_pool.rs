use std::{
    sync::{mpsc, Arc, Mutex},
    thread::JoinHandle,
};

#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

#[derive(Debug)]
struct Worker {
    id: usize,
    thread: JoinHandle<()>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool with the specified number of threads.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> Self {
        assert!(size > 0, "ThreadPool size must be greater than 0");

        let mut workers = Vec::with_capacity(size);

        let (tx, rx) = mpsc::channel::<Job>();

        let receiver = Arc::new(Mutex::new(rx));

        for id in 0..size {
            let worker = Worker::new(id, receiver.clone());
            workers.push(worker);
        }

        Self {
            workers,
            sender: tx,
        }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        let job = Box::new(f);
        self.sender
            .send(job)
            .expect("Worker we are sending to has died!");
    }
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = std::thread::Builder::new()
            .spawn(move || loop {
                let job = receiver
                    .lock()
                    .expect("Mutex is in a poisoning state!")
                    .recv()
                    .expect("Sender was dropped!");
                println!("Worker {id} got a job; executing...");

                job();
            })
            .unwrap();

        Worker { id, thread }
    }
}
