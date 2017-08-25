use bigdecimal::BigDecimal;
use parser::{Token, ParseError};
use std::collections::HashMap;
use std::iter::Peekable;
use std::{self, fmt, mem};

/// An error when calculating
#[derive(Debug)]
pub enum CalcError {
	DivideByZero,
	ExpectedEOF(Token),
	IncorrectArguments(usize, usize),
	InvalidSyntax,
	ParseError(ParseError),
	SeparatorInDef,
	TooLarge,
	UnclosedParen,
	UnknownFunction(String),
	UnknownVariable(String)
}
impl fmt::Display for CalcError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			CalcError::ExpectedEOF(ref found) => write!(f, "Expected EOF, found {}", found),
			CalcError::IncorrectArguments(expected, received) =>
				write!(f, "Incorrect amount of arguments (Expected {}, got {})", expected, received),
			CalcError::ParseError(ref error) => write!(f, "{}", error),
			CalcError::UnknownFunction(ref name) =>
				write!(f, "Unknown function \"{}\"\nHint: Cannot assume multiplication of variables because of ambiguity", name),
			CalcError::UnknownVariable(ref name) => write!(f, "Unknown variable \"{}\"", name),
			_ => write!(f, "{}", self.description())
		}
	}
}
impl std::error::Error for CalcError {
	fn description(&self) -> &str {
		match *self {
			CalcError::DivideByZero => "Cannot divide by zero",
			CalcError::ExpectedEOF(_) => "Expected EOF",
			CalcError::IncorrectArguments(..) => "Incorrect amount of arguments",
			CalcError::InvalidSyntax => "Invalid syntax",
			CalcError::ParseError(ref error)  => error.description(),
			CalcError::SeparatorInDef => "A function definition cannot have multiple arguments",
			CalcError::TooLarge => "You can only do this operation on smaller numbers",
			CalcError::UnclosedParen => "Unclosed parenthensis",
			CalcError::UnknownFunction(_) => "Unknown function",
			CalcError::UnknownVariable(_) => "Unknown variable"
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

/// A Context for `calculate` to pass around to all its sub-functions
pub struct Context<'a, I: Iterator<Item = Token>> {
	/// The tokens gotten by the parser
	pub tokens: Peekable<I>,
	toplevel: bool,
	/// A reference to a map of variables
	pub variables: &'a mut HashMap<String, BigDecimal>,
	/// A reference to a map of functions
	pub functions: &'a mut HashMap<String, Vec<Token>>
}

/// Calculates the result in a recursive descent fashion
pub fn calculate<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	let expr1 = calc_level2(context)?;

	if let Some(&Token::Xor) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calculate(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigDecimal::from(primitive1 ^ primitive2));
	}

	match context.tokens.peek() {
		Some(&Token::ParenClose) |
		Some(&Token::Separator)
		if !context.toplevel => Ok(expr1),

		Some(_) => Err(CalcError::ExpectedEOF(context.tokens.next().unwrap())),
		None => Ok(expr1)
	}
}
fn calc_level2<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	let expr1 = calc_level3(context)?;

	if let Some(&Token::Or) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level2(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigDecimal::from(primitive1 | primitive2));
	}

	Ok(expr1)
}
fn calc_level3<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	let expr1 = calc_level4(context)?;

	if let Some(&Token::And) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level3(context)?;

		use num::ToPrimitive;
		let primitive1 = to_primitive!(expr1, to_i64);
		let primitive2 = to_primitive!(expr2, to_i64);

		return Ok(BigDecimal::from(primitive1 & primitive2));
	}

	Ok(expr1)
}
fn calc_level4<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	let expr1 = calc_level5(context)?;

	if let Some(&Token::BitshiftLeft) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level4(context)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		use num::bigint::ToBigInt;
		return Ok(BigDecimal::new(expr1.to_bigint().unwrap() << primitive2, 0));
	} else if let Some(&Token::BitshiftRight) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level4(context)?;

		use num::ToPrimitive;
		let primitive2 = to_primitive!(expr2, to_usize);

		use num::bigint::ToBigInt;
		return Ok(BigDecimal::new(expr1.to_bigint().unwrap() >> primitive2, 0));
	}

	Ok(expr1)
}
fn calc_level5<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
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
fn calc_level6<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	let expr1 = calc_level7(context, None)?;

	if let Some(&Token::Mult) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level6(context)?;

		return Ok(expr1 * expr2);
	} else if let Some(&Token::Div) = context.tokens.peek() {
		context.tokens.next();
		let expr2 = calc_level6(context)?;

		use num::Zero;
		if expr2.is_zero() {
			return Err(CalcError::DivideByZero);
		}

		return Ok(expr1 / expr2);
	}

	Ok(expr1)
}
fn calc_level7<I: Iterator<Item = Token>>(context: &mut Context<I>, name: Option<String>) -> Result<BigDecimal, CalcError> {
	if let Some(&Token::ParenOpen) = context.tokens.peek() {
		context.tokens.next();
		let toplevel = mem::replace(&mut context.toplevel, false);

		let mut args = vec![calculate(context)?];

		while let Some(&Token::Separator) = context.tokens.peek() {
			context.tokens.next();
			args.push(calculate(context)?);
		}
		context.toplevel = toplevel;
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
					args[0] = BigDecimal::from(primitive1.pow(primitive2));
				},
				_ => {
					let tokens = match context.functions.get(&name) {
						Some(tokens) => tokens.clone(),
						None => return Err(CalcError::UnknownFunction(name))
					};
					let len = args.len();
					for (i, arg) in args.into_iter().enumerate() {
						let mut name = String::with_capacity(2);
						name.push('$');
						name.push_str(&(i + 1).to_string());
						context.variables.insert(name, arg);
					}
					let val = calculate(&mut Context {
						tokens: tokens.into_iter().peekable(),
						toplevel: false,
						variables: &mut context.variables,
						functions: &mut context.functions
					});
					for i in 1..len+1 {
						let mut name = String::with_capacity(2);
						name.push('$');
						name.push_str(&i.to_string());
					}
					return val;
				}
			}
		} else {
			usage!(1);
		}

		return Ok(args.remove(0));
	} else if name.is_none() {
		if let Some(&Token::BlockName(_)) = context.tokens.peek() {
			// Really ugly code, but we need to know the type *before* we walk out on it
			if let Some(Token::BlockName(name)) = context.tokens.next() {
				return calc_level7(context, Some(name));
			}
		}
	}

	Ok(calc_level8(context)?)
}
fn calc_level8<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	if let Some(&Token::Not) = context.tokens.peek() {
		context.tokens.next();
		use num::ToPrimitive;
		let expr = get_number(context)?;
		let primitive = to_primitive!(expr, to_i64);

		return Ok(BigDecimal::from(!primitive));
	}

	let expr = get_number(context)?;
	if let Some(&Token::Factorial) = context.tokens.peek() {
		return Ok(factorial(expr));
	}
	Ok(expr)
}
fn factorial(num: BigDecimal) -> BigDecimal {
	use num::One;
	let one = BigDecimal::one();
	if num == one { one } else { num.clone() + factorial(num - one) }
}
fn get_number<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
	match context.tokens.next() {
		Some(Token::Num(num)) => Ok(num),
		Some(Token::VarAssign(name)) => {
			if let Some(&Token::ParenOpen) = context.tokens.peek() {
				context.tokens.next();
				let mut fn_tokens = Vec::new();

				loop {
					let token = match context.tokens.next() {
						Some(Token::Separator) => return Err(CalcError::SeparatorInDef),
						Some(token) => token,
						None => return Err(CalcError::UnclosedParen)
					};
					let exit = token == Token::ParenClose;
					fn_tokens.push(token);

					if exit { break; }
				}

				context.functions.insert(name, fn_tokens);
			} else {
				let val = calculate(context)?;
				context.variables.insert(name, val);
			}
			use num::Zero;
			Ok(BigDecimal::zero())
		},
		Some(Token::VarGet(name)) => {
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
