use std::{self, fmt, mem};
use std::iter::Peekable;
use num::BigInt;
use parser::Token;

#[derive(Debug)]
pub enum CalcError {
	UnknownFunction(String),
	IncorrectArguments(usize, usize),
	TooLarge,
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
			CalcError::TooLarge => "You can only do this operation on smaller numbers",
			CalcError::InvalidSyntax => "Invalid syntax",
			CalcError::UnclosedParen => "Unclosed parenthensis"
		}
	}
}

macro_rules! to_primitive {
	($expr:expr, $type:ident) => {
		match $expr.$type() {
			Some(primitive) => primitive,
			None => return Err(CalcError::TooLarge),
		}
	}
}

pub fn calculate<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level2(tokens)?;

	if let Some(&Token::Xor) = tokens.peek() {
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

	if let Some(&Token::Or) = tokens.peek() {
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

	if let Some(&Token::And) = tokens.peek() {
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

	if let Some(&Token::BitshiftLeft) = tokens.peek() {
		tokens.next();
		let expr2 = calc_level4(tokens)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		return Ok(expr1 << primitive2);
	} else if let Some(&Token::BitshiftRight) = tokens.peek() {
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

	if let Some(&Token::Add) = tokens.peek() {
		tokens.next();
		let expr2 = calc_level5(tokens)?;

		return Ok(expr1 + expr2);
	} else if let Some(&Token::Sub) = tokens.peek() {
		tokens.next();
		let expr2 = calc_level5(tokens)?;

		return Ok(expr1 - expr2);
	}

	Ok(expr1)
}
fn calc_level6<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level7(tokens, None)?;

	if let Some(&Token::Mult) = tokens.peek() {
		tokens.next();
		let expr2 = calc_level6(tokens)?;

		return Ok(expr1 * expr2);
	} else if let Some(&Token::Div) = tokens.peek() {
		tokens.next();
		let expr2 = calc_level6(tokens)?;

		return Ok(expr1 / expr2);
	}

	Ok(expr1)
}
fn calc_level7<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>, name: Option<String>) -> Result<BigInt, CalcError> {
	if let Some(&Token::ParenOpen) = tokens.peek() {
		tokens.next();
		let mut args = vec![calculate(tokens)?];

		while let Some(&Token::Separator) = tokens.peek() {
			tokens.next();
			args.push(calculate(tokens)?);
		}
		if Some(Token::ParenClose) != tokens.next() {
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
					usage!(1);
					use num::Signed;
					args[0] = args[0].abs();
				},
				"pow" => {
					usage!(2);
					use num::ToPrimitive;
					let primitive1 = to_primitive!(args[0], to_i64);
					let primitive2 = to_primitive!(args[1], to_u32);
					args[0] = BigInt::from(primitive1.pow(primitive2));
				},
				"binary" => {
					usage!(1);
					use num::{Zero, One};
					let (zero, one, ten) = (BigInt::zero(), BigInt::one(), BigInt::from(10));
					let old = mem::replace(&mut args[0], zero.clone());

					let mut i = 0;
					let mut old_clone = old.clone();
					while old_clone > zero {
						old_clone = old_clone >> 1;
						i += 1;
					}
					while old > zero {
						let new = old.clone() >> i;
						args[0] = args[0].clone() * ten.clone();
						if new % 2 == one {
							args[0] = args[0].clone() + one.clone();
						}
						if i == 0 {
							break;
						}
						i -= 1;
					}
				},
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

	Ok(calc_level8(tokens)?)
}
fn calc_level8<I: Iterator<Item = Token>>(tokens: &mut Peekable<I>) -> Result<BigInt, CalcError> {
	if let Some(&Token::Not) = tokens.peek() {
		tokens.next();
		use num::ToPrimitive;
		let expr = get_number(tokens)?;
		let primitive = to_primitive!(expr, to_i64);

		return Ok(BigInt::from(!primitive));
	}

	let expr = get_number(tokens)?;
	if let Some(&Token::Factorial) = tokens.peek() {
		return Ok(factorial(expr));
	}
	Ok(expr)
}
fn factorial(num: BigInt) -> BigInt {
	use num::One;
	let one = BigInt::one();
	if num == one { one } else { num.clone() + factorial(num - 1) }
}
fn get_number<I: Iterator<Item = Token>>(tokens: &mut I) -> Result<BigInt, CalcError> {
	if let Some(Token::Num(num)) = tokens.next() {
		return Ok(num);
	}
	Err(CalcError::InvalidSyntax)
}
