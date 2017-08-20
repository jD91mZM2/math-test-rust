use std::{self, fmt, mem};
use num::BigInt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
	BlockName(String),
	ParenOpen,
	Separator,
	ParenClose,
	Num(BigInt),
	Add,
	Sub,
	Mult,
	Div,
	Mod,
	And,
	Or,
	Xor,
	BitshiftLeft,
	BitshiftRight,
	Not,
	Factorial
}

#[derive(Debug)]
pub enum ParseError {
	NotANumber,
	DisallowedChar(char),
	UnclosedBitShift(char)
}
impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			ParseError::DisallowedChar(c) => write!(f, "Character '{}' neither a number nor a valid function name", c),
			ParseError::UnclosedBitShift(c) => write!(f, "Character '{}' isn't followed by another '{}'.\n\
														  Looks like a failed attempt to bitshift.", c, c),
			_ => write!(f, "{}", self.description()),
		}
	}
}
impl std::error::Error for ParseError {
	fn description(&self) -> &str {
		match *self {
			ParseError::NotANumber => "Not a number",
			ParseError::DisallowedChar(_) => "A character you used was not allowed",
			ParseError::UnclosedBitShift(_) => "A < or > wasn't followed by another one, which is the way to bitshift"
		}
	}
}

pub fn parse(input: &str) -> Result<Vec<Token>, ParseError> {
	let mut output = Vec::new();
	let mut buffer = String::new();

	macro_rules! join_minus {
		($num:expr) => {
			if output.last() == Some(&Token::Sub) {
				match if output.len() >= 2 {
						Some(&output[output.len() - 2])
					} else {
						None
					} {
					Some(&Token::Num(_)) => (),
					Some(&Token::ParenClose) => (),
					_ => {
						output.pop();
						$num = -$num;
					}
				}
			}
		}
	}
	macro_rules! flush {
		() => {
			if !buffer.is_empty() {
				let mut num: BigInt = mem::replace(&mut buffer, String::new())
											.parse()
											.map_err(|_| ParseError::NotANumber)?;
				join_minus!(num);
				output.push(Token::Num(num));
			}
		}
	}

	let mut chars = input.chars().enumerate();
	while let Some((i, c)) = chars.next() {
		let token = match c {
			' ' => continue,
			',' => Some(Token::Separator),
			')' => Some(Token::ParenClose),
			'+' => Some(Token::Add),
			'-' => Some(Token::Sub),
			'*' => Some(Token::Mult),
			'/' => Some(Token::Div),
			'%' => Some(Token::Mod),
			'&' => Some(Token::And),
			'|' => Some(Token::Or),
			'^' => Some(Token::Xor),
			'<' => {
				if chars.next() != Some((i+1, '<')) {
					return Err(ParseError::UnclosedBitShift('<'));
				}
				Some(Token::BitshiftLeft)
			},
			'>' => {
				if chars.next() != Some((i+1, '>')) {
					return Err(ParseError::UnclosedBitShift('>'));
				}
				Some(Token::BitshiftRight)
			},
			'~' => Some(Token::Not),
			'!' => Some(Token::Factorial),
			_   => None
		};

		if let Some(token) = token {
			flush!();
			output.push(token);
		} else if c == '(' {
			if !buffer.is_empty() {
				match buffer.parse::<BigInt>() {
					Ok(mut num) => {
						join_minus!(num);
						output.push(Token::Num(num));
						output.push(Token::Mult);
					},
					Err(_) => {
						output.push(Token::BlockName(buffer));
					}
				};
				buffer = String::new();
			}
			output.push(Token::ParenOpen);
		} else {
			let code = c as u32;
			if (code < '0' as u32 || code > '9' as u32) &&
				(code < 'a' as u32 || code > 'z' as u32) &&
				(code < 'A' as u32 || code > 'Z' as u32) {

				return Err(ParseError::DisallowedChar(c))
			} else {
				buffer.push(c);
			}
		}
	}

	flush!();

	Ok(output)
}
