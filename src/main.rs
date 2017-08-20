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
	variables.insert("in".to_string(),  BigInt::from(10));
	variables.insert("out".to_string(), BigInt::from(10));

	for arg in env::args().skip(1) {
		if let Some(output) = calculate(&arg, &mut variables) {
			println!("{}", output);
		}
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
		if let Some(output) = calculate(&input, &mut variables) {
			println!("= {}", output);
		}
	}
}

pub fn calculate(input: &str, variables: &mut HashMap<String, BigInt>) -> Option<String> {
	use num::ToPrimitive;
	let radix = match variables.get("in").unwrap().to_u32() {
		Some(radix) if radix >= 2 && radix <= 36 => radix,
		Some(_) => {
			eprintln!("Warning: \"in\" must be between 2 and 36.");
			10
		},
		None => {
			eprintln!("Warning: Unsupported \"in\" variable value");
			10
		}
	};
	match parser::parse(input, radix) {
		Ok(parsed) => {
			match calculator::calculate(&mut calculator::Context {
				tokens: parsed.into_iter().peekable(),
				variables: variables,
				toplevel: true
			}) {
				Ok(result) => {
					use num::Zero;
					if result.is_zero() {
						return None;
					}
					match variables.get("out").unwrap().to_u8() {
						Some(2)  => return Some(format!("{:b}", result)),
						Some(10) => return Some(result.to_string()),
						Some(16) => return Some(format!("{:X}", result)),
						_  => {
							eprintln!("Warning: Unsupported \"out\" variable value");
							return Some(result.to_string())
						},
					}
				}
				Err(err) => eprintln!("Error: {}", err)
			}
		},
		Err(err) => eprintln!("Error: {}", err)
	}
	None
}
