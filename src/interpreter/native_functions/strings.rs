// File: src/interpreter/native_functions/strings.rs
//
// String manipulation native functions

use crate::builtins;
use crate::interpreter::{DictMap, Value};
use std::sync::Arc;

pub fn handle(name: &str, args: &[Value]) -> Option<Value> {
    let result = match name {
        "len" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Int(builtins::str_len(&**s) as i64)
            } else {
                return None; // Let collections module handle other types
            }
        }

        "to_upper" | "upper" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::to_upper(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "to_lower" | "lower" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::to_lower(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "capitalize" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::capitalize(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "trim" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::trim(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "trim_start" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::trim_start(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "trim_end" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::trim_end(&**s)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "char_at" => {
            if let (Some(Value::Str(s)), Some(index_val)) = (args.first(), args.get(1)) {
                let index = match index_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                Value::Str(Arc::new(builtins::char_at(&**s, index)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "is_empty" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Bool(builtins::is_empty(&**s))
            } else {
                Value::Bool(true)
            }
        }

        "count_chars" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Int(builtins::count_chars(&**s))
            } else {
                Value::Int(0)
            }
        }

        "contains" => {
            // Polymorphic: strings handled here, arrays delegated to collections.rs
            match args.first() {
                Some(Value::Array(_)) => return None,
                Some(Value::Str(s)) => match args.get(1) {
                    Some(Value::Str(substr)) => {
                        Value::Int(if builtins::contains(&**s, &**substr) { 1 } else { 0 })
                    }
                    _ => Value::Error(
                        "contains() requires two arguments: string/array and substring/item"
                            .to_string(),
                    ),
                },
                Some(_) => {
                    Value::Error("contains() first argument must be a string or array".to_string())
                }
                None => Value::Error(
                    "contains() requires two arguments: string/array and substring/item"
                        .to_string(),
                ),
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
                Value::Str(Arc::new(builtins::substring(&**s, start, end)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "replace_str" | "replace" => {
            if let (Some(Value::Str(s)), Some(Value::Str(old)), Some(Value::Str(new))) =
                (args.first(), args.get(1), args.get(2))
            {
                Value::Str(Arc::new(builtins::replace(&**s, &**old, &**new)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "starts_with" => {
            if let (Some(Value::Str(s)), Some(Value::Str(prefix))) = (args.first(), args.get(1)) {
                Value::Bool(builtins::starts_with(&**s, &**prefix))
            } else {
                Value::Bool(false)
            }
        }

        "ends_with" => {
            if let (Some(Value::Str(s)), Some(Value::Str(suffix))) = (args.first(), args.get(1)) {
                Value::Bool(builtins::ends_with(&**s, &**suffix))
            } else {
                Value::Bool(false)
            }
        }

        "index_of" => {
            // Polymorphic: strings handled here, arrays delegated to collections.rs
            match args.first() {
                Some(Value::Array(_)) => return None,
                Some(Value::Str(s)) => match args.get(1) {
                    Some(Value::Str(substr)) => {
                        Value::Int(builtins::index_of(&**s, &**substr) as i64)
                    }
                    _ => Value::Error(
                        "index_of() requires two arguments: string/array and substring/item"
                            .to_string(),
                    ),
                },
                Some(_) => {
                    Value::Error("index_of() first argument must be a string or array".to_string())
                }
                None => Value::Error(
                    "index_of() requires two arguments: string/array and substring/item"
                        .to_string(),
                ),
            }
        }

        "repeat" => {
            if let (Some(Value::Str(s)), Some(count_val)) = (args.first(), args.get(1)) {
                let count = match count_val {
                    Value::Int(n) => *n as f64,
                    Value::Float(n) => *n,
                    _ => 0.0,
                };
                Value::Str(Arc::new(builtins::repeat(&**s, count)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "split" => {
            if let (Some(Value::Str(s)), Some(Value::Str(delimiter))) = (args.first(), args.get(1))
            {
                let parts = builtins::split(&**s, &**delimiter);
                let values: Vec<Value> =
                    parts.into_iter().map(|s| Value::Str(Arc::new(s))).collect();
                Value::Array(Arc::new(values))
            } else {
                Value::Array(Arc::new(vec![]))
            }
        }

        "regex_match" => {
            if let (Some(Value::Str(text)), Some(Value::Str(pattern))) = (args.first(), args.get(1))
            {
                Value::Bool(builtins::regex_match(text.as_ref(), pattern.as_ref()))
            } else {
                Value::Error(
                    "regex_match requires two string arguments (text, pattern)".to_string(),
                )
            }
        }

        "regex_find_all" => {
            if let (Some(Value::Str(text)), Some(Value::Str(pattern))) = (args.first(), args.get(1))
            {
                let matches = builtins::regex_find_all(text.as_ref(), pattern.as_ref());
                let values: Vec<Value> =
                    matches.into_iter().map(|s| Value::Str(Arc::new(s))).collect();
                Value::Array(Arc::new(values))
            } else {
                Value::Error(
                    "regex_find_all requires two string arguments (text, pattern)".to_string(),
                )
            }
        }

        "regex_replace" => {
            if let (
                Some(Value::Str(text)),
                Some(Value::Str(pattern)),
                Some(Value::Str(replacement)),
            ) = (args.first(), args.get(1), args.get(2))
            {
                Value::Str(Arc::new(builtins::regex_replace(
                    text.as_ref(),
                    pattern.as_ref(),
                    replacement.as_ref(),
                )))
            } else {
                Value::Error(
                    "regex_replace requires three string arguments (text, pattern, replacement)"
                        .to_string(),
                )
            }
        }

        "regex_split" => {
            if let (Some(Value::Str(text)), Some(Value::Str(pattern))) = (args.first(), args.get(1))
            {
                let parts = builtins::regex_split(text.as_ref(), pattern.as_ref());
                let values: Vec<Value> =
                    parts.into_iter().map(|s| Value::Str(Arc::new(s))).collect();
                Value::Array(Arc::new(values))
            } else {
                Value::Error(
                    "regex_split requires two string arguments (text, pattern)".to_string(),
                )
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
                        Value::Str(s) => (&**s).to_string(),
                        Value::Int(n) => n.to_string(),
                        Value::Float(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => format!("{:?}", v),
                    })
                    .collect();
                Value::Str(Arc::new(builtins::join(&strings, &**separator)))
            } else {
                Value::Str(Arc::new(String::new()))
            }
        }

        "ssg_render_pages" => {
            // ssg_render_pages(source_pages: Array<String>) -> Dict
            // Returns { "pages": Array<String>, "checksum": Int }
            if args.len() != 1 {
                Value::Error(format!(
                    "ssg_render_pages() expects 1 argument (array of source pages), got {}",
                    args.len()
                ))
            } else {
                match args.first() {
                    Some(Value::Array(source_pages)) => {
                        let mut rendered_pages = Vec::with_capacity(source_pages.len());
                        let mut checksum: i64 = 0;

                        for (index, page) in source_pages.iter().enumerate() {
                            let source_body = match page {
                                Value::Str(body) => body,
                                _ => {
                                    return Some(Value::Error(format!(
                                        "ssg_render_pages() source page at index {} must be a string",
                                        index
                                    )));
                                }
                            };

                            let index_str = index.to_string();
                            let mut html = String::with_capacity(source_body.len() + 64);
                            html.push_str("<html><body><h1>Post ");
                            html.push_str(index_str.as_str());
                            html.push_str("</h1><article>");
                            html.push_str(source_body.as_ref());
                            html.push_str("</article></body></html>");

                            checksum += html.len() as i64;
                            rendered_pages.push(Value::Str(Arc::new(html)));
                        }

                        let mut result = DictMap::default();
                        result.insert("pages".into(), Value::Array(Arc::new(rendered_pages)));
                        result.insert("checksum".into(), Value::Int(checksum));
                        Value::Dict(Arc::new(result))
                    }
                    _ => Value::Error(
                        "ssg_render_pages() requires an array of source page strings".to_string(),
                    ),
                }
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
                Value::Str(Arc::new(builtins::str_pad_left(&**s, width, &**pad_char)))
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
                Value::Str(Arc::new(builtins::str_pad_right(&**s, width, &**pad_char)))
            } else {
                Value::Error("pad_right() requires 3 arguments: string, width, char".to_string())
            }
        }

        "lines" => {
            if let Some(Value::Str(s)) = args.first() {
                let lines = builtins::str_lines(&**s);
                Value::Array(Arc::new(lines.into_iter().map(|s| Value::Str(Arc::new(s))).collect()))
            } else {
                Value::Error("lines() requires a string argument".to_string())
            }
        }

        "words" => {
            if let Some(Value::Str(s)) = args.first() {
                let words = builtins::str_words(&**s);
                Value::Array(Arc::new(words.into_iter().map(|s| Value::Str(Arc::new(s))).collect()))
            } else {
                Value::Error("words() requires a string argument".to_string())
            }
        }

        "str_reverse" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::str_reverse(&**s)))
            } else {
                Value::Error("str_reverse() requires a string argument".to_string())
            }
        }

        "slugify" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::str_slugify(&**s)))
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
                Value::Str(Arc::new(builtins::str_truncate(&**s, max_len, &**suffix)))
            } else {
                Value::Error("truncate() requires 3 arguments: string, length, suffix".to_string())
            }
        }

        "to_camel_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::str_to_camel_case(&**s)))
            } else {
                Value::Error("to_camel_case() requires a string argument".to_string())
            }
        }

        "to_snake_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::str_to_snake_case(&**s)))
            } else {
                Value::Error("to_snake_case() requires a string argument".to_string())
            }
        }

        "to_kebab_case" => {
            if let Some(Value::Str(s)) = args.first() {
                Value::Str(Arc::new(builtins::str_to_kebab_case(&**s)))
            } else {
                Value::Error("to_kebab_case() requires a string argument".to_string())
            }
        }

        _ => return None, // Not a string function
    };

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn str_value(value: &str) -> Value {
        Value::Str(Arc::new(value.to_string()))
    }

    #[test]
    fn test_ssg_render_pages_success_returns_pages_and_checksum() {
        let args = vec![Value::Array(Arc::new(vec![
            str_value("# Post 0\n\nGenerated page 0"),
            str_value("# Post 1\n\nGenerated page 1"),
        ]))];

        let result = handle("ssg_render_pages", &args).unwrap();
        match result {
            Value::Dict(dict) => {
                let pages = dict.get("pages").expect("pages key missing");
                let checksum = dict.get("checksum").expect("checksum key missing");

                match pages {
                    Value::Array(values) => {
                        assert_eq!(values.len(), 2);
                        assert!(
                            matches!(&values[0], Value::Str(s) if s.as_ref() == "<html><body><h1>Post 0</h1><article># Post 0\n\nGenerated page 0</article></body></html>")
                        );
                        assert!(
                            matches!(&values[1], Value::Str(s) if s.as_ref() == "<html><body><h1>Post 1</h1><article># Post 1\n\nGenerated page 1</article></body></html>")
                        );
                    }
                    _ => panic!("Expected pages to be an Array"),
                }

                let expected_checksum = "<html><body><h1>Post 0</h1><article># Post 0\n\nGenerated page 0</article></body></html>".len() as i64
                    + "<html><body><h1>Post 1</h1><article># Post 1\n\nGenerated page 1</article></body></html>".len() as i64;

                match checksum {
                    Value::Int(value) => assert_eq!(*value, expected_checksum),
                    _ => panic!("Expected checksum to be an Int"),
                }
            }
            _ => panic!("Expected Dict result from ssg_render_pages"),
        }
    }

    #[test]
    fn test_ssg_render_pages_requires_array_argument() {
        let args = vec![str_value("not-an-array")];
        let result = handle("ssg_render_pages", &args).unwrap();

        match result {
            Value::Error(message) => {
                assert!(message.contains("requires an array of source page strings"));
            }
            _ => panic!("Expected Value::Error for non-array input"),
        }
    }

    #[test]
    fn test_regex_match_and_find_all() {
        let match_result =
            handle("regex_match", &[str_value("hello123"), str_value("^[a-z]+\\d+$")]).unwrap();
        assert!(matches!(match_result, Value::Bool(true)));

        let find_all_result =
            handle("regex_find_all", &[str_value("a1 b22 c333"), str_value("\\d+")]).unwrap();

        match find_all_result {
            Value::Array(values) => {
                assert_eq!(values.len(), 3);
                assert!(matches!(&values[0], Value::Str(s) if s.as_ref() == "1"));
                assert!(matches!(&values[1], Value::Str(s) if s.as_ref() == "22"));
                assert!(matches!(&values[2], Value::Str(s) if s.as_ref() == "333"));
            }
            _ => panic!("Expected Value::Array from regex_find_all"),
        }
    }

    #[test]
    fn test_regex_replace_and_split() {
        let replace_result =
            handle("regex_replace", &[str_value("a1 b22"), str_value("\\d+"), str_value("#")])
                .unwrap();
        assert!(matches!(&replace_result, Value::Str(s) if s.as_ref() == "a# b#"));

        let split_result =
            handle("regex_split", &[str_value("a, b; c"), str_value("[,;]\\s*")]).unwrap();

        match split_result {
            Value::Array(values) => {
                assert_eq!(values.len(), 3);
                assert!(matches!(&values[0], Value::Str(s) if s.as_ref() == "a"));
                assert!(matches!(&values[1], Value::Str(s) if s.as_ref() == "b"));
                assert!(matches!(&values[2], Value::Str(s) if s.as_ref() == "c"));
            }
            _ => panic!("Expected Value::Array from regex_split"),
        }
    }

    #[test]
    fn test_regex_argument_validation_errors() {
        let match_error = handle("regex_match", &[Value::Int(1)]).unwrap();
        assert!(
            matches!(match_error, Value::Error(message) if message.contains("regex_match requires two string arguments"))
        );

        let replace_error = handle("regex_replace", &[str_value("a"), str_value("b")]).unwrap();
        assert!(
            matches!(replace_error, Value::Error(message) if message.contains("regex_replace requires three string arguments"))
        );
    }

    #[test]
    fn test_contains_and_index_of_string_behavior() {
        let contains_true =
            handle("contains", &[str_value("ruff language"), str_value("lang")]).unwrap();
        assert!(matches!(contains_true, Value::Int(1)));

        let contains_false =
            handle("contains", &[str_value("ruff language"), str_value("python")]).unwrap();
        assert!(matches!(contains_false, Value::Int(0)));

        let index_found = handle("index_of", &[str_value("abcabc"), str_value("ca")]).unwrap();
        assert!(matches!(index_found, Value::Int(2)));

        let index_missing = handle("index_of", &[str_value("abcabc"), str_value("zz")]).unwrap();
        assert!(matches!(index_missing, Value::Int(-1)));
    }

    #[test]
    fn test_contains_and_index_of_argument_shape_errors() {
        let contains_missing = handle("contains", &[str_value("ruff")]).unwrap();
        assert!(
            matches!(contains_missing, Value::Error(message) if message.contains("contains() requires two arguments"))
        );

        let index_missing = handle("index_of", &[str_value("ruff")]).unwrap();
        assert!(
            matches!(index_missing, Value::Error(message) if message.contains("index_of() requires two arguments"))
        );

        let contains_invalid_type = handle("contains", &[Value::Int(1), str_value("x")]).unwrap();
        assert!(
            matches!(contains_invalid_type, Value::Error(message) if message.contains("first argument must be a string or array"))
        );

        let index_invalid_type = handle("index_of", &[Value::Bool(true), str_value("x")]).unwrap();
        assert!(
            matches!(index_invalid_type, Value::Error(message) if message.contains("first argument must be a string or array"))
        );
    }

    #[test]
    fn test_contains_and_index_of_delegate_array_case_to_collections() {
        let array_args =
            [Value::Array(Arc::new(vec![Value::Int(1), Value::Int(2)])), Value::Int(2)];
        assert!(handle("contains", &array_args).is_none());
        assert!(handle("index_of", &array_args).is_none());
    }

    #[test]
    fn test_ssg_render_pages_requires_string_elements() {
        let args = vec![Value::Array(Arc::new(vec![Value::Int(1)]))];
        let result = handle("ssg_render_pages", &args).unwrap();

        match result {
            Value::Error(message) => {
                assert!(message.contains("source page at index 0 must be a string"));
            }
            _ => panic!("Expected Value::Error for non-string source page"),
        }
    }

    #[test]
    fn test_ssg_render_pages_validates_argument_count() {
        let args = vec![Value::Array(Arc::new(vec![])), Value::Array(Arc::new(vec![]))];
        let result = handle("ssg_render_pages", &args).unwrap();

        match result {
            Value::Error(message) => {
                assert!(message.contains("expects 1 argument"));
            }
            _ => panic!("Expected Value::Error for invalid argument count"),
        }
    }
}
