extern crate bigdecimal;
extern crate num;
extern crate rustyline;

mod calculator;
mod parser;

use bigdecimal::BigDecimal;
use rustyline::Editor;
use rustyline::error::ReadlineError;
use std::collections::HashMap;
use std::env;

fn main() {
	let mut terminate = false;
	let mut variables = HashMap::new();
	variables.insert("out".to_string(), BigDecimal::from(10));
	let mut functions = HashMap::new();

	for arg in env::args().skip(1) {
		if let Some(output) = calculate(&arg, &mut variables, &mut functions) {
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
		if let Some(output) = calculate(&input, &mut variables, &mut functions) {
			println!("= {}", output);
		}
	}
}

pub fn calculate(
		input: &str,
		variables: &mut HashMap<String, BigDecimal>,
		functions: &mut HashMap<String, Vec<parser::Token>>
	) -> Option<String> {
	use num::ToPrimitive;
	match parser::parse(input) {
		Ok(parsed) => {
			match calculator::calculate(&mut calculator::Context {
				tokens: parsed.into_iter().peekable(),
				toplevel: true,
				variables: variables,
				functions: functions
			}) {
				Ok(result) => {
					use num::Zero;
					use num::bigint::ToBigInt;
					if result.is_zero() {
						return None;
					}
					match variables.get("out").unwrap().to_u8() {
						Some(2)  => return Some(format!("{:b}", result.to_bigint().unwrap())),
						Some(10) => return Some(result.to_string()),
						Some(16) => return Some(format!("{:X}", result.to_bigint().unwrap())),
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
