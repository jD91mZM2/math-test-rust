extern crate num;

mod parser;

use std::io;

fn main() {
	loop {
		let mut input = String::new();

		match io::stdin().read_line(&mut input) {
			Ok(0) => break,
			Ok(_) => (),
			Err(err) => {
				eprintln!("Read from STDIN failed.");
				eprintln!("Details: {}", err);
				break;
			},
		}
		let input = input.trim();

		println!("{:?}", parser::parse(input));
	}
}
