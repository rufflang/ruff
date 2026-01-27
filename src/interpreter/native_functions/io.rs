// File: src/interpreter/native_functions/io.rs
//
// I/O-related native functions (print, input, etc.)

use crate::interpreter::{Interpreter, Value};

/// Handle I/O-related function calls
/// Returns Some(value) if the function was handled, None if not recognized
pub fn handle(interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "print" => {
            let output_parts: Vec<String> =
                arg_values.iter().map(Interpreter::stringify_value).collect();
            interp.write_output(&output_parts.join(" "));
            Value::Null
        }
        _ => return None,
    };
    Some(result)
}
