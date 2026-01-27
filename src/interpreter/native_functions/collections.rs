// File: src/interpreter/native_functions/collections.rs
//
// Collection manipulation native functions (arrays, dicts, sets)

use crate::builtins;
use crate::interpreter::{Interpreter, Value};
use std::collections::{HashMap, HashSet, VecDeque};

pub fn handle(interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        // Polymorphic len function - handles arrays, dicts, sets, queues, stacks, bytes
        "len" => match arg_values.first() {
            Some(Value::Array(arr)) => Value::Int(arr.len() as i64),
            Some(Value::Dict(dict)) => Value::Int(dict.len() as i64),
            Some(Value::Bytes(bytes)) => Value::Int(bytes.len() as i64),
            Some(Value::Set(set)) => Value::Int(set.len() as i64),
            Some(Value::Queue(queue)) => Value::Int(queue.len() as i64),
            Some(Value::Stack(stack)) => Value::Int(stack.len() as i64),
            Some(Value::Str(_)) => return None, // Let strings module handle this
            _ => Value::Int(0),
        },

        // Polymorphic contains - handles both strings and arrays
        "contains" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Array(arr)), Some(item)) => Value::Bool(builtins::array_contains(arr, item)),
            _ => return None, // Let strings module handle string case
        },

        // Polymorphic index_of - handles both strings and arrays
        "index_of" => match (arg_values.first(), arg_values.get(1)) {
            (Some(Value::Array(arr)), Some(item)) => Value::Int(builtins::array_index_of(arr, item)),
            _ => return None, // Let strings module handle string case
        },

        // Array functions
        "push" | "append" => {
            if let Some(Value::Array(mut arr)) = arg_values.first().cloned() {
                if let Some(item) = arg_values.get(1).cloned() {
                    arr.push(item);
                    Value::Array(arr)
                } else {
                    Value::Array(arr)
                }
            } else {
                Value::Array(vec![])
            }
        }

        "pop" => {
            if let Some(Value::Array(mut arr)) = arg_values.first().cloned() {
                let popped = arr.pop().unwrap_or(Value::Int(0));
                Value::Array(vec![Value::Array(arr), popped])
            } else {
                Value::Array(vec![])
            }
        }

        "slice" => {
            if let (Some(Value::Array(arr)), Some(Value::Int(start)), Some(Value::Int(end))) =
                (arg_values.first(), arg_values.get(1), arg_values.get(2))
            {
                let start_idx = (*start as usize).max(0).min(arr.len());
                let end_idx = (*end as usize).max(start_idx).min(arr.len());
                Value::Array(arr[start_idx..end_idx].to_vec())
            } else {
                Value::Array(vec![])
            }
        }

        "concat" => {
            if let (Some(Value::Array(arr1)), Some(Value::Array(arr2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let mut result = arr1.clone();
                result.extend(arr2.clone());
                Value::Array(result)
            } else {
                Value::Array(vec![])
            }
        }

        "insert" => {
            if let (Some(Value::Array(arr)), Some(index_val), Some(item)) =
                (arg_values.first().cloned(), arg_values.get(1), arg_values.get(2).cloned())
            {
                let index = match index_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("insert() index must be a number".to_string())),
                };

                match builtins::array_insert(arr, index, item) {
                    Ok(new_arr) => Value::Array(new_arr),
                    Err(e) => Value::Error(e),
                }
            } else {
                Value::Error("insert() requires 3 arguments: array, index, and item".to_string())
            }
        }

        "remove" => {
            match (arg_values.first().cloned(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(item)) => {
                    Value::Array(builtins::array_remove(arr, item))
                }
                (Some(Value::Dict(mut dict)), Some(Value::Str(key))) => {
                    let removed = dict.remove(key).unwrap_or(Value::Int(0));
                    Value::Array(vec![Value::Dict(dict), removed])
                }
                _ => Value::Array(vec![]),
            }
        }

        "remove_at" => {
            if let (Some(Value::Array(arr)), Some(index_val)) =
                (arg_values.first().cloned(), arg_values.get(1))
            {
                let index = match index_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("remove_at() index must be a number".to_string())),
                };

                match builtins::array_remove_at(arr, index) {
                    Ok((new_arr, removed)) => Value::Array(vec![Value::Array(new_arr), removed]),
                    Err(e) => Value::Error(e),
                }
            } else {
                Value::Error("remove_at() requires 2 arguments: array and index".to_string())
            }
        }

        "clear" => {
            match arg_values.first() {
                Some(Value::Array(_)) => Value::Array(builtins::array_clear()),
                Some(Value::Dict(_)) => Value::Dict(HashMap::new()),
                _ => Value::Array(vec![]),
            }
        }

        // Array higher-order functions that need interpreter
        "map" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "map requires two arguments: array and function".to_string(),
                ));
            }

            let (array, func) = match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(func @ Value::Function(_, _, _))) => {
                    (arr.clone(), func.clone())
                }
                _ => return Some(Value::Error("map expects an array and a function".to_string())),
            };

            let mut result = Vec::new();
            for element in array {
                let func_result = interp.call_user_function(&func, &[element]);
                result.push(func_result);
            }
            Value::Array(result)
        }

        "filter" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "filter requires two arguments: array and function".to_string(),
                ));
            }

            let (array, func) = match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(func @ Value::Function(_, _, _))) => {
                    (arr.clone(), func.clone())
                }
                _ => return Some(Value::Error("filter expects an array and a function".to_string())),
            };

            let mut result = Vec::new();
            for element in array {
                let func_result = interp.call_user_function(&func, &[element.clone()]);

                let is_truthy = match func_result {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    Value::Float(n) => n != 0.0,
                    Value::Str(s) => !s.is_empty(),
                    _ => false,
                };

                if is_truthy {
                    result.push(element);
                }
            }
            Value::Array(result)
        }

        "reduce" => {
            if arg_values.len() < 3 {
                return Some(Value::Error(
                    "reduce requires three arguments: array, initial value, and function"
                        .to_string(),
                ));
            }

            let (array, initial, func) =
                match (arg_values.first(), arg_values.get(1), arg_values.get(2)) {
                    (
                        Some(Value::Array(arr)),
                        Some(init),
                        Some(func @ Value::Function(_, _, _)),
                    ) => (arr.clone(), init.clone(), func.clone()),
                    _ => {
                        return Some(Value::Error(
                            "reduce expects an array, an initial value, and a function"
                                .to_string(),
                        ))
                    }
                };

            let mut accumulator = initial;
            for element in array {
                accumulator = interp.call_user_function(&func, &[accumulator, element]);
            }
            accumulator
        }

        "find" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "find requires two arguments: array and function".to_string(),
                ));
            }

            let (array, func) = match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(func @ Value::Function(_, _, _))) => {
                    (arr.clone(), func.clone())
                }
                _ => return Some(Value::Error("find expects an array and a function".to_string())),
            };

            for element in array {
                let func_result = interp.call_user_function(&func, &[element.clone()]);

                let is_truthy = match func_result {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    Value::Float(n) => n != 0.0,
                    Value::Str(s) => !s.is_empty(),
                    _ => false,
                };

                if is_truthy {
                    return Some(element);
                }
            }
            Value::Int(0)
        }

        "any" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "any requires two arguments: array and function".to_string(),
                ));
            }

            let (array, func) = match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(func @ Value::Function(_, _, _))) => {
                    (arr.clone(), func.clone())
                }
                _ => return Some(Value::Error("any expects an array and a function".to_string())),
            };

            for element in array {
                let func_result = interp.call_user_function(&func, &[element]);

                let is_truthy = match func_result {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    Value::Float(n) => n != 0.0,
                    Value::Str(s) => !s.is_empty(),
                    _ => false,
                };

                if is_truthy {
                    return Some(Value::Bool(true));
                }
            }
            Value::Bool(false)
        }

        "all" => {
            if arg_values.len() < 2 {
                return Some(Value::Error(
                    "all requires two arguments: array and function".to_string(),
                ));
            }

            let (array, func) = match (arg_values.first(), arg_values.get(1)) {
                (Some(Value::Array(arr)), Some(func @ Value::Function(_, _, _))) => {
                    (arr.clone(), func.clone())
                }
                _ => return Some(Value::Error("all expects an array and a function".to_string())),
            };

            for element in array {
                let func_result = interp.call_user_function(&func, &[element]);

                let is_truthy = match func_result {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    Value::Float(n) => n != 0.0,
                    Value::Str(s) => !s.is_empty(),
                    _ => false,
                };

                if !is_truthy {
                    return Some(Value::Bool(false));
                }
            }
            Value::Bool(true)
        }

        "sort" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut sorted = arr.clone();
                sorted.sort_by(|a, b| match (a, b) {
                    (Value::Int(x), Value::Int(y)) => x.cmp(y),
                    (Value::Float(x), Value::Float(y)) => {
                        x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Int(x), Value::Float(y)) => {
                        (*x as f64).partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Float(x), Value::Int(y)) => {
                        x.partial_cmp(&(*y as f64)).unwrap_or(std::cmp::Ordering::Equal)
                    }
                    (Value::Str(x), Value::Str(y)) => x.cmp(y),
                    _ => std::cmp::Ordering::Equal,
                });
                Value::Array(sorted)
            } else {
                Value::Error("sort requires an array argument".to_string())
            }
        }

        "reverse" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut reversed = arr.clone();
                reversed.reverse();
                Value::Array(reversed)
            } else {
                Value::Error("reverse requires an array argument".to_string())
            }
        }

        "unique" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut seen = HashSet::new();
                let mut result = Vec::new();

                for element in arr {
                    let key = format!("{:?}", element);
                    if seen.insert(key) {
                        result.push(element.clone());
                    }
                }
                Value::Array(result)
            } else {
                Value::Error("unique requires an array argument".to_string())
            }
        }

        "sum" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut int_sum: i64 = 0;
                let mut float_sum: f64 = 0.0;
                let mut has_float = false;

                for element in arr {
                    match element {
                        Value::Int(n) => {
                            if has_float {
                                float_sum += *n as f64;
                            } else {
                                int_sum += n;
                            }
                        }
                        Value::Float(n) => {
                            if !has_float {
                                float_sum = int_sum as f64;
                                has_float = true;
                            }
                            float_sum += n;
                        }
                        _ => {}
                    }
                }

                if has_float {
                    Value::Float(float_sum)
                } else {
                    Value::Int(int_sum)
                }
            } else {
                Value::Error("sum requires an array argument".to_string())
            }
        }

        "chunk" => {
            if let (Some(Value::Array(arr)), Some(size_val)) =
                (arg_values.first(), arg_values.get(1))
            {
                let size = match size_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("chunk() size must be a number".to_string())),
                };
                Value::Array(builtins::array_chunk(arr, size))
            } else {
                Value::Error("chunk() requires 2 arguments: array and size".to_string())
            }
        }

        "flatten" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                Value::Array(builtins::array_flatten(arr))
            } else {
                Value::Error("flatten() requires an array argument".to_string())
            }
        }

        "zip" => {
            if let (Some(Value::Array(arr1)), Some(Value::Array(arr2))) =
                (arg_values.first(), arg_values.get(1))
            {
                Value::Array(builtins::array_zip(arr1, arr2))
            } else {
                Value::Error("zip() requires 2 array arguments".to_string())
            }
        }

        "enumerate" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                Value::Array(builtins::array_enumerate(arr))
            } else {
                Value::Error("enumerate() requires an array argument".to_string())
            }
        }

        "take" => {
            if let (Some(Value::Array(arr)), Some(n_val)) =
                (arg_values.first(), arg_values.get(1))
            {
                let n = match n_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("take() count must be a number".to_string())),
                };
                Value::Array(builtins::array_take(arr, n))
            } else {
                Value::Error("take() requires 2 arguments: array and count".to_string())
            }
        }

        "skip" => {
            if let (Some(Value::Array(arr)), Some(n_val)) =
                (arg_values.first(), arg_values.get(1))
            {
                let n = match n_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("skip() count must be a number".to_string())),
                };
                Value::Array(builtins::array_skip(arr, n))
            } else {
                Value::Error("skip() requires 2 arguments: array and count".to_string())
            }
        }

        "windows" => {
            if let (Some(Value::Array(arr)), Some(size_val)) =
                (arg_values.first(), arg_values.get(1))
            {
                let size = match size_val {
                    Value::Int(n) => *n,
                    Value::Float(n) => *n as i64,
                    _ => return Some(Value::Error("windows() size must be a number".to_string())),
                };
                Value::Array(builtins::array_windows(arr, size))
            } else {
                Value::Error("windows() requires 2 arguments: array and size".to_string())
            }
        }

        "range" => match builtins::range(&arg_values) {
            Ok(arr) => Value::Array(arr),
            Err(e) => Value::Error(e),
        },

        // Dict functions
        "keys" => {
            if let Some(Value::Dict(dict)) = arg_values.first() {
                let mut keys: Vec<String> = dict.keys().cloned().collect();
                keys.sort();
                let keys: Vec<Value> = keys.into_iter().map(|k| Value::Str(k)).collect();
                Value::Array(keys)
            } else {
                Value::Array(vec![])
            }
        }

        "values" => {
            if let Some(Value::Dict(dict)) = arg_values.first() {
                let mut keys: Vec<&String> = dict.keys().collect();
                keys.sort();
                let vals: Vec<Value> = keys.iter().map(|k| dict.get(*k).unwrap().clone()).collect();
                Value::Array(vals)
            } else {
                Value::Array(vec![])
            }
        }

        "has_key" => {
            if let (Some(Value::Dict(dict)), Some(Value::Str(key))) =
                (arg_values.first(), arg_values.get(1))
            {
                Value::Int(if dict.contains_key(key) { 1 } else { 0 })
            } else {
                Value::Int(0)
            }
        }

        "items" => {
            if let Some(Value::Dict(dict)) = arg_values.first() {
                let mut keys: Vec<&String> = dict.keys().collect();
                keys.sort();
                let items: Vec<Value> = keys
                    .iter()
                    .map(|k| Value::Array(vec![Value::Str((*k).clone()), dict.get(*k).unwrap().clone()]))
                    .collect();
                Value::Array(items)
            } else {
                Value::Array(vec![])
            }
        }

        "get" => {
            if let (Some(Value::Dict(dict)), Some(Value::Str(key))) =
                (arg_values.first(), arg_values.get(1))
            {
                let default = arg_values.get(2).cloned().unwrap_or(Value::Null);
                dict.get(key).cloned().unwrap_or(default)
            } else {
                Value::Null
            }
        }

        "merge" => {
            if let (Some(Value::Dict(dict1)), Some(Value::Dict(dict2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let mut result = dict1.clone();
                for (k, v) in dict2.iter() {
                    result.insert(k.clone(), v.clone());
                }
                Value::Dict(result)
            } else {
                Value::Dict(HashMap::new())
            }
        }

        "invert" => {
            if let Some(Value::Dict(dict)) = arg_values.first() {
                Value::Dict(builtins::dict_invert(dict))
            } else {
                Value::Error("invert() requires a dict argument".to_string())
            }
        }

        "update" => {
            if let (Some(Value::Dict(dict1)), Some(Value::Dict(dict2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let mut result = dict1.clone();
                for (k, v) in dict2.iter() {
                    result.insert(k.clone(), v.clone());
                }
                Value::Dict(result)
            } else {
                Value::Dict(HashMap::new())
            }
        }

        "get_default" => {
            if let (Some(Value::Dict(dict)), Some(Value::Str(key)), Some(default_val)) =
                (arg_values.first(), arg_values.get(1), arg_values.get(2))
            {
                if let Some(value) = dict.get(key) {
                    value.clone()
                } else {
                    default_val.clone()
                }
            } else {
                Value::Error(
                    "get_default() requires 3 arguments: dict, key, default_value".to_string(),
                )
            }
        }

        // Set functions
        "set_add" => {
            if let (Some(Value::Set(mut set)), Some(item)) =
                (arg_values.first().cloned(), arg_values.get(1).cloned())
            {
                let exists = set.iter().any(|v| Interpreter::values_equal(v, &item));
                if !exists {
                    set.push(item);
                }
                Value::Set(set)
            } else {
                Value::Set(Vec::new())
            }
        }

        "set_has" => {
            if let (Some(Value::Set(set)), Some(item)) = (arg_values.first(), arg_values.get(1)) {
                let exists = set.iter().any(|v| Interpreter::values_equal(v, item));
                Value::Bool(exists)
            } else {
                Value::Bool(false)
            }
        }

        "set_remove" => {
            if let (Some(Value::Set(mut set)), Some(item)) =
                (arg_values.first().cloned(), arg_values.get(1))
            {
                set.retain(|v| !Interpreter::values_equal(v, item));
                Value::Set(set)
            } else {
                Value::Set(Vec::new())
            }
        }

        "set_union" => {
            if let (Some(Value::Set(set1)), Some(Value::Set(set2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let mut result = set1.clone();
                for item in set2 {
                    let exists = result.iter().any(|v| Interpreter::values_equal(v, item));
                    if !exists {
                        result.push(item.clone());
                    }
                }
                Value::Set(result)
            } else {
                Value::Set(Vec::new())
            }
        }

        "set_intersect" => {
            if let (Some(Value::Set(set1)), Some(Value::Set(set2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let result: Vec<Value> = set1
                    .iter()
                    .filter(|v| set2.iter().any(|v2| Interpreter::values_equal(v, v2)))
                    .cloned()
                    .collect();
                Value::Set(result)
            } else {
                Value::Set(Vec::new())
            }
        }

        "set_difference" => {
            if let (Some(Value::Set(set1)), Some(Value::Set(set2))) =
                (arg_values.first(), arg_values.get(1))
            {
                let result: Vec<Value> = set1
                    .iter()
                    .filter(|v| !set2.iter().any(|v2| Interpreter::values_equal(v, v2)))
                    .cloned()
                    .collect();
                Value::Set(result)
            } else {
                Value::Set(Vec::new())
            }
        }

        "set_to_array" => {
            if let Some(Value::Set(set)) = arg_values.first() {
                Value::Array(set.clone())
            } else {
                Value::Array(Vec::new())
            }
        }

        // Queue functions
        "Queue" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                let mut queue = VecDeque::new();
                for item in arr {
                    queue.push_back(item.clone());
                }
                Value::Queue(queue)
            } else {
                Value::Queue(VecDeque::new())
            }
        }

        "queue_enqueue" => {
            if let (Some(Value::Queue(mut queue)), Some(item)) =
                (arg_values.first().cloned(), arg_values.get(1).cloned())
            {
                queue.push_back(item);
                Value::Queue(queue)
            } else {
                Value::Queue(VecDeque::new())
            }
        }

        "queue_dequeue" => {
            if let Some(Value::Queue(mut queue)) = arg_values.first().cloned() {
                if let Some(item) = queue.pop_front() {
                    Value::Array(vec![Value::Queue(queue), item])
                } else {
                    Value::Array(vec![Value::Queue(queue), Value::Null])
                }
            } else {
                Value::Array(vec![Value::Queue(VecDeque::new()), Value::Null])
            }
        }

        "queue_peek" => {
            if let Some(Value::Queue(queue)) = arg_values.first() {
                queue.front().cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }

        "queue_is_empty" => {
            if let Some(Value::Queue(queue)) = arg_values.first() {
                Value::Bool(queue.is_empty())
            } else {
                Value::Bool(true)
            }
        }

        "queue_to_array" => {
            if let Some(Value::Queue(queue)) = arg_values.first() {
                Value::Array(queue.iter().cloned().collect())
            } else {
                Value::Array(Vec::new())
            }
        }

        // Stack functions
        "Stack" => {
            if let Some(Value::Array(arr)) = arg_values.first() {
                Value::Stack(arr.clone())
            } else {
                Value::Stack(Vec::new())
            }
        }

        "stack_push" => {
            if let (Some(Value::Stack(mut stack)), Some(item)) =
                (arg_values.first().cloned(), arg_values.get(1).cloned())
            {
                stack.push(item);
                Value::Stack(stack)
            } else {
                Value::Stack(Vec::new())
            }
        }

        "stack_pop" => {
            if let Some(Value::Stack(mut stack)) = arg_values.first().cloned() {
                if let Some(item) = stack.pop() {
                    Value::Array(vec![Value::Stack(stack), item])
                } else {
                    Value::Array(vec![Value::Stack(stack), Value::Null])
                }
            } else {
                Value::Array(vec![Value::Stack(Vec::new()), Value::Null])
            }
        }

        "stack_peek" => {
            if let Some(Value::Stack(stack)) = arg_values.first() {
                stack.last().cloned().unwrap_or(Value::Null)
            } else {
                Value::Null
            }
        }

        "stack_is_empty" => {
            if let Some(Value::Stack(stack)) = arg_values.first() {
                Value::Bool(stack.is_empty())
            } else {
                Value::Bool(true)
            }
        }

        "stack_to_array" => {
            if let Some(Value::Stack(stack)) = arg_values.first() {
                Value::Array(stack.clone())
            } else {
                Value::Array(Vec::new())
            }
        }

        _ => return None,
    };

    Some(result)
}
