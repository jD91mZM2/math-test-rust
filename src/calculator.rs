use num::BigInt;
use parser::Token;
use std::collections::HashMap;
use std::iter::Peekable;
use std::{self, fmt, mem};

#[derive(Debug)]
pub enum CalcError {
	UnknownFunction(String),
	UnknownVariable(String),
	IncorrectArguments(usize, usize),
	TooLarge,
	InvalidSyntax,
	UnclosedParen
}
impl fmt::Display for CalcError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			CalcError::UnknownFunction(ref name) => write!(f, "Unknown function \"{}\"\n\
															Hint: Cannot assume multiplication because of ambiguity.", name),
			CalcError::UnknownVariable(ref name) => write!(f, "Unknown variable \"{}\"", name),
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
			CalcError::UnknownVariable(_) => "Unknown variable",
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
			None => return Err(CalcError::TooLarge)
		}
	}
}

pub struct Context<'a, I: Iterator<Item = Token>> {
	pub tokens: Peekable<I>,
	pub variables: &'a mut HashMap<String, BigInt>
}

pub fn calculate<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level2(context)?;

	if let Some(&Token::Xor) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calculate(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 ^ primitive2));
	}

	Ok(expr1)
}
pub fn calc_level2<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level3(context)?;

	if let Some(&Token::Or) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level2(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 | primitive2));
	}

	Ok(expr1)
}
pub fn calc_level3<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level4(context)?;

	if let Some(&Token::And) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level3(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigInt::from(primitive1 & primitive2));
	}

	Ok(expr1)
}
pub fn calc_level4<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level5(context)?;

	if let Some(&Token::BitshiftLeft) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level4(context)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		return Ok(expr1 << primitive2);
	} else if let Some(&Token::BitshiftRight) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level4(context)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		return Ok(expr1 >> primitive2);
	}

	Ok(expr1)
}
pub fn calc_level5<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level6(context)?;

	if let Some(&Token::Add) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level5(context)?;

		return Ok(expr1 + expr2);
	} else if let Some(&Token::Sub) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level5(context)?;

		return Ok(expr1 - expr2);
	}

	Ok(expr1)
}
fn calc_level6<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	let expr1 = calc_level7(context, None)?;

	if let Some(&Token::Mult) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level6(context)?;

		return Ok(expr1 * expr2);
	} else if let Some(&Token::Div) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level6(context)?;

		return Ok(expr1 / expr2);
	}

	Ok(expr1)
}
fn calc_level7<I: Iterator<Item = Token>>(context: &mut Context<I>, name: Option<String>) -> Result<BigInt, CalcError> {
	if let Some(&Token::ParenOpen) = context.tokens.peek() {
		context.tokens.next();
		let mut args = vec![calculate(context)?];

		while let Some(&Token::Separator) = context.tokens.peek() {
			context.tokens.next();
			args.push(calculate(context)?);
		}
		if Some(Token::ParenClose) != context.tokens.next() {
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
		if let Some(&Token::BlockName(_)) = context.tokens.peek() {
			// Really ugly code, but we need to know the type *before* we walk out on it.
			if let Some(Token::BlockName(name)) = context.tokens.next() {
				return calc_level7(context, Some(name));
			}
		}
	}

	Ok(calc_level8(context)?)
}
fn calc_level8<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	if let Some(&Token::Not) = context.tokens.peek() {
		context.tokens.next();
		use num::ToPrimitive;
		let expr = get_number(context)?;
		let primitive = to_primitive!(expr, to_i64);

		return Ok(BigInt::from(!primitive));
	}

	let expr = get_number(context)?;
	if let Some(&Token::Factorial) = context.tokens.peek() {
		return Ok(factorial(expr));
	}
	Ok(expr)
}
fn factorial(num: BigInt) -> BigInt {
	use num::One;
	let one = BigInt::one();
	if num == one { one } else { num.clone() + factorial(num - 1) }
}
fn get_number<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigInt, CalcError> {
	match context.tokens.next() {
		Some(Token::Num(num)) => Ok(num),
		Some(Token::VarName(name)) => {
			let val = calculate(context)?;
			context.variables.insert(name, val.clone());
			Ok(val)
		},
		Some(Token::VarVal(name)) => {
			Ok(
				match context.variables.get(&name) {
					Some(val) => val.clone(),
					None => return Err(CalcError::UnknownVariable(name))
				}
			)
		},
		_ => Err(CalcError::InvalidSyntax)
	}
}
