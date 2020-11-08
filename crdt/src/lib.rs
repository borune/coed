//! # Crdt for textediting with transport 
//!
//! A library for using a CRDT text for a collaborative text editor.
extern crate transport;
extern crate ditto;

#[macro_use]
extern crate serde_derive;
extern crate toml;
extern crate serde_json;

use transport::Transport;
use ditto::Text;
use std::fs::File;
use std::io::Read;
use std::io::Write;

pub struct CrText {
	obj: ditto::Text,
	node_id: u32,
    transport: transport::Transport,
}

#[derive(Debug,Deserialize)]
struct Config {
    node: u32,
    crdt: String,
    ip: String,
    port: String,
    other_ip: Option<String>,
    other_port: Option<String>,
}

#[derive(Serialize, Deserialize)]
enum Package<'a> {
    Leave (String),
    Listener (Option<std::net::SocketAddr>),
    Peers (Vec<std::net::SocketAddr>),
    Op (ditto::text::Op),
    State (ditto::TextState<'a>),
}

impl CrText {
	/// Returns a new CRDT struct. Expects a filename for configurating the network
	/// and a tx channel to return incomming data from each node. 
	pub fn new(filename:&str, tx_ch:std::sync::mpsc::Sender< String >) -> Self {
	    let mut contents = String::new();
	    File::open(filename)
	    	.expect("config file not found")
	    	.read_to_string(&mut contents)
	        .expect("something went wrong reading the file");
	    let config: Config = toml::from_str(&contents).unwrap();
	    let tx_ch_clone = tx_ch.clone();
	    let mut trans = Transport::new(config.ip, config.port, tx_ch_clone);

        match (config.other_ip, config.other_port) {
            (Some(ip),Some(port)) => {
				let listener = Some(trans.listener_addr);
				let listener_package = serde_json::to_string( &Package::Listener(listener)).unwrap();
            	let addr:std::net::SocketAddr = (ip + ":" + &port).parse().unwrap();
                trans.join(addr,tx_ch,listener_package);
            }
            (_,_) => (),
        }

		CrText {
            obj: Text::new(),
            node_id: config.node,
            transport: trans,
		}	
	}

    pub fn value(&self) -> String {
        self.obj.local_value()
    }

    pub fn update(&mut self, index:usize, replace:usize, s:&str ) {
        let op = self.obj.replace( index, replace, s ).unwrap().unwrap();
        self.transport.distribute(serde_json::to_string( &Package::Op(op) ).unwrap());
    }

    pub fn incomming_conn(&mut self) {
    	let mut stream = self.transport.listener_receiver.iter().next().unwrap();

		let list = self.transport.other_listeners.clone();
		let p = &Package::Peers( list );
    	let responce_of_ip_list = serde_json::to_string( p ).unwrap();
    	stream.write( responce_of_ip_list.as_bytes() ).unwrap();

		let state_package = &Package::State( self.obj.clone_state() );
		let state_package_str = serde_json::to_string( state_package ).unwrap();
        stream.write( state_package_str.as_bytes() ).unwrap();

    	self.transport.net_txs.push(stream);
    }

    pub fn connected(&mut self) -> Vec<std::net::SocketAddr>{
    	self.transport.connected()
    }

    pub fn inc_package(&mut self, s:String) {
    	let package = serde_json::from_str( &s ).unwrap();
		match package { 
		    Package::Op(op) => {
		    	self.obj.execute_op(op);
		    }
		    Package::State(state) => {
		        self.obj = ditto::Text::from_state(state,Some(self.node_id)).unwrap();
		    }
		    Package::Listener(addr) => {
		    	match addr {
		    		Some(address) => {
		    			self.transport.other_listeners.push(address);
		    		}
		    		None => {}
		    	}
		    }
		    Package::Peers(ip_list) => {
		    	for addr in ip_list {
		    		println!("{:?}", addr);
		    		let ch = self.transport.tx_ch.clone();
					let listener_package = serde_json::to_string( &Package::Listener(None)).unwrap();
		    		self.transport.join(addr, ch, listener_package);
		    	}
		    }
		    Package::Leave(addr) => { 
		        let index = &self.transport.net_txs
		            .iter()
		            .position(|tx| tx.peer_addr().unwrap().to_string() != addr)
		            .unwrap();
		        self.transport.net_txs.remove(*index);
		    },
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
