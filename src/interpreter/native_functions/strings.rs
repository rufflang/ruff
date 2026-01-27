// File: src/interpreter/native_functions/strings.rs
//
// String manipulation native functions

use crate::builtins;
use crate::interpreter::Value;

pub fn handle(name: &str, args: &[Value]) -> Option<Value> {
    let result = match name {
        "len" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Int(builtins::str_len(s) as i64)
            } else {
                return None // Let collections module handle other types
            }
        }

        "to_upper" | "upper" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::to_upper(s))
            } else {
                Value::Str(String::new())
            }
        }

        "to_lower" | "lower" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::to_lower(s))
            } else {
                Value::Str(String::new())
            }
        }

        "capitalize" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::capitalize(s))
            } else {
                Value::Str(String::new())
            }
        }

        "trim" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::trim(s))
            } else {
                Value::Str(String::new())
            }
        }

        "trim_start" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::trim_start(s))
            } else {
                Value::Str(String::new())
            }
        }

        "trim_end" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::trim_end(s))
            } else {
                Value::Str(String::new())
            }
        }

        "char_at" => {
            if let (Some(Value::Str(s)), Some(index_val)) = (args.first(), args.get(1)) {
                let index = match index_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                Value::Str(builtins::char_at(s, index))
            } else {
                Value::Str(String::new())
            }
        }

        "is_empty" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Bool(builtins::is_empty(s))
            } else {
                Value::Bool(true)
            }
        }

        "count_chars" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Int(builtins::count_chars(s))
            } else {
                Value::Int(0)
            }
        }

        "contains" => {
            // Polymorphic: works with strings (other types handled in collections)
            match (args.first(), args.get(1)) {
                (Some(Value::Str(s)), Some(Value::Str(substr))) => {
                    Value::Int(if builtins::contains(s, substr) { 1 } else { 0 })
                }
                _ => return None, // Let collections.rs handle array case
            }
        }

        "substring" => {
            if let (Some(Value::Str(s)), Some(start_val), Some(end_val)) =
                (args.first(), args.get(1), args.get(2))
            {
                let start = match start_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                let end = match end_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                Value::Str(builtins::substring(s, start, end))
            } else {
                Value::Str(String::new())
            }
        }

        "replace_str" | "replace" => {
            if let (Some(Value::Str(s)), Some(Value::Str(old)), Some(Value::Str(new))) =
                (args.first(), args.get(1), args.get(2))
            {
                Value::Str(builtins::replace(s, old, new))
            } else {
                Value::Str(String::new())
            }
        }

        "starts_with" => {
            if let (Some(Value::Str(s)), Some(Value::Str(prefix))) = (args.first(), args.get(1))
            {
                Value::Bool(builtins::starts_with(s, prefix))
            } else {
                Value::Bool(false)
            }
        }

        "ends_with" => {
            if let (Some(Value::Str(s)), Some(Value::Str(suffix))) = (args.first(), args.get(1))
            {
                Value::Bool(builtins::ends_with(s, suffix))
            } else {
                Value::Bool(false)
            }
        }

        "index_of" => {
            // Polymorphic: works with strings (other types handled in collections)
            match (args.first(), args.get(1)) {
                (Some(Value::Str(s)), Some(Value::Str(substr))) => {
                    Value::Int(builtins::index_of(s, substr) as i64)
                }
                _ => return None, // Let collections.rs handle array case
            }
        }

        "repeat" => {
            if let (Some(Value::Str(s)), Some(count_val)) = (args.first(), args.get(1)) {
                let count = match count_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                Value::Str(builtins::repeat(s, count))
            } else {
                Value::Str(String::new())
            }
        }

        "split" => {
            if let (Some(Value::Str(s)), Some(Value::Str(delimiter))) = (args.first(), args.get(1))
            {
                let parts = builtins::split(s, delimiter);
                let values: Vec<Value> = parts.into_iter().map(Value::Str).collect();
                Value::Array(values)
            } else {
                Value::Array(vec![])
            }
        }

        "join" => {
            if let (Some(Value::Array(arr)), Some(Value::Str(separator))) =
                (args.first(), args.get(1))
            {
                // Convert array elements to strings
                let strings: Vec<String> = arr
                    .iter()
                    .map(|v| match v {
                        Value::Str(s) => s.clone(),
                        Value::Int(n) => n.to_string(),
                        Value::Float(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Value::Str(builtins::join(&strings, separator))
            } else {
                Value::Str(String::new())
            }
        }

        "pad_left" => {
            if let (Some(Value::Str(s)), Some(width_val), Some(Value::Str(pad_char))) =
                (args.first(), args.get(1), args.get(2))
            {
                let width = match width_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => 0,
                };
                Value::Str(builtins::str_pad_left(s, width, pad_char))
            } else {
                Value::Error("pad_left() requires 3 arguments: string, width, char".to_string())
            }
        }

        "pad_right" => {
            if let (Some(Value::Str(s)), Some(width_val), Some(Value::Str(pad_char))) =
                (args.first(), args.get(1), args.get(2))
            {
                let width = match width_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => 0,
                };
                Value::Str(builtins::str_pad_right(s, width, pad_char))
            } else {
                Value::Error("pad_right() requires 3 arguments: string, width, char".to_string())
            }
        }

        "lines" => {
            if let Some(Value::Str(s)) = args.first() {
                let lines = builtins::str_lines(s);
                Value::Array(lines.into_iter().map(Value::Str).collect())
            } else {
                Value::Error("lines() requires a string argument".to_string())
            }
        }

        "words" => {
            if let Some(Value::Str(s)) = args.first() {
                let words = builtins::str_words(s);
                Value::Array(words.into_iter().map(Value::Str).collect())
            } else {
                Value::Error("words() requires a string argument".to_string())
            }
        }

        "str_reverse" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::str_reverse(s))
            } else {
                Value::Error("str_reverse() requires a string argument".to_string())
            }
        }

        "slugify" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::str_slugify(s))
            } else {
                Value::Error("slugify() requires a string argument".to_string())
            }
        }

        "truncate" => {
            if let (Some(Value::Str(s)), Some(len_val), Some(Value::Str(suffix))) =
                (args.first(), args.get(1), args.get(2))
            {
                let max_len = match len_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => 0,
                };
                Value::Str(builtins::str_truncate(s, max_len, suffix))
            } else {
                Value::Error("truncate() requires 3 arguments: string, length, suffix".to_string())
            }
        }

        "to_camel_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::str_to_camel_case(s))
            } else {
                Value::Error("to_camel_case() requires a string argument".to_string())
            }
        }

        "to_snake_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::str_to_snake_case(s))
            } else {
                Value::Error("to_snake_case() requires a string argument".to_string())
            }
        }

        "to_kebab_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(builtins::str_to_kebab_case(s))
            } else {
                Value::Error("to_kebab_case() requires a string argument".to_string())
            }
        }

        _ => return None, // Not a string function
    };

    Some(result)
}
