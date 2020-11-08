extern crate terminal;
extern crate crdt;

use terminal::{Events,Terminal};

struct Cursor {
	index: usize,
	col: u16,
	// row: u16,
}

struct Editor {
	crdt: crdt::CrText,
	cursor: Cursor 
}

impl Events for Editor {
	fn on_key_press(&mut self, s:&str) {
		self.crdt.update(self.cursor.index,0,s);
		self.cursor.inc();
	}
	fn on_key_backspace(&mut self) {
		match self.cursor.dec() {
			Some(index) => self.crdt.update(index,1,""),
			None => {}
		}
	}
	fn on_join_conn(&mut self) -> Vec<std::net::SocketAddr> {
		self.crdt.connected()	
	}
	fn on_new_incomming_conn(&mut self) -> Vec<std::net::SocketAddr> {
		self.crdt.incomming_conn();
		self.crdt.connected()	
	}
	fn on_net_package(&mut self, s:&str) {
		self.crdt.inc_package(s.to_string());
	}
	fn on_screen_refresh(&self) -> (String,usize) {
		(self.crdt.value(),self.cursor.index())
	}
	fn on_key_quit(&self) {
		println!("got a key stroke to quit");
	}
}

impl Cursor {
	pub fn index(&self) -> usize {
		self.index
	}
	pub fn inc(&mut self) {
		self.index = self.index + 1;
		self.col = self.col + 1;
	}
	pub fn dec(&mut self) -> Option<usize> {
		if self.index > 0 {			
			self.index = self.index - 1;
			self.col = self.col - 1;
			Some(self.index)
		} else {
			None
		}
		
	}
}

fn main() {
	// Path to the config.toml file needed for transport layer    
	let args: Vec<String> = std::env::args().collect();
	// Set up a new terminal screen, switches to secondary clean screen,
	// that can switch back to the console screen contents with the 
	// screen.exit()
	let mut term = Terminal::new();
	let tx_ch = term.screen_tx_channel();
	term.add_event_hook(Editor {
		crdt: crdt::CrText::new( &args[1], tx_ch ),
		cursor: Cursor {index:0,col:0}
	});

	term.clear();
	term.top_bar();
	term.bottom_bar(" ".to_string());
	term.done();

	term.keys();
	term.screen();
	term.exit();
}
