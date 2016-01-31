use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

pub mod bencode;

fn handle_client(stream: TcpStream) {
    match stream.peer_addr().unwrap() {
        SocketAddr::V4(ipv4) => {
            println!("OMG I GOT A CONNECTION! From {}", ipv4);
        }
        SocketAddr::V6(ipv6) => {
            println!("OMG I GOT A CONNECTION! From {}", ipv6);
        }
    }
}

fn main() {
    println!("Hello, world!");

    let listener = TcpListener::bind("127.0.0.1:11234").unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                        // connection succeeded
                        handle_client(stream)
                    });
            }
            Err(e) => {
                println!("We got an error! {}", e);
            }
        }
    }

    // close the socket server
    drop(listener);
}
