use bigdecimal::BigDecimal;
use num::bigint::Sign;
use parser::{Token, ParseError};
use std::collections::HashMap;
use std::iter::Peekable;
use std::{self, mem};

/// An error when calculating
#[derive(Debug, Fail)]
pub enum CalcError {
    #[fail(display = "Cannot divide by zero")]
    DivideByZero,
    #[fail(display = "Expected EOF, found {}", _0)]
    ExpectedEOF(Token),
    #[fail(display = "Incorrect amount of arguments (Expected {}, got {})", _0, _1)]
    IncorrectArguments(usize, usize),
    #[fail(display = "Invalid syntax")]
    InvalidSyntax,
    #[fail(display = "You may only do this on positive numbers")]
    NotAPositive,
    #[fail(display = "Number must fit the range of a {} primitive", _0)]
    NotAPrimitive(&'static str),
    #[fail(display = "You may only do this on whole numbers")]
    NotAWhole,
    #[fail(display = "Parse error: {}", _0)]
    ParseError(#[cause] ParseError),
    #[fail(display = "A function definition cannot have multiple arguments")]
    SeparatorInDef,
    #[fail(display = "Too many levels deep. This could be an issue with endless recursion.")]
    TooDeep,
    #[fail(display = "Unclosed parentheses")]
    UnclosedParen,
    #[fail(display = "Unknown function \"{}\"\nHint: Cannot assume multiplication of variables because of ambiguity", _0)]
    UnknownFunction(String),
    #[fail(display = "Unknown variable \"{}\"", _0)]
    UnknownVariable(String)
}

macro_rules! to_primitive {
    ($expr:expr, $type:ident, $primitive:expr) => {
        match $expr.$type() {
            Some(primitive) => primitive,
            None => return Err(CalcError::NotAPrimitive($primitive))
        }
    }
}

/// A Context for `calculate` to pass around to all its sub-functions
pub struct Context<'a, I: Iterator<Item = Token>> {
    level: u8,

    /// The tokens gotten by the parser
    pub tokens: Peekable<I>,
    /// A reference to a map of variables
    pub variables: &'a mut HashMap<String, BigDecimal>,
    /// A reference to a map of functions
    pub functions: &'a mut HashMap<String, Vec<Token>>
}
impl<'a, I: Iterator<Item = Token>> Context<'a, I> {
    pub fn new(
        tokens: Peekable<I>,
        variables: &'a mut HashMap<String, BigDecimal>,
        functions: &'a mut HashMap<String, Vec<Token>>
    ) -> Self {

        Context {
            level: 0,
            tokens: tokens,
            variables: variables,
            functions: functions
        }
    }
}

/// Calculates the result in a recursive descent fashion
pub fn calculate<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    if context.level == std::u8::MAX {
        return Err(CalcError::TooDeep);
    }

    let expr1 = calc_level2(context)?;

    if let Some(&Token::Xor) = context.tokens.peek() {
        context.tokens.next();
        let expr2 = calculate(context)?;

        use num::ToPrimitive;
        let primitive1 = to_primitive!(expr1, to_i64, "i64");
        let primitive2 = to_primitive!(expr2, to_i64, "i64");

        return Ok(BigDecimal::from(primitive1 ^ primitive2));
    }

    match context.tokens.peek() {
        Some(&Token::ParenClose) |
        Some(&Token::Separator)
        if context.level != 0 => Ok(expr1),

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
        let primitive1 = to_primitive!(expr1, to_i64, "i64");
        let primitive2 = to_primitive!(expr2, to_i64, "i64");

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
        let primitive1 = to_primitive!(expr1, to_i64, "i64");
        let primitive2 = to_primitive!(expr2, to_i64, "i64");

        return Ok(BigDecimal::from(primitive1 & primitive2));
    }

    Ok(expr1)
}
fn calc_level4<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    let mut expr1 = calc_level5(context)?;

    loop {
        use num::bigint::ToBigInt;
        if let Some(&Token::BitshiftLeft) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level5(context)?;

            use num::ToPrimitive;
            let primitive2 = to_primitive!(expr2, to_usize, "usize");

            require_whole(&expr1)?;
            expr1 = BigDecimal::new(expr1.to_bigint().unwrap() << primitive2, 0);
        } else if let Some(&Token::BitshiftRight) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level5(context)?;

            use num::ToPrimitive;
            let primitive2 = to_primitive!(expr2, to_usize, "usize");

            require_whole(&expr1)?;
            expr1 = BigDecimal::new(expr1.to_bigint().unwrap() >> primitive2, 0);
        } else {
            break;
        }
    }

    Ok(expr1)
}
fn calc_level5<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    let mut expr1 = calc_level6(context)?;

    loop {
        if let Some(&Token::Add) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level6(context)?;

            expr1 += expr2;
        } else if let Some(&Token::Sub) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level6(context)?;

            expr1 -= expr2;
        } else {
            break;
        }
    }

    Ok(expr1)
}
fn calc_level6<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    let mut expr1 = calc_level7(context)?;

    loop {
        if let Some(&Token::Mul) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level7(context)?;

            expr1 *= expr2;
        } else if let Some(&Token::Div) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level7(context)?;

            use num::Zero;
            if expr2.is_zero() {
                return Err(CalcError::DivideByZero);
            }

            expr1 = expr1 / expr2;
        } else if let Some(&Token::Rem) = context.tokens.peek() {
            context.tokens.next();
            let expr2 = calc_level7(context)?;

            use num::Zero;
            if expr2.is_zero() {
                return Err(CalcError::DivideByZero);
            }

            use num::bigint::ToBigInt;
            expr1 = BigDecimal::new(expr1.to_bigint().unwrap() % expr2.to_bigint().unwrap(), 0);
        } else {
            break;
        }
    }

    Ok(expr1)
}
fn calc_level7<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    let expr1 = calc_level8(context)?;
    if let Some(&Token::Pow) = context.tokens.peek() {
        context.tokens.next();
        let expr2 = calc_level7(context)?; // Right associative

        return pow(expr1, expr2, None, 0);
    }
    Ok(expr1)
}
fn calc_level8<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    let expr = calc_level9(context)?;
    if let Some(&Token::Factorial) = context.tokens.peek() {
        context.tokens.next();

        return factorial(expr, None, 0);
    }
    Ok(expr)
}
fn calc_level9<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    if let Some(&Token::Not) = context.tokens.peek() {
        context.tokens.next();
        use num::ToPrimitive;
        let expr = calc_level9(context)?;
        let primitive = to_primitive!(expr, to_i64, "i64");

        return Ok(BigDecimal::from(!primitive));
    }

    Ok(calc_paren(context, None)?)
}
fn calc_paren<I: Iterator<Item = Token>>(context: &mut Context<I>, name: Option<String>) -> Result<BigDecimal, CalcError> {
    if let Some(&Token::ParenOpen) = context.tokens.peek() {
        context.tokens.next();

        let mut args = Vec::new();

        if let Some(&Token::ParenClose) = context.tokens.peek() {
        } else {
            context.level += 1;

            args.push(calculate(context)?);

            while let Some(&Token::Separator) = context.tokens.peek() {
                context.tokens.next();
                args.push(calculate(context)?);
            }

            context.level -= 1;
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
                    use num::Zero;
                    args[0] = pow(mem::replace(&mut args[0], BigDecimal::zero()), args.remove(1), None, 0)?;
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
                        level: context.level + 1,
                        variables: &mut context.variables,
                        functions: &mut context.functions
                    });
                    for i in 1..len+1 {
                        let mut name = String::with_capacity(2);
                        name.push('$');
                        name.push_str(&i.to_string());
                        context.variables.remove(&name);
                    }
                    return val;
                }
            }
        } else {
            usage!(1);
        }

        if args.is_empty() {
            use num::Zero;
            return Ok(BigDecimal::zero())
        } else {
            return Ok(args.remove(0));
        }
    } else if name.is_none() {
        if let Some(&Token::BlockName(_)) = context.tokens.peek() {
            // Really ugly code, but we need to know the type *before* we walk out on it
            if let Some(Token::BlockName(name)) = context.tokens.next() {
                return calc_paren(context, Some(name));
            }
        }
    }

    Ok(get_number(context)?)
}
fn get_number<I: Iterator<Item = Token>>(context: &mut Context<I>) -> Result<BigDecimal, CalcError> {
    match context.tokens.next() {
        Some(Token::Num(num)) => Ok(num),
        Some(Token::Sub) => {
            Ok(-calc_paren(context, None)?)
        },
        Some(Token::VarAssign(name)) => {
            if let Some(&Token::ParenOpen) = context.tokens.peek() {
                context.tokens.next();
                let mut fn_tokens = Vec::new();

                let mut depth = 1;
                loop {
                    let token = match context.tokens.next() {
                        Some(Token::Separator) if depth == 1 => return Err(CalcError::SeparatorInDef),
                        Some(token) => token,
                        None => return Err(CalcError::UnclosedParen)
                    };
                    if token == Token::ParenOpen {
                        depth += 1;
                    } else if token == Token::ParenClose {
                        depth -= 1;
                    }
                    fn_tokens.push(token);

                    if depth == 0 {
                        break;
                    } else if depth == std::u8::MAX {
                        return Err(CalcError::TooDeep);
                    }
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
fn require_whole(num: &BigDecimal) -> Result<(), CalcError> {
    if num.with_scale(0) == *num {
        Ok(())
    } else {
        Err(CalcError::NotAWhole)
    }
}
fn require_positive(num: &BigDecimal) -> Result<(), CalcError> {
    match num.sign() {
        Sign::NoSign |
        Sign::Plus => Ok(()),
        Sign::Minus => Err(CalcError::NotAPositive)
    }
}
/// Calculates the factorial of `num`
pub fn factorial(num: BigDecimal, acc: Option<BigDecimal>, times: u8) -> Result<BigDecimal, CalcError> {
    if times == std::u8::MAX {
        return Err(CalcError::TooDeep);
    }
    require_whole(&num)?;
    require_positive(&num)?;

    use num::{Zero, One};
    if num.is_zero() {
        Ok(acc.unwrap_or_else(BigDecimal::one))
    } else {
        let acc = acc.unwrap_or_else(BigDecimal::one);
        let acc = Some(acc * &num);
        // Y THIS NO TAILCALL OPTIMIZE
        factorial(num - BigDecimal::one(), acc, times + 1)
    }
}
/// Calculates `num` to the power of `power`
pub fn pow(num: BigDecimal, power: BigDecimal, acc: Option<BigDecimal>, times: u8) -> Result<BigDecimal, CalcError> {
    if times == std::u8::MAX {
        return Err(CalcError::TooDeep);
    }
    require_positive(&num)?;
    require_whole(&power)?;

    use num::{Zero, One};
    match power.sign() {
        Sign::NoSign => {
            Ok(acc.unwrap_or_else(|| num.clone()))
        },
        Sign::Plus => {
            let one = BigDecimal::one();
            let two = BigDecimal::from(2);

            if (&power % &two).is_zero() {
                let acc = Some(acc.unwrap_or(one) * &num * &num);
                pow(num, power / two, acc, times + 1)
            } else {
                let power = power - &one;
                let acc = Some(acc.unwrap_or(one) * &num);
                pow(num, power, acc, times + 1)
            }
        },
        Sign::Minus => {
            use num::Signed;
            pow(BigDecimal::one() / num, power.abs(), acc, times + 1)
        }
    }
}
