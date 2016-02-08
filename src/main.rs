#[macro_use]
extern crate lazy_static;
extern crate ws;
extern crate env_logger;
extern crate serde_json;
extern crate clap;
extern crate ansi_term;

use ws::{connect, CloseCode};
use serde_json::Value;
use clap::{Arg, App};
use ansi_term::{Style, Colour};
use ansi_term::Colour::{Fixed, Black};

struct DashboardColors {
	ident: Colour,
	stdout: Style,
	redirect: Style,
	warning: Style,
	error: Style,
	none: Style
}

lazy_static! {
	static ref COLORS: DashboardColors = DashboardColors {
		ident: Fixed(244), // gray
		stdout: Black.on(Fixed(254)), // gray
		redirect: Black.on(Fixed(225)), // purple
		warning: Black.on(Fixed(221)), // yellow
		error: Black.on(Fixed(210)), // red
		none: Style::new()
	};
}

fn print_like_dashboard(msg: ws::Message) {
	let colors = &*COLORS;
	let text = msg.as_text().unwrap();
	let ev: Value = serde_json::from_str(text).unwrap();
	//println!("{:?}", ev);
	let ident = ev.lookup("job_data.ident").unwrap().as_string().unwrap();
	if let Some(message) = ev.find("message") {
		let trimmed = message.as_string().unwrap().trim_right();
		if !trimmed.is_empty() {
			for line in trimmed.lines() {
				let padding = if line.starts_with("ERROR ") { "" } else { " " };
				println!("{} {}{}", colors.ident.paint(ident), padding, colors.stdout.paint(line));
			}
		}
	} else {
		let response_code = ev.find("response_code").unwrap().as_u64().unwrap();
		let url = ev.find("url").unwrap().as_string().unwrap();
		let color = match response_code {
			c if c >= 400 && c < 500 => colors.warning,
			c if c == 0 || c >= 500 => colors.error,
			c if c >= 300 && c < 400 => colors.redirect,
			_ => colors.none
		};
		println!("{}  {}",
			colors.ident.paint(ident),
			color.paint(
				format!(" {:>3} {}", response_code, url)));
	}
}

fn main() {
	let modes = ["dashboard", "json"];
	let matches =
		App::new("gs-firehose")
			.version("0.1")
			.about("Connects to a grab-site or ArchiveBot server and dumps all messages in either a human-readable or JSON format.")
			.arg(Arg::with_name("mode")
				.long("mode")
				.help("Output mode.  Default: 'dashboard'.  Use 'json' to dump raw traffic.")
				.takes_value(true)
				.possible_values(&modes))
			.arg(Arg::with_name("WS_URL")
				.help("The WebSocket URL to connect to.  Default: ws://127.0.0.1:29001")
				.index(1))
			.get_matches();

	let url = matches.value_of("WS_URL").unwrap_or("ws://127.0.0.1:29001");
	let mode = matches.value_of("mode").unwrap_or("dashboard");

	// Set up logging.  Set the RUST_LOG env variable to see output.
	env_logger::init().unwrap();

	if let Err(error) = connect(url, |out| {
		// Queue a message to be sent when the WebSocket is open
		if let Err(_) = out.send(r#"{"type": "hello", "mode": "dashboard"}"#) {
			panic!("Websocket couldn't queue an initial message.")
		}

		// The handler needs to take ownership of out, so we use move
		move |msg: ws::Message| {
			match mode {
				"dashboard" => print_like_dashboard(msg),
				"json" => println!("{}", msg),
				_ => panic!("Invalid mode {}", mode)
			};
			out.close(CloseCode::Normal)
		}
	}) {
		println!("Failed to create WebSocket due to: {:?}", error);
	}
}
