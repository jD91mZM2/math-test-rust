extern crate num;
extern crate rustyline;

mod calculator;
mod parser;

use std::env;
use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
	let mut terminate = false;
	for arg in env::args().skip(1) {
		calculate(&arg);
		terminate = true;
	}

	if terminate {
		return;
	}

	let mut rl = Editor::<()>::new();
	loop {
		let input = match rl.readline("> ") {
			Ok(input) => input,
			Err(ReadlineError::Interrupted) |
			Err(ReadlineError::Eof) => break,
			Err(err) => {
				eprintln!("Read from STDIN failed.");
				eprintln!("Details: {}", err);
				break;
			},
		};
		rl.add_history_entry(&input);
		calculate(&input);
	}
}

pub fn calculate(input: &str) {
	match parser::parse(input) {
		Ok(parsed) => match calculator::calculate(&mut parsed.into_iter().peekable()) {
			Ok(result) =>  println!("{}", result),
			Err(err)   => eprintln!("Error: {}", err)
		},
		Err(err) => eprintln!("Error: {}", err)
	}
}
