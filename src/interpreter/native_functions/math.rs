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
