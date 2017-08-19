use std::{self, fmt, mem};
use num::BigInt;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
	Num(BigInt),
	Add,
	Sub,
	Mult,
	Div,
	And,
	Or,
	Xor,
	BitshiftLeft,
	BitshiftRight,
	Block(Option<String>, Vec<Token>)
}

#[derive(Debug)]
pub enum ParseError {
	NotANumber,
	TooManyParens,
	ErrUnclosed,
	ErrUnmatched,
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
			ParseError::TooManyParens => "Too many levels of parenthensis",
			ParseError::ErrUnclosed => "Unclosed ( - No closing )",
			ParseError::ErrUnmatched => "Unmatched ) -  No matching (",
			ParseError::DisallowedChar(_) => "A character you used was not allowed",
			ParseError::UnclosedBitShift(_) => "A < or > wasn't followed by another one, which is the way to bitshift"
		}
	}
}

pub fn parse(input: &str) -> Result<Vec<Token>, ParseError> {
	let mut output = Vec::new();
	let mut buffer = String::new();

	macro_rules! flush {
		() => {
			if !buffer.is_empty() {
				let mut num: BigInt = mem::replace(&mut buffer, String::new())
											.parse()
											.map_err(|_| ParseError::NotANumber)?;
				if output.last() == Some(&Token::Sub) {
					output.pop();
					num = -num;
				}
				output.push(Token::Num(num));
			}
		}
	}

	let mut chars = input.chars().enumerate();
	while let Some((i, c)) = chars.next() {
		let token = match c {
			'+' => Some(Token::Add),
			'-' => Some(Token::Sub),
			'*' => Some(Token::Mult),
			'/' => Some(Token::Div),
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
			_   => None
		};

		if let Some(token) = token {
			flush!();
			output.push(token);
		} else if c == '(' {
			let name = mem::replace(&mut buffer, String::new());
			let name = match name.parse() {
				Ok(num) => {
					output.push(Token::Num(num));
					output.push(Token::Mult);
					None
				},
				Err(_) => {
					if name.is_empty() {
						None
					} else {
						Some(name)
					}
				}
			};

			let end;
			let mut parens = 0u8;
			loop {
				if let Some((i, c)) = chars.next() {
					if c == '(' {
						match parens.checked_add(1) {
							Some(new) => parens = new,
							None => return Err(ParseError::TooManyParens),
						}
					} else if c == ')' {
						if parens <= 1 {
							end = i;
							break;
						}

						parens -= 1;
					}
				} else {
					return Err(ParseError::ErrUnclosed)
				}
			}

			output.push(Token::Block(name, parse(&input[i+1..end])?));
		} else if c == ')' {
			return Err(ParseError::ErrUnmatched);
		} else {
			let code = c as u32;
			if (code < '0' as u32 || code > '9' as u32) &&
				(code < 'a' as u32 || code > 'z' as u32) &&
				(code < 'A' as u32 || code > 'Z' as u32) &&
				(c != ' ') {

				return Err(ParseError::DisallowedChar(c))
			} else {
				buffer.push(c);
			}
		}
	}

	flush!();

	Ok(output)
}
