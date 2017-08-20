use std::{self, fmt, mem};
use num::BigInt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
	BlockName(String),
	ParenOpen,
	Separator,
	ParenClose,
	VarName(String),
	VarVal(String),
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
	DisallowedChar(char),
	UnclosedBitShift(char),
	DisallowedVariable(String)
}
impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			ParseError::DisallowedChar(c) => write!(f, "Character '{}' neither a number nor a valid letter \
														in a function or variable name.", c),
			ParseError::UnclosedBitShift(c) => write!(f, "Character '{}' isn't followed by another '{}'.\n\
														  Looks like a failed attempt to bitshift.", c, c),
			ParseError::DisallowedVariable(ref var) => write!(f, "\"{}\" is not a valid variable name.", var),
		}
	}
}
impl std::error::Error for ParseError {
	fn description(&self) -> &str {
		match *self {
			ParseError::DisallowedChar(_) => "A character you used was not allowed",
			ParseError::UnclosedBitShift(_) => "A < or > wasn't followed by another one, which is the way to bitshift",
			ParseError::DisallowedVariable(_) => "Not a valid variable name."
		}
	}
}

pub fn parse(input: &str) -> Result<Vec<Token>, ParseError> {
	let mut output = Vec::new();
	let mut buffer = String::new();

	macro_rules! prepare_num {
		($num:expr) => {
			if let Some(&Token::Sub) = output.last() {
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
	macro_rules! prepare_var {
		() => {
			if let Some(&Token::Num(_)) = output.last() {
				output.push(Token::Mult);
			}
		}
	}
	macro_rules! flush {
		() => {
			if !buffer.is_empty() {
				let buffer = mem::replace(&mut buffer, String::new());
				match buffer.parse::<BigInt>() {
					Ok(mut num) => {
						prepare_num!(num);
						output.push(Token::Num(num));
					},
					Err(_) => {
						prepare_var!();
						output.push(Token::VarVal(buffer));
					}
				}
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
						prepare_num!(num);
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
		} else if c == '=' {
			let buffer = mem::replace(&mut buffer, String::new());
			if buffer.is_empty() || buffer.chars().all(|c| c.is_digit(10)) {
				return Err(ParseError::DisallowedVariable(buffer));
			}
			output.push(Token::VarName(buffer));
		} else {
			let code = c as u32;
			let digit = code >= '0' as u32 && code <= '9' as u32;
			if digit ||
				(code >= 'a' as u32 && code <= 'z' as u32) ||
				(code >= 'A' as u32 && code <= 'Z' as u32) ||
				c == '_' {

				if !digit && buffer.chars().all(|c| c.is_digit(10)) {
					flush!();
				}
				buffer.push(c);
			} else {
				return Err(ParseError::DisallowedChar(c))
			}
		}
	}

	flush!();

	Ok(output)
}
