use std::{self, fmt, mem};
use num::BigInt;
use parser::Token;

#[derive(Debug)]
pub enum CalcError {
	UnknownFunction(String),
	BitwiseTooLarge,
	InvalidSyntax
}
impl fmt::Display for CalcError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use std::error::Error;
		match *self {
			CalcError::UnknownFunction(ref name) => write!(f, "Unknown function \"{}\"", name),
			_ => write!(f, "{}", self.description())
		}
	}
}
impl std::error::Error for CalcError {
	fn description(&self) -> &str {
		match *self {
			CalcError::UnknownFunction(_) => "Unknown function",
			CalcError::BitwiseTooLarge => "You can only do bitwise operations on smaller numbers",
			CalcError::InvalidSyntax => "Invalid syntax"
		}
	}
}

pub fn calculate(mut tokens: Vec<Token>) -> Result<BigInt, CalcError> {
	for token in &mut tokens {
		if let Token::Block(..) = *token {
			// Kinda ugly. Necessary to check type before mem::replacing out of it...
			if let Token::Block(name, tokens) = mem::replace(token, Token::Empty) {
				let mut num = calculate(tokens)?;

				if let Some(name) = name {
					use num::Signed;
					match &*name {
						"abs" => num = num.abs(),
						_ => return Err(CalcError::UnknownFunction(name))
					}
				}

				*token = Token::Num(num);
			}
		}
	}

	calculate_operators(&mut tokens, &[Token::Mult, Token::Div, Token::Mod])?;
	calculate_operators(&mut tokens, &[Token::Add, Token::Sub])?;
	calculate_operators(&mut tokens, &[Token::BitshiftLeft, Token::BitshiftRight])?;
	calculate_operators(&mut tokens, &[Token::And])?;
	calculate_operators(&mut tokens, &[Token::Xor])?;
	calculate_operators(&mut tokens, &[Token::Or])?;

	if let Some(Token::Num(num)) = tokens.pop() {
		Ok(num)
	} else {
		Err(CalcError::InvalidSyntax)
	}
}

pub fn calculate_operators(tokens: &mut Vec<Token>, operators: &[Token]) -> Result<(), CalcError> {
	if tokens.len() <= 2 {
		return Ok(());
	}
	for i in 0..(tokens.len() - 2) {
		if !operators.contains(&tokens[i + 1]) {
			continue;
		}
		match (&tokens[i], &tokens[i + 2]) {
			(&Token::Num(_), &Token::Num(_)) => (),
			_ => continue
		}

		let num1 = match mem::replace(&mut tokens[i], Token::Empty) {
			Token::Num(num) => num,
			_ => unreachable!()
		};
		let num2 = match mem::replace(&mut tokens[i + 2], Token::Empty) {
			Token::Num(num) => num,
			_ => unreachable!()
		};

		tokens[i + 1] = Token::Num(match tokens[i + 1] {
			Token::Add  => num1 + num2,
			Token::Sub  => num1 - num2,
			Token::Mult => num1 * num2,
			Token::Div  => num1 / num2,
			Token::BitshiftLeft => {
				use num::ToPrimitive;
				let primitive2 = match num2.to_usize() {
					Some(primitive) => primitive,
					None => return Err(CalcError::BitwiseTooLarge)
				};

				num1 << primitive2
			},
			Token::BitshiftRight => {
				use num::ToPrimitive;
				let primitive2 = match num2.to_usize() {
					Some(primitive) => primitive,
					None => return Err(CalcError::BitwiseTooLarge)
				};

				num1 >> primitive2
			},
			_ => {
				use num::ToPrimitive;
				let primitive1 = match num1.to_i64() {
					Some(primitive) => primitive,
					None => return Err(CalcError::BitwiseTooLarge)
				};
				let primitive2 = match num2.to_i64() {
					Some(primitive) => primitive,
					None => return Err(CalcError::BitwiseTooLarge)
				};

				match tokens[i + 1] {
					Token::And  => BigInt::from(primitive1 & primitive2),
					Token::Or   => BigInt::from(primitive1 | primitive2),
					Token::Xor  => BigInt::from(primitive1 ^ primitive2),
					_ => unreachable!()
				}
			}
		});

		tokens.swap(i + 1, i + 2); // To move the real value towards AFTER the two empties.
	}

	tokens.retain(|item| *item != Token::Empty);

	Ok(())
}
