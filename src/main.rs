use std::net::TcpListener;

fn main() {
  let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
  for stream_res in listener.incoming() {
    let stream = stream_res.unwrap();
    println!("Connection established!");
  }
}