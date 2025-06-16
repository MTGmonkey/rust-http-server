use std::sync::{Arc, Mutex, mpsc};
use std::thread;

pub enum HTTP_RESPONSE_CODE {
    HTTP_200_OK,
    HTTP_400_BAD_REQUEST,
    HTTP_404_NOT_FOUND,
    HTTP_501_NOT_IMPLEMENTED,
    HTTP_505_HTTP_VERSION_NOT_SUPPORTED,
}
impl std::fmt::Debug for HTTP_RESPONSE_CODE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            HTTP_RESPONSE_CODE::HTTP_200_OK => write!(f, "HTTP_200_OK"),
            HTTP_RESPONSE_CODE::HTTP_400_BAD_REQUEST => write!(f, "HTTP_400_BAD_REQUEST"),
            HTTP_RESPONSE_CODE::HTTP_404_NOT_FOUND => write!(f, "HTTP_404_NOT_FOUND"),
            HTTP_RESPONSE_CODE::HTTP_501_NOT_IMPLEMENTED => write!(f, "HTTP_501_NOT_IMPLEMENTED"),
            HTTP_RESPONSE_CODE::HTTP_505_HTTP_VERSION_NOT_SUPPORTED => {
                write!(f, "HTTP_505_HTTP_VERSION_NOT_SUPPORTED")
            }
        }
    }
}
impl std::fmt::Display for HTTP_RESPONSE_CODE {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            HTTP_RESPONSE_CODE::HTTP_200_OK => write!(f, "200 OK"),
            HTTP_RESPONSE_CODE::HTTP_400_BAD_REQUEST => write!(f, "400 BAD REQUEST"),
            HTTP_RESPONSE_CODE::HTTP_404_NOT_FOUND => write!(f, "404 NOT FOUND"),
            HTTP_RESPONSE_CODE::HTTP_501_NOT_IMPLEMENTED => write!(f, "501 NOT IMPLEMENTED"),
            HTTP_RESPONSE_CODE::HTTP_505_HTTP_VERSION_NOT_SUPPORTED => {
                write!(f, "HTTP_505_HTTP_VERSION_NOT_SUPPORTED")
            }
        }
    }
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        match self.sender.as_ref() {
            Some(sender) => match sender.send(job) {
                Ok(_) => {}
                Err(e) => println!("[ERROR]: {e}"),
            },
            None => println!("[ERROR] no sender to execute!"),
        }
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            println!("Shutting down worker {}", worker.id);
            match worker.thread.join() {
                Ok(join) => join,
                Err(_) => println!(
                    "Failed to join thread while shutting down worker {}",
                    worker.id
                ),
            }
        }
    }
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                match receiver.lock() {
                    Ok(receiver) => {
                        let message = receiver.recv();
                        match message {
                            Ok(job) => {
                                println!("Worker {id} got a job, executing.");
                                job();
                            }
                            Err(_) => {
                                println!("Worker {id} disconnected, shutting down.");
                                break;
                            }
                        }
                    }
                    Err(e) => println!("[ERROR]: {e}"),
                }
            }
        });
        Worker { id, thread }
    }
}
