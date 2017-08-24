use std::{self, fmt, mem};
use bigdecimal::BigDecimal;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Token {
	BlockName(String),
	ParenOpen,
	Separator,
	ParenClose,
	VarAssign(String),
	VarGet(String),
	Num(BigDecimal),
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

impl fmt::Display for Token {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		match *self {
			Token::BlockName(ref name) => write!(f, "\"{}\"", name),
			Token::ParenOpen => write!(f, "("),
			Token::Separator => write!(f, ","),
			Token::ParenClose => write!(f, ")"),
			Token::VarAssign(ref name) => write!(f, "Variable assignment \"{}\"", name),
			Token::VarGet(ref name) => write!(f, "Variable \"{}\"", name),
			Token::Num(ref num) => write!(f, "Number {}", num),
			Token::Add => write!(f, "Plus (+)"),
			Token::Sub => write!(f, "Minus (-)"),
			Token::Mult => write!(f, "Times (*)"),
			Token::Div => write!(f, "Division symbol (/)"),
			Token::Mod => write!(f, "Modulus (%)"),
			Token::And => write!(f, "Bitwise AND (&)"),
			Token::Or => write!(f, "Bitwise OR (|)"),
			Token::Xor => write!(f, "Bitwise XOR (^)"),
			Token::BitshiftLeft => write!(f, "Bitshift left (<<)"),
			Token::BitshiftRight => write!(f, "Bitshift right (>>)"),
			Token::Not => write!(f, "Bitwise NOT (~)"),
			Token::Factorial => write!(f, "Factorial (!)")
		}
	}
}

#[derive(Debug)]
pub enum ParseError {
	DisallowedChar(char),
	DisallowedDecimal,
	DisallowedVariable(String),
	UnclosedBitShift(char)
}
impl fmt::Display for ParseError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			ParseError::DisallowedChar(c) => write!(f, "Character '{}' neither a number nor a valid letter \
														in a function or variable name.", c),
			ParseError::UnclosedBitShift(c) => write!(f, "Character '{}' isn't followed by another '{}'.\n\
														  Looks like a failed attempt to bitshift.", c, c),
			ParseError::DisallowedVariable(ref var) => write!(f, "\"{}\" is not a valid variable name.", var),
			_ => write!(f, "{}", self.description())
		}
	}
}
impl std::error::Error for ParseError {
	fn description(&self) -> &str {
		match *self {
			ParseError::DisallowedChar(_) => "A character you used was not allowed",
			ParseError::DisallowedDecimal => "You may only use whole numbers in this context",
			ParseError::DisallowedVariable(_) => "Not a valid variable name.",
			ParseError::UnclosedBitShift(_) => "A < or > wasn't followed by another one, which is the way to bitshift"
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

					Some(&Token::Num(_)) |
					Some(&Token::ParenClose) |
					Some(&Token::VarGet(_)) => (),
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
				match parse_num(&buffer) {
					Ok(mut num) => {
						prepare_num!(num);
						output.push(Token::Num(num));
					},
					Err(_) => {
						prepare_var!();
						output.push(Token::VarGet(buffer));
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
				match parse_num(&buffer) {
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
			if buffer.is_empty() || is_num(&buffer) || buffer.starts_with('$') {
				return Err(ParseError::DisallowedVariable(buffer));
			}
			output.push(Token::VarAssign(buffer));
		} else {
			let code = c as u32;
			let was_num = is_num(&buffer);
			let old_len = buffer.len();

			buffer.push(c);
			let num = is_num(&buffer);
			if num ||
				(code >= 'a' as u32 && code <= 'z' as u32) ||
				(code >= 'A' as u32 && code <= 'Z' as u32) ||
				(c == '_' || c == '$') {

				if was_num && !num && !is_num_prefix(&buffer) {;
					buffer.drain(old_len..);
					flush!();
					buffer.push(c);
				}
			} else {
				if c == '.' {
					return Err(ParseError::DisallowedDecimal);
				}
				buffer.drain(old_len..);
				return Err(ParseError::DisallowedChar(c));
			}
		}
	}

	flush!();

	Ok(output)
}

fn parse_num(num: &str) -> Result<BigDecimal, ::bigdecimal::ParseBigDecimalError> {
	use num::{BigInt, Num};
	if num.starts_with("0x") {
		return Ok(BigDecimal::new(BigInt::from_str_radix(&num[2..], 16)?, 0));
	} else if num.starts_with("0o") {
		return Ok(BigDecimal::new(BigInt::from_str_radix(&num[2..], 8)?, 0));
	} else if num.starts_with("0b") {
		return Ok(BigDecimal::new(BigInt::from_str_radix(&num[2..], 2)?, 0));
	}

	num.parse()
}
fn is_num_prefix(prefix: &str) -> bool {
	prefix == "0x" || prefix == "0o" || prefix == "0b"
}
fn is_num(mut num: &str) -> bool {
	let mut radix = 10;
	if num.starts_with("0x") {
		num = &num[2..];
		radix = 16;
	} else if num.starts_with("0o") {
		num = &num[2..];
		radix = 8;
	} else if num.starts_with("0b") {
		num = &num[2..];
		radix = 2;
	}
	!num.is_empty() && num.chars().all(|c| c.is_digit(radix) || (radix == 10 && c == '.'))
}
