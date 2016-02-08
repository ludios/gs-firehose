extern crate ws;
extern crate env_logger;
extern crate serde_json;
extern crate clap;
extern crate ansi_term;

use ws::{connect, CloseCode};
use serde_json::Value;
use clap::{Arg, App};
use ansi_term::Colour::{Fixed, Black};
use ansi_term::Style;

fn main() {
    let matches =
	App::new("gs-firehose")
		.version("0.1")
		.about("Connects to a grab-site or ArchiveBot server and dumps all messages in either a human-readable or JSON format.")
		.arg(Arg::with_name("WS_URL")
			.help("The WebSocket URL to connect to.  Default: ws://127.0.0.1:29001")
			.index(1))
		.get_matches();

	let url = matches.value_of("WS_URL").unwrap_or("ws://127.0.0.1:29001");

	// Set up logging.  Set the RUST_LOG env variable to see output.
	env_logger::init().unwrap();

	let gray = Fixed(244);
	let black_on_gray = Black.on(Fixed(254));
	let black_on_purple = Black.on(Fixed(225));
	let black_on_yellow = Black.on(Fixed(221));
	let black_on_red = Black.on(Fixed(210));
	let no_style = Style::new();

	if let Err(error) = connect(url, |out| {
		// Queue a message to be sent when the WebSocket is open
		if let Err(_) = out.send(r#"{"type": "hello", "mode": "dashboard"}"#) {
			println!("Websocket couldn't queue an initial message.")
		}

		// The handler needs to take ownership of out, so we use move
		move |msg: ws::Message| {
			let text = msg.as_text().unwrap();
			let ev: Value = serde_json::from_str(text).unwrap();
			//println!("{:?}", ev);
			let ident = ev.lookup("job_data.ident").unwrap().as_string().unwrap();
			if let Some(message) = ev.find("message") {
				let trimmed = message.as_string().unwrap().trim_right();
				if !trimmed.is_empty() {
					for line in trimmed.lines() {
						if line.starts_with("ERROR ") {
							println!("{} {}", gray.paint(ident), black_on_gray.paint(line));
						} else {
							println!("{}  {}", gray.paint(ident), black_on_gray.paint(line));
						}
					}
				}
			} else {
				let response_code = ev.find("response_code").unwrap().as_u64().unwrap();
				let url = ev.find("url").unwrap().as_string().unwrap();
				let color = match response_code {
					c if c >= 400 && c < 500 => black_on_yellow,
					c if c == 0 || c >= 500 => black_on_red,
					c if c >= 300 && c < 400 => black_on_purple,
					_ => no_style
				};
				println!("{}  {}",
					gray.paint(ident),
					color.paint(
						format!(" {:>3} {}", response_code, url)));
			}

			out.close(CloseCode::Normal)
		}

	}) {
		println!("Failed to create WebSocket due to: {:?}", error);
	}
}
