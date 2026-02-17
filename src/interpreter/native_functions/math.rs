// File: src/interpreter/native_functions/math.rs
//
// Math-related native functions

use crate::builtins;
use crate::interpreter::Value;

/// Handle math-related function calls  
/// Returns Some(value) if the function was handled, None if not recognized
pub fn handle(name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Math functions - single argument
        "abs" | "sqrt" | "floor" | "ceil" | "round" | "sin" | "cos" | "tan" | "log" | "exp" => {
            if arg_values.len() > 1 {
                return Some(Value::Error(format!("{}() expects 1 argument", name)));
            }

            if let Some(val) = arg_values.first() {
                let x = match val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Int(0)),
                };
                let result = match name {
                    "abs" => builtins::abs(x),
                    "sqrt" => builtins::sqrt(x),
                    "floor" => builtins::floor(x),
                    "ceil" => builtins::ceil(x),
                    "round" => builtins::round(x),
                    "sin" => builtins::sin(x),
                    "cos" => builtins::cos(x),
                    "tan" => builtins::tan(x),
                    "log" => builtins::log(x),
                    "exp" => builtins::exp(x),
                    _ => 0.0,
                };
                Value::Float(result)
            } else {
                Value::Int(0)
            }
        }

        // Math functions - two arguments
        "pow" | "min" | "max" => {
            if arg_values.len() > 2 {
                return Some(Value::Error(format!("{}() expects 2 arguments", name)));
            }

            if let (Some(val_a), Some(val_b)) = (arg_values.first(), arg_values.get(1)) {
                let a = match val_a {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Int(0)),
                };
                let b = match val_b {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => return Some(Value::Int(0)),
                };
                let result = match name {
                    "pow" => builtins::pow(a, b),
                    "min" => builtins::min(a, b),
                    "max" => builtins::max(a, b),
                    _ => 0.0,
                };
                Value::Float(result)
            } else {
                Value::Int(0)
            }
        }

        _ => return None,
    };
    Some(result)
}

#[cfg(test)]
mod tests {
    use super::handle;
    use crate::interpreter::Value;
    use std::sync::Arc;

    #[test]
    fn test_math_api_strict_arity_rejects_extra_arguments() {
        let abs_extra =
            handle("abs", &[Value::Int(-7), Value::Int(1)]).expect("abs should return a result");
        assert!(
            matches!(abs_extra, Value::Error(message) if message.contains("abs() expects 1 argument"))
        );

        let sqrt_extra =
            handle("sqrt", &[Value::Int(9), Value::Int(1)]).expect("sqrt should return a result");
        assert!(
            matches!(sqrt_extra, Value::Error(message) if message.contains("sqrt() expects 1 argument"))
        );

        let exp_extra =
            handle("exp", &[Value::Int(1), Value::Int(2)]).expect("exp should return a result");
        assert!(
            matches!(exp_extra, Value::Error(message) if message.contains("exp() expects 1 argument"))
        );

        let pow_extra = handle("pow", &[Value::Int(2), Value::Int(8), Value::Int(1)])
            .expect("pow should return a result");
        assert!(
            matches!(pow_extra, Value::Error(message) if message.contains("pow() expects 2 arguments"))
        );

        let min_extra = handle("min", &[Value::Int(1), Value::Int(2), Value::Int(3)])
            .expect("min should return a result");
        assert!(
            matches!(min_extra, Value::Error(message) if message.contains("min() expects 2 arguments"))
        );

        let max_extra = handle("max", &[Value::Int(1), Value::Int(2), Value::Int(3)])
            .expect("max should return a result");
        assert!(
            matches!(max_extra, Value::Error(message) if message.contains("max() expects 2 arguments"))
        );
    }

    #[test]
    fn test_math_api_preserves_missing_argument_fallbacks() {
        let abs_missing = handle("abs", &[]).expect("abs should return a result");
        assert!(matches!(abs_missing, Value::Int(0)));

        let pow_missing = handle("pow", &[Value::Int(2)]).expect("pow should return a result");
        assert!(matches!(pow_missing, Value::Int(0)));

        let pow_invalid = handle("pow", &[Value::Int(2), Value::Str(Arc::new("bad".to_string()))])
            .expect("pow should return a result");
        assert!(matches!(pow_invalid, Value::Int(0)));
    }
}
