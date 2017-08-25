extern crate bigdecimal;
extern crate num;

pub mod calculator;
pub mod parser;

use bigdecimal::BigDecimal;
use std::collections::HashMap;

/// Calls both parser::parse and calculator::calculate
/// and merges the output into one happy Result.
pub fn parse_and_calc(
		input: &str,
		variables: &mut HashMap<String, BigDecimal>,
		functions: &mut HashMap<String, Vec<parser::Token>>
	) -> Result<BigDecimal, calculator::CalcError> {

	parser::parse(input).map_err(|err| err.into()).and_then(|parsed| {
		calculator::calculate(&mut calculator::Context {
			tokens: parsed.into_iter().peekable(),
			toplevel: true,
			variables: variables,
			functions: functions
		})
	})
}
