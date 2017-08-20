extern crate num;
extern crate rustyline;

mod calculator;
mod parser;

use num::BigInt;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::env;

fn main() {
	let mut terminate = false;
	let mut variables = HashMap::new();

	for arg in env::args().skip(1) {
		calculate(&arg, &mut variables);
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
		if input.is_empty() {
			continue;
		}
		rl.add_history_entry(&input);
		calculate(&input, &mut variables);
	}
}

pub fn calculate(input: &str, variables: &mut HashMap<String, BigInt>) {
	match parser::parse(input) {
		Ok(parsed) => {
			match calculator::calculate(&mut calculator::Context {
				variables: variables,
				tokens: parsed.into_iter().peekable()
			}) {
				// Ok(result) =>  println!("{} (Binary: {:b}) (Hex: {:X})", result, result, result),
				Ok(result) =>  println!("{}", result),
				Err(err)   => eprintln!("Error: {}", err)
			}
		},
		Err(err) => eprintln!("Error: {}", err)
	}
}
