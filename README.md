gs-firehose
===

This is the terminal version of the grab-site dashboard.  It doesn't put crawl jobs in their own log windows, though you can use grep to filter for a particular job.

Usage
---
1.	[Install Rust Nightly](https://www.rust-lang.org/downloads.html), which includes rustc and cargo.
2.	`git clone https://github.com/ludios/gs-firehose`
3.	`cd gs-firehose/`
4.	`cargo build --release`
5.	`./target/release/gs-firehose --help`

	or just

	`./target/release/gs-firehose`

	to connect to the default grab-site WebSocket URL `ws://127.0.0.1:29000`

Terminal colors
---
The colors were set for dark text-on-light background terminals, please submit a PR for the reverse.
