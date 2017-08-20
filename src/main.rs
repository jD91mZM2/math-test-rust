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
				variables: variables,
				tokens: parsed.into_iter().peekable()
			}) {
				// Ok(result) =>  println!("{} (Binary: {:b}) (Hex: {:X})", result, result, result),
				Ok(result) => {
					use num::Zero;
					if result.is_zero() {
						return;
					}
					match variables.get("out").unwrap().to_u8() {
						Some(2)  => println!("= {:b}", result),
						Some(10) => println!("= {}",   result),
						Some(16) => println!("= {:X}", result),
						_  =>       {
							eprintln!("Warning: Unsupported \"out\" variable value");
							println!("= {}", result);
						},
					}
				}
				Err(err)   => eprintln!("Error: {}", err)
			}
		},
		Err(err) => eprintln!("Error: {}", err)
	}
}
