extern crate num;
extern crate rustyline;

mod calculator;
mod parser;

use rustyline::Editor;
use rustyline::error::ReadlineError;

fn main() {
	loop {
		let mut rl = Editor::<()>::new();
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

		match parser::parse(&input) {
			Ok(parsed) => match calculator::calculate(parsed) {
				Ok(result) =>  println!("{}", result),
				Err(err)   => eprintln!("{}", err)
			},
			Err(err) => eprintln!("{}", err)
		}
	}
}
