use crate::update_fn::bma_fn_tree::*;
use crate::update_fn::enums::*;
use crate::update_fn::tokenizer::{try_tokenize_bma_formula, BmaFnToken};

/// Parse an BMA update function formula string representation into an actual expression tree.
/// Basically a wrapper for tokenize+parse (used often for testing/debug purposes).
///
/// NEEDS to call [validate_props] to fully finish the preprocessing step.
pub fn parse_bma_formula(formula: &str) -> Result<BmaFnUpdate, String> {
    let tokens = try_tokenize_bma_formula(formula.to_string())?;
    let tree = parse_bma_fn_tokens(&tokens)?;
    Ok(tree)
}

/// Utility method to find the first occurrence of a specific token in the token tree.
fn index_of_first(tokens: &[BmaFnToken], token: BmaFnToken) -> Option<usize> {
    return tokens.iter().position(|t| *t == token);
}

/// Parse `tokens` of BMA update fn formula into an abstract syntax tree using recursive steps.
pub fn parse_bma_fn_tokens(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    parse_1_div(tokens)
}

/// Recursive parsing step 1: extract `/` operators.
fn parse_1_div(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    let div_token = index_of_first(tokens, BmaFnToken::Binary(ArithOp::Div));
    Ok(if let Some(i) = div_token {
        BmaFnUpdate::mk_arithmetic(
            parse_2_mul(&tokens[..i])?,
            parse_1_div(&tokens[(i + 1)..])?,
            ArithOp::Div,
        )
    } else {
        parse_2_mul(tokens)?
    })
}

/// Recursive parsing step 2: extract `*` operators.
fn parse_2_mul(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    let mul_token = index_of_first(tokens, BmaFnToken::Binary(ArithOp::Mult));
    Ok(if let Some(i) = mul_token {
        BmaFnUpdate::mk_arithmetic(
            parse_3_minus(&tokens[..i])?,
            parse_2_mul(&tokens[(i + 1)..])?,
            ArithOp::Mult,
        )
    } else {
        parse_3_minus(tokens)?
    })
}

/// Recursive parsing step 3: extract `-` operators.
fn parse_3_minus(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    let minus_token = index_of_first(tokens, BmaFnToken::Binary(ArithOp::Minus));
    Ok(if let Some(i) = minus_token {
        BmaFnUpdate::mk_arithmetic(
            parse_4_plus(&tokens[..i])?,
            parse_3_minus(&tokens[(i + 1)..])?,
            ArithOp::Minus,
        )
    } else {
        parse_4_plus(tokens)?
    })
}

/// Recursive parsing step 4: extract `+` operators.
fn parse_4_plus(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    let minus_token = index_of_first(tokens, BmaFnToken::Binary(ArithOp::Plus));
    Ok(if let Some(i) = minus_token {
        BmaFnUpdate::mk_arithmetic(
            parse_5_others(&tokens[..i])?,
            parse_4_plus(&tokens[(i + 1)..])?,
            ArithOp::Plus,
        )
    } else {
        parse_5_others(tokens)?
    })
}

/// Recursive parsing step 5: extract literals and recursively solve sub-formulae in parentheses
/// and in functions.
fn parse_5_others(tokens: &[BmaFnToken]) -> Result<BmaFnUpdate, String> {
    if tokens.is_empty() {
        Err("Expected formula, found nothing.".to_string())
    } else {
        if tokens.len() == 1 {
            // This should be name (var/function) or a parenthesis group, anything
            // else does not make sense.
            match &tokens[0] {
                BmaFnToken::Atomic(Literal::Str(name)) => {
                    return Ok(BmaFnUpdate::mk_variable(name.as_str()));
                }
                BmaFnToken::Atomic(Literal::Int(num)) => {
                    return Ok(BmaFnUpdate::mk_constant(*num));
                }
                BmaFnToken::Aggregate(operator, arguments) => {
                    let mut arg_expression_nodes = Vec::new();
                    for inner in arguments {
                        // it must be a token list
                        if let BmaFnToken::TokenList(inner_token_list) = inner {
                            arg_expression_nodes.push(parse_bma_fn_tokens(inner_token_list)?);
                        } else {
                            return Err(
                                "Function must be applied on `BmaFnToken::TokenList` args."
                                    .to_string(),
                            );
                        }
                    }
                    return Ok(BmaFnUpdate::mk_aggregation(
                        operator.clone(),
                        arg_expression_nodes,
                    ));
                }
                BmaFnToken::Unary(operator, argument) => {
                    return if let BmaFnToken::TokenList(inner_token_list) = *argument.clone() {
                        Ok(BmaFnUpdate::mk_unary(
                            parse_bma_fn_tokens(&inner_token_list)?,
                            operator.clone(),
                        ))
                    } else {
                        return Err(
                            "Function must be applied on `BmaFnToken::TokenList` args.".to_string()
                        );
                    }
                }
                // recursively solve sub-formulae in parentheses
                BmaFnToken::TokenList(inner) => {
                    return parse_bma_fn_tokens(inner);
                }
                _ => {} // otherwise, fall through to the error at the end.
            }
        }
        Err(format!("Unexpected: {tokens:?}. Expecting formula."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::update_fn::bma_fn_tree::BmaFnUpdate;
    use crate::update_fn::enums::{AggregateFn, ArithOp, UnaryFn};

    #[test]
    fn test_parse_simple_addition() {
        let input = "3 + 5";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_arithmetic(
            BmaFnUpdate::mk_constant(3),
            BmaFnUpdate::mk_constant(5),
            ArithOp::Plus,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_simple_subtraction() {
        let input = "10 - 7";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_arithmetic(
            BmaFnUpdate::mk_constant(10),
            BmaFnUpdate::mk_constant(7),
            ArithOp::Minus,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_multiplication_and_division() {
        let input = "8 * 4 / 2";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_arithmetic(
            BmaFnUpdate::mk_arithmetic(
                BmaFnUpdate::mk_constant(8),
                BmaFnUpdate::mk_constant(4),
                ArithOp::Mult,
            ),
            BmaFnUpdate::mk_constant(2),
            ArithOp::Div,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_nested_arithmetic() {
        let input = "3 + (5 * 2)";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_arithmetic(
            BmaFnUpdate::mk_constant(3),
            BmaFnUpdate::mk_arithmetic(
                BmaFnUpdate::mk_constant(5),
                BmaFnUpdate::mk_constant(2),
                ArithOp::Mult,
            ),
            ArithOp::Plus,
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_abs_function() {
        let input = "abs(5)";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_unary(BmaFnUpdate::mk_constant(5), UnaryFn::Abs);
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_aggregate_min() {
        let input = "min(3, 5, 5 + variable)";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_aggregation(
            AggregateFn::Min,
            vec![
                BmaFnUpdate::mk_constant(3),
                BmaFnUpdate::mk_constant(5),
                BmaFnUpdate::mk_arithmetic(
                    BmaFnUpdate::mk_constant(5),
                    BmaFnUpdate::mk_variable("variable"),
                    ArithOp::Plus,
                ),
            ],
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_unmatched_parentheses() {
        let input = "3 + (5 * 2";
        let result = parse_bma_formula(input);
        assert!(result.is_err());
        assert_eq!(
            result,
            Err("Expected ')' to previously encountered opening counterpart.".to_string())
        );
    }

    #[test]
    fn test_parse_invalid_token() {
        let input = "5 + @";
        let result = parse_bma_formula(input);
        assert!(result.is_err());
        assert_eq!(result, Err("Unexpected character: '@'".to_string()));
    }

    #[test]
    fn test_parse_function_with_multiple_arguments() {
        let input = "max(3, 5, 10)";
        let result = parse_bma_formula(input);
        let expected = BmaFnUpdate::mk_aggregation(
            AggregateFn::Max,
            vec![
                BmaFnUpdate::mk_constant(3),
                BmaFnUpdate::mk_constant(5),
                BmaFnUpdate::mk_constant(10),
            ],
        );
        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_empty_formula() {
        let input = "";
        let result = parse_bma_formula(input);
        assert!(result.is_err());
        assert_eq!(result, Err("Expected formula, found nothing.".to_string()));
    }
}
