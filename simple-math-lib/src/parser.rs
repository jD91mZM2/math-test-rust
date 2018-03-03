use bigdecimal::BigDecimal;
use calculator::CalcError;
use std::{fmt, mem};

/// A token
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
    BlockName(String),
    Num(BigDecimal),
    ParenClose,
    ParenOpen,
    Separator,
    VarAssign(String),
    VarGet(String),

    Add,
    And,
    BitshiftLeft,
    BitshiftRight,
    Div,
    Factorial,
    Mul,
    Not,
    Or,
    Pow,
    Rem,
    Sub,
    Xor
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Token::BlockName(ref name) => write!(f, "\"{}\"", name),
            Token::Num(ref num) => write!(f, "Number {}", num),
            Token::ParenClose => write!(f, ")"),
            Token::ParenOpen => write!(f, "("),
            Token::Separator => write!(f, ","),
            Token::VarAssign(ref name) => write!(f, "Variable assignment \"{}\"", name),
            Token::VarGet(ref name) => write!(f, "Variable \"{}\"", name),

            Token::Add => write!(f, "Plus (+)"),
            Token::And => write!(f, "Bitwise AND (&)"),
            Token::BitshiftLeft => write!(f, "Bitshift left (<<)"),
            Token::BitshiftRight => write!(f, "Bitshift right (>>)"),
            Token::Div => write!(f, "Division symbol (/)"),
            Token::Factorial => write!(f, "Factorial (!)"),
            Token::Mul => write!(f, "Times (*)"),
            Token::Not => write!(f, "Bitwise NOT (~)"),
            Token::Or => write!(f, "Bitwise OR (|)"),
            Token::Pow => write!(f, "Exponential (**)"),
            Token::Rem => write!(f, "Remainder (%)"),
            Token::Sub => write!(f, "Minus (-)"),
            Token::Xor => write!(f, "Bitwise XOR (^)")
        }
    }
}

/// An error when parsing
#[derive(Debug, Fail)]
pub enum ParseError {
    #[fail(display = "Character '{}' neither a number nor a valid letter \
                      in a function or variable name.", _0)]
    DisallowedChar(char),
    #[fail(display = "You may only use whole numbers in this context")]
    DisallowedDecimal,
    #[fail(display = "\"{}\" is not a valid variable name.", _0)]
    DisallowedVariable(String),
    #[fail(display = "Character '{0}' isn't followed by another '{0}'.\n\
                      Looks like a failed attempt to bitshift.", _0)]
    UnclosedBitShift(char)
}
impl Into<CalcError> for ParseError {
    fn into(self) -> CalcError {
        CalcError::ParseError(self)
    }
}

/// "Parse" the string into a list of tokens.
/// This is technically actually a tokenizer...
pub fn parse(input: &str) -> Result<Vec<Token>, ParseError> {
    let mut output = Vec::new();
    let mut buffer = String::new();

    macro_rules! prepare_var {
        () => {
            if let Some(&Token::Num(_)) = output.last() {
                output.push(Token::Mul);
            }
        }
    }
    macro_rules! flush {
        () => {
            if !buffer.is_empty() {
                let buffer = mem::replace(&mut buffer, String::new());
                match parse_num(&buffer) {
                    Ok(num) => {
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

    let mut chars = input.chars().enumerate().peekable();
    while let Some((i, c)) = chars.next() {
        let token = match c {
            ' ' => continue,
            ',' => Some(Token::Separator),
            ')' => Some(Token::ParenClose),
            '+' => Some(Token::Add),
            '-' => Some(Token::Sub),
            '*' => if let Some(&(_, '*')) = chars.peek() {
                    chars.next();
                    Some(Token::Pow)
                } else {
                    Some(Token::Mul)
                },
            '/' => Some(Token::Div),
            '%' => Some(Token::Rem),
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
                    Ok(num) => {
                        output.push(Token::Num(num));
                        output.push(Token::Mul);
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
            if buffer.is_empty() || is_num(&buffer) || buffer.starts_with('$') || buffer.starts_with('0') {
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
                (code >= '0' as u32 && code <= '9' as u32) ||
                (c == '_' || c == '$') {

                if was_num && !num && !buffer.starts_with('0') {
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
fn is_num(mut num: &str) -> bool {
    let radix = if num.len() < 2 {
        10
    } else {
        match &num[..2] {
            "0x" => 16,
            "0o" => 8,
            "0b" => 2,
            _ => 10
        }
    };

    if radix != 10 {
        num = &num[2..];
    }

    !num.is_empty() && num.chars().all(|c| c.is_digit(radix) || (radix == 10 && c == '.'))
}
