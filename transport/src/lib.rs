extern crate terminal;
extern crate serde_json;

use terminal::Source;
use std::net::{TcpListener, TcpStream};
// use std::net::Shutdown; // need to implement shutdown later
use std::thread;
use std::sync::mpsc::channel;
use std::str;
use std::io::{Read, Write};

pub struct Transport {
    pub listener_addr: std::net::SocketAddr,
    pub other_listeners: Vec<std::net::SocketAddr>,
    pub tx_ch: std::sync::mpsc::Sender< String >,
    pub net_txs: Vec< std::net::TcpStream >,
    pub listener_receiver: std::sync::mpsc::Receiver< std::net::TcpStream >
}

impl Transport {
    pub fn new(ip:String, port:String, tx_ch:std::sync::mpsc::Sender< String >) -> Transport {
        let listener = TcpListener::bind( ip + ":" + &port )
            .expect("Transport::new() Could not bind to the given ip");
        let listener_addr = listener.local_addr().unwrap();
        let (listener_sender, listener_receiver) = channel();
        let tx_ch_clone = tx_ch.clone();
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Err(e) => { 
                        eprintln!("Transport::new() - Stream failed: {}", e) 
                    }
                    Ok(stream) => {
                        let tx_ch_clone_2 = tx_ch_clone.clone();
                        listener_sender.send( stream.try_clone() 
                            .expect("Transport::new() - clone failed") ).unwrap();
                        tx_ch_clone.send( serde_json::to_string(&Source::Stream)
                            .unwrap()).unwrap();
                        thread::spawn(move || {
                            incoming_conn(stream,tx_ch_clone_2);
                        });
                    }
                }
            }
        });

        Transport {
            listener_addr: listener_addr,
            other_listeners: vec![],
            tx_ch: tx_ch,
            net_txs: vec![],
            listener_receiver: listener_receiver
        }
    }
    pub fn join(&mut self, 
            addr:std::net::SocketAddr, 
            tx_ch:std::sync::mpsc::Sender< String >,
            listener_package: String) {
        let mut stream = TcpStream::connect(addr)
            .expect("Transport::join() - Could not connect to server");
        stream.write(listener_package.as_bytes())
                .expect("transport::distribute - Failed to write to server");           
        self.net_txs.push( stream.try_clone()
            .expect("Transport::join - stream clone failed") );
        tx_ch.send( serde_json::to_string(&Source::Join).unwrap()).unwrap();
        thread::spawn(move || { 
            let mut buf = [0; 512];
            loop {
                let bytes_read = stream.read(&mut buf)
                    .expect("Transport::join() Failed to read stream");
                if bytes_read != 0 {  
                    let s = str::from_utf8(&buf[..bytes_read]).unwrap().to_string();
                    let st = serde_json::to_string( &(Source::Net(s)) ).unwrap();
                    tx_ch.send(st).unwrap();
                }
            }
        });
    }
    pub fn distribute(&mut self, op:String) {
        for mut stream in &self.net_txs {
            stream.write(op.as_bytes())
                .expect("transport::distribute - Failed to write to server");           
        }
    }
    pub fn connected(&self) -> Vec<std::net::SocketAddr> {
        let mut ip_list = Vec::new();
        for t in &self.net_txs {
            ip_list.push( t.peer_addr().unwrap());
        }
        ip_list
    }

}

fn incoming_conn(mut stream: TcpStream, tx_ch:std::sync::mpsc::Sender<String>) {
    let mut buf = [0; 512];
    loop {
        let bytes_read = stream.read(&mut buf)
            .expect("Transport::new() Failed to read stream");
        if bytes_read != 0 {  
            let s = str::from_utf8(&buf[..bytes_read]).unwrap().to_string();
            let st = serde_json::to_string( &(Source::Net(s)) ).unwrap();
            tx_ch.send(st).unwrap();
        }
    }   
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
