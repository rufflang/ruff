// File: src/interpreter/native_functions/concurrency.rs
//
// Concurrency-related native functions (spawn, channels, etc.)

use crate::interpreter::{Interpreter, Value};
use std::sync::{Arc, Mutex};

pub fn handle(_interp: &mut Interpreter, name: &str, _arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "channel" => {
            // channel() - creates a new channel for thread communication
            use std::sync::mpsc;
            let (sender, receiver) = mpsc::channel();
            #[allow(clippy::arc_with_non_send_sync)]
            let channel = Arc::new(Mutex::new((sender, receiver)));
            Value::Channel(channel)
        }

        _ => return None,
    };

    Some(result)
}
