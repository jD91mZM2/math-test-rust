use std::{self, fmt};
use std::iter::Peekable;
use num::BigInt;
use parser::Token;

#[derive(Debug)]
pub enum CalcError {
	UnknownFunction(String),
	IncorrectArguments(usize, usize),
	BitwiseTooLarge,
	InvalidSyntax,
	UnclosedParen
}
impl fmt::Display for CalcError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			CalcError::UnknownFunction(ref name) => write!(f, "Unknown function \"{}\"", name),
			CalcError::IncorrectArguments(expected, received) =>
				write!(f, "Incorrect amount of arguments (Expected {}, got {})", expected, received),
			_ => write!(f, "{}", self.description())
		}
	}
}
impl std::error::Error for CalcError {
	fn description(&self) -> &str {
		match *self {
			CalcError::UnknownFunction(_) => "Unknown function",
			CalcError::IncorrectArguments(..) => "Incorrect amount of arguments!",
			CalcError::BitwiseTooLarge => "You can only do bitwise operations on smaller numbers",
			CalcError::InvalidSyntax => "Invalid syntax",
			CalcError::UnclosedParen => "Unclosed parenthensis"
		}
	}
}

macro_rules! to_primitive {
	($expr:expr, $type:ident) => {
		match $expr.$type() {
			Some(primitive) => primitive,
			None => return Err(CalcError::BitwiseTooLarge),
		}
	}
}

pub fn calculate<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level2(tokens)?;

	if tokens.peek() == Some(&Token::Xor) {
		tokens.next();
		let expr2 = calculate(tokens)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 ^ primitive2));
	}

	Ok(expr1)
}
pub fn calc_level2<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level3(tokens)?;

	if tokens.peek() == Some(&Token::Or) {
		tokens.next();
		let expr2 = calc_level2(tokens)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 | primitive2));
	}

	Ok(expr1)
}
pub fn calc_level3<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level4(tokens)?;

	if tokens.peek() == Some(&Token::And) {
		tokens.next();
		let expr2 = calc_level3(tokens)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 & primitive2));
	}

	Ok(expr1)
}
pub fn calc_level4<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level5(tokens)?;

	if tokens.peek() == Some(&Token::BitshiftLeft) {
		tokens.next();
		let expr2 = calc_level4(tokens)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		return Ok(expr1 << primitive2);
	} else if tokens.peek() == Some(&Token::BitshiftRight) {
		tokens.next();
		let expr2 = calc_level4(tokens)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		return Ok(expr1 >> primitive2);
	}

	Ok(expr1)
}
pub fn calc_level5<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level6(tokens)?;

	if tokens.peek() == Some(&Token::Add) {
		tokens.next();
		let expr2 = calc_level5(tokens)?;

		return Ok(expr1 + expr2);
	} else if tokens.peek() == Some(&Token::Sub) {
		tokens.next();
		let expr2 = calc_level5(tokens)?;

		return Ok(expr1 - expr2);
	}

	Ok(expr1)
}
fn calc_level6<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level7(tokens, None)?;

	if tokens.peek() == Some(&Token::Mult) {
		tokens.next();
		let expr2 = calc_level6(tokens)?;

		return Ok(expr1 * expr2);
	} else if tokens.peek() == Some(&Token::Div) {
		tokens.next();
		let expr2 = calc_level6(tokens)?;

		return Ok(expr1 / expr2);
	}

	Ok(expr1)
}
fn calc_level7<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>, name: Option<String>) -> Result<BigInt, CalcError> {
	if tokens.peek() == Some(&Token::ParenOpen) {
		tokens.next();
		let mut args = vec![calculate(tokens)?];

		while let Some(&Token::Separator) = tokens.peek() {
			tokens.next();
			args.push(calculate(tokens)?);
		}
		if tokens.next() != Some(Token::ParenClose) {
			return Err(CalcError::UnclosedParen);
		}

		macro_rules! usage {
			($expected:expr) => {
				if args.len() != $expected {
					return Err(CalcError::IncorrectArguments($expected, args.len()));
				}
			}
		}

		if let Some(name) = name {
			match &*name {
				"abs" => {
					use num::Signed;
					args[0] = args[0].abs();
				},
				"pow" => {
					usage!(2);
					use num::ToPrimitive;
					let primitive1 = to_primitive!(args[0], to_i64);
					let primitive2 = to_primitive!(args[1], to_u32);
					args[0] = BigInt::from(primitive1.pow(primitive2));
				}
				_ => {
					return Err(CalcError::UnknownFunction(name));
				}
			}
		} else {
			usage!(1);
		}

		return Ok(args.remove(0));
	} else if name.is_none() {
		if let Some(&Token::BlockName(_)) = tokens.peek() {
			// Really ugly code, but we need to know the type *before* we walk out on it.
			if let Some(Token::BlockName(name)) = tokens.next() {
				return calc_level7(tokens, Some(name));
			}
		}
	}

	Ok(get_number(tokens)?)
}
fn get_number<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	if let Some(Token::Num(num)) = tokens.next() {
		return Ok(num);
	}
	Err(CalcError::InvalidSyntax)
}
