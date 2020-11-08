extern crate termion;
use termion::input::TermRead;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::ToMainScreen;
use std::{io::{Write, stdout, stdin},
		  sync::mpsc,
		  thread
		 };
use termion::screen::AlternateScreen;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;


#[derive(Serialize, Deserialize)]
pub enum Source {
    Key (String),
	Quit,
	Join,
	Backspace,
	Stream,
    Net (String),
}

#[allow(unused_variables)]
pub trait Events {
	fn on_key_press(&mut self, s:&str);
	fn on_key_backspace(&mut self);
	fn on_screen_refresh(&self) -> (String,usize);
	fn on_key_quit(&self) {}
	fn on_join_conn(&mut self) -> Vec<std::net::SocketAddr>;
	fn on_new_incomming_conn(&mut self) -> Vec<std::net::SocketAddr>;
	fn on_net_package(&mut self, s:&str);
}

pub struct Terminal {
	screen: termion::screen::AlternateScreen<termion::raw::RawTerminal<std::io::Stdout>>,
	// screen: termion::raw::RawTerminal<std::io::Stdout>,
	hooks: Vec<Box<Events>>,
	rx: std::sync::mpsc::Receiver<String>,
	tx: std::sync::mpsc::Sender<String>,
}

impl Terminal {
	pub fn new() -> Self {
		let (tx, rx) = mpsc::channel();
		Self {
			// screen: stdout().into_raw_mode().unwrap(),
			screen: AlternateScreen::from(stdout().into_raw_mode().unwrap()),
			hooks: Vec::new(),
			rx: rx,
			tx: tx,
		}
	}

	pub fn add_event_hook<E: Events + 'static>(&mut self, hook: E) {
		self.hooks.push(Box::new(hook));
	}

	pub fn screen_tx_channel(&self) -> std::sync::mpsc::Sender<String> {
		self.tx.clone()
	}

	pub fn clear(&mut self) {
		write!(self.screen, "{}", termion::clear::All ).unwrap();
	}
	pub fn done(&mut self) {
	    self.screen.flush().unwrap();
	}
	pub fn top_bar(&mut self) {
		let termsize = termion::terminal_size().ok();
	    let termwidth = termsize.map(|(w,_)| w - 2).unwrap();
	    // let termheight = termsize.map(|(_,h)| h - 2).unwrap();
	    write!(self.screen,
	           "{}{}====== Colaborative editor ====== ESC to exit  ",
	           termion::cursor::Goto(1, 1),
	           termion::style::Bold,
	        ).unwrap();
        write!(self.screen,"{}",termion::cursor::Goto(47, 1)).unwrap();
	    for _ in 44..termwidth {
            write!(self.screen,"=").unwrap();
        }
	    write!(self.screen, "{}{}",
	           termion::style::Reset,
	           termion::cursor::Goto(1, 2)
	        ).unwrap();
	}

	pub fn bottom_bar(&mut self, s:String) {
		let termsize = termion::terminal_size().ok();
	    let termwidth = termsize.map(|(w,_)| w - 2).unwrap() + 2;
	    let termheight = termsize.map(|(_,h)| h - 2).unwrap() + 2;
	    let text = format!("====== Connected to [ {} ] ",s);
	    write!(self.screen,"{}{}{}",
	           termion::cursor::Goto(1, termheight),
	           termion::style::Bold,
	           &text,
	        ).unwrap();

	    for _ in (text.len() as u16)..termwidth {
            write!(self.screen,"=").unwrap();
        }
	    write!(self.screen, "{}{}",
	           termion::style::Reset,
	           termion::cursor::Goto(1, 2)
	        ).unwrap();
	}

	pub fn keys(&mut self) {
		let stdin = stdin();
		let ksx = self.tx.clone();
	    thread::spawn(move || {
			for c in stdin.keys() {
				let package = match c.unwrap() {
		            Key::Esc       => Source::Quit,
		            Key::Char(c)   => Source::Key(format!("{}", c)),
		            Key::Backspace => Source::Backspace,
		            // Key::Alt(c)    => format!("Alt-{}", c),
		            // Key::Ctrl(c)   => format!("Ctrl-{}", c),
		            // Key::Left      => "<left>".to_string(),
		            // Key::Right     => "<right>".to_string(),
		            // Key::Up        => "<up>".to_string(),
		            // Key::Down      => "<down>".to_string(),
		            _              => Source::Key( "".to_string() ),
			    };
            	let st = serde_json::to_string( &package ).unwrap();
				ksx.send( st ).unwrap();
			}
		});

	}

	pub fn screen(&mut self) {
		let mut list:Vec<std::net::SocketAddr> = Vec::new();
		for c in self.rx.iter() {
			match serde_json::from_str(&c).unwrap() {
				Source::Key(key_pressed) => {
					for hook in &mut self.hooks {
						hook.on_key_press(&key_pressed);
					}
				}
				Source::Backspace => {
					for hook in &mut self.hooks {
						hook.on_key_backspace();
					}
				}
				Source::Join => {
					for hook in &mut self.hooks {
						list = hook.on_join_conn();
					}
				}
				Source::Quit => {
					for hook in &self.hooks {
						hook.on_key_quit();
					}
	            	break
				}
				Source::Stream => {
					for hook in &mut self.hooks {
						list = hook.on_new_incomming_conn();
					}
				}
				Source::Net(json_string) => {
					for hook in &mut self.hooks {
						hook.on_net_package(&json_string);
					}
				}
			};

			let mut scr = "".to_string();
			let mut index = 0;
			for hook in &self.hooks {
				let h = hook.on_screen_refresh();
				scr = h.0;
				index = h.1;
			}

			let termsize = termion::terminal_size().ok();
		    let termwidth = termsize.map(|(w,_)| w - 2).unwrap();
		    let termheight = termsize.map(|(_,h)| h).unwrap();
			let mut text_list = String::new();
			for addr in &list {
				text_list = text_list + &format!("{:?}",&addr);
			}

		    let text = format!("====== Connected to [ {} ] ", text_list);
		    write!(self.screen,"{}{}{}",
		           termion::cursor::Goto(1, termheight),
		           termion::style::Bold,
		           &text,
		        ).unwrap();

		    for _ in (text.len() as u16)..termwidth {
	            write!(self.screen,"=").unwrap();
	        }
		    write!(self.screen, "{}{}",
		           termion::style::Reset,
		           termion::cursor::Goto(1, 2)).unwrap();

			for i in 2..(termheight-1) {
			    write!(self.screen,
			    		"{}{}",
			           termion::cursor::Goto(1, i),
			           termion::clear::CurrentLine
			        ).unwrap();
			}
		    write!(self.screen, "{}",
		           termion::cursor::Goto(1, 2)
		        ).unwrap();

		    let split = scr.split("\n");
			let mut row:u16 = 1;
			let mut cursor:(u16,u16) = (1,1);
			for s in split {
				write!(self.screen, "{}{}", termion::cursor::Goto(1,row+1),s).unwrap();
				row = row + 1;
				if index > s.len() {
					index = index - (s.len() + 1);
				} else {
					cursor = ((index as u16) + 1, row);
				}
			}
			write!(self.screen, "{}", termion::cursor::Goto(cursor.0,cursor.1)).unwrap();


			self.screen.flush().unwrap();
		}
	}

	pub fn exit(&mut self) {
		write!(self.screen, "{}", ToMainScreen ).unwrap();
	    self.screen.flush().unwrap();
	}

}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn it_works() {
        assert_eq!(Terminal::new(), Terminal);
    }
}
