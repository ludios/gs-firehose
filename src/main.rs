extern crate ws;
extern crate env_logger;
extern crate serde_json;

use ws::{connect, CloseCode};
use serde_json::Value;
use std::env;

fn main() {
	// Set up logging.  Set the RUST_LOG env variable to see output.
	env_logger::init().unwrap();

	if let Err(error) = connect("ws://127.0.0.1:29001", |out| {
		// Queue a message to be sent when the WebSocket is open
		if let Err(_) = out.send(r#"{"type": "hello", "mode": "dashboard"}"#) {
			println!("Websocket couldn't queue an initial message.")
		}

		// The handler needs to take ownership of out, so we use move
		move |msg: ws::Message| {
			let text = msg.as_text().unwrap();
			let ev: Value = serde_json::from_str(text).unwrap();
			if let Some(message) = ev.find("message") {
				let trimmed = message.as_string().unwrap().trim_right();
				if !trimmed.is_empty() {
					println!("{}", trimmed);
				}
			} else {
				let response_code = ev.find("response_code").unwrap().as_u64().unwrap();
				let url = ev.find("url").unwrap().as_string().unwrap();
				println!(" {} {}", response_code, url);
			}

			out.close(CloseCode::Normal)
		}

	}) {
		println!("Failed to create WebSocket due to: {:?}", error);
	}
}
