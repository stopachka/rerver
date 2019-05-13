use std::io;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

/*
  Job
*/ 

trait FnBox {
  fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
  fn call_box(self: Box<Self>) {
    (*self)()
  }
}

type Job = Box<FnBox + Send + 'static>;

/* 
  Worker
*/

struct Worker {
  _thread: thread::JoinHandle<()>,
}

impl Worker {
  fn new(
    id: usize, 
    receiver: Arc<Mutex<mpsc::Receiver<Job>>>
  ) -> Worker {
    let _thread = thread::spawn(move || {
      loop {
        let job = receiver.lock().unwrap().recv().unwrap();    
        println!("worker {} got a job; executing.", id);
        job.call_box();
      }
    });

    Worker {_thread}
  }
}

/* 
  ThreadPool
*/

struct ThreadPool {
  _workers: Vec<Worker>,
  sender: mpsc::Sender<Job>,
}

impl ThreadPool {
  fn new(size: usize) -> ThreadPool {
    let (sender, plain_receiver) = mpsc::channel();
    
    let receiver = Arc::new(Mutex::new(plain_receiver));

    let _workers = (1..size).map(|i| { Worker::new(i, Arc::clone(&receiver)) }).collect();

    ThreadPool {_workers, sender}
  }
  fn push(&self, f: Job) {
    match self.sender.send(f) {
      Ok(_) => println!("sent a job"),
      Err(e) => println!("failed to send a job: {}", e),
    }
  }
}

fn handle_connection(mut stream: TcpStream) -> Result<(), io::Error> {
  let mut buffer = [0; 512];
  stream.read(&mut buffer)?;
  if buffer.starts_with(b"GET /sleep") {
    thread::sleep(Duration::from_secs(5));
  }
  let contents = r#"
    <!DOCTYPE html>
    <html lang="en">
    <head>
      <meta charset="utf-8">
      <title>Hello!</title>
    </head>
    <body>
      <h1>Hello!</h1>
      <p>Hi from Rust</p>
    </body>
  </html>"#;
  let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}", contents.len(), contents);
  stream.write(response.as_bytes())?;
  stream.flush()?;

  Ok(())
}

fn handle_listener(listener: TcpListener, pool_size: usize) {
  let pool = ThreadPool::new(pool_size);
  for stream_res in listener.incoming() {
    match stream_res {
      Ok(stream) => {
        pool.push(
          Box::new(
            move || {
              match handle_connection(stream) {
                Ok(_) => println!("handled connection"),
                Err(e) => println!("failed to handle connectiton: {}", e)
              }
            }
          )
        );
      },
      Err(e) => println!("failed to receive a stream: {}", e)
    }
  }
}

fn main() {
  match TcpListener::bind("127.0.0.1:7878") {
    Ok(listener) => handle_listener(listener, 10),
    Err(e) => println!("failed to bind: {}", e),
  }
}