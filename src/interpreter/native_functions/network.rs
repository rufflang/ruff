// File: src/interpreter/native_functions/network.rs
//
// Network-related native functions (TCP, UDP sockets)

use crate::interpreter::{DictMap, Interpreter, Value};
use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

pub fn handle(_interp: &mut Interpreter, name: &str, arg_values: &[Value]) -> Option<Value> {
    let result = match name {
        "tcp_listen" => {
            if arg_values.len() != 2 {
                Value::Error("tcp_listen requires (string_host, int_port) arguments".to_string())
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::Str(host)), Some(Value::Int(port))) => {
                        let address = format!("{}:{}", host.as_ref(), port);
                        match std::net::TcpListener::bind(&address) {
                            Ok(listener) => {
                                let _ = listener.set_nonblocking(false);
                                Value::TcpListener {
                                    listener: Arc::new(Mutex::new(listener)),
                                    addr: address,
                                }
                            }
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to bind TCP listener on '{}': {}",
                                    address, error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "tcp_listen requires (string_host, int_port) arguments".to_string(),
                    ),
                }
            }
        }

        "tcp_accept" => {
            if arg_values.len() != 1 {
                Value::Error("tcp_accept requires a TcpListener argument".to_string())
            } else {
                if let Some(Value::TcpListener { listener, .. }) = arg_values.first() {
                    let listener_guard = listener.lock().unwrap();
                    match listener_guard.accept() {
                        Ok((stream, peer_address)) => Value::TcpStream {
                            stream: Arc::new(Mutex::new(stream)),
                            peer_addr: peer_address.to_string(),
                        },
                        Err(error) => Value::ErrorObject {
                            message: format!("Failed to accept connection: {}", error),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                } else {
                    Value::Error("tcp_accept requires a TcpListener argument".to_string())
                }
            }
        }

        "tcp_connect" => {
            if arg_values.len() != 2 {
                Value::Error("tcp_connect requires (string_host, int_port) arguments".to_string())
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::Str(host)), Some(Value::Int(port))) => {
                        let address = format!("{}:{}", host.as_ref(), port);
                        match std::net::TcpStream::connect(&address) {
                            Ok(stream) => Value::TcpStream {
                                stream: Arc::new(Mutex::new(stream)),
                                peer_addr: address,
                            },
                            Err(error) => Value::ErrorObject {
                                message: format!("Failed to connect to '{}': {}", address, error),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "tcp_connect requires (string_host, int_port) arguments".to_string(),
                    ),
                }
            }
        }

        "tcp_send" => {
            if arg_values.len() != 2 {
                Value::Error(
                    "tcp_send requires (TcpStream, string_or_bytes_data) arguments".to_string(),
                )
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::TcpStream { stream, .. }), Some(Value::Str(data))) => {
                        let mut stream_guard = stream.lock().unwrap();
                        match stream_guard.write_all(data.as_ref().as_bytes()) {
                            Ok(_) => match stream_guard.flush() {
                                Ok(_) => Value::Int(data.len() as i64),
                                Err(error) => Value::ErrorObject {
                                    message: format!("Failed to flush TCP stream: {}", error),
                                    stack: Vec::new(),
                                    line: None,
                                    cause: None,
                                },
                            },
                            Err(error) => Value::ErrorObject {
                                message: format!("Failed to send data over TCP: {}", error),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    (Some(Value::TcpStream { stream, .. }), Some(Value::Bytes(data))) => {
                        let mut stream_guard = stream.lock().unwrap();
                        match stream_guard.write_all(data) {
                            Ok(_) => match stream_guard.flush() {
                                Ok(_) => Value::Int(data.len() as i64),
                                Err(error) => Value::ErrorObject {
                                    message: format!("Failed to flush TCP stream: {}", error),
                                    stack: Vec::new(),
                                    line: None,
                                    cause: None,
                                },
                            },
                            Err(error) => Value::ErrorObject {
                                message: format!("Failed to send data over TCP: {}", error),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "tcp_send requires (TcpStream, string_or_bytes_data) arguments".to_string(),
                    ),
                }
            }
        }

        "tcp_receive" => {
            if arg_values.len() != 2 {
                Value::Error("tcp_receive requires (TcpStream, int_size) arguments".to_string())
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::TcpStream { stream, .. }), Some(Value::Int(size))) => {
                        if *size <= 0 {
                            return Some(Value::Error(
                                "tcp_receive size must be positive".to_string(),
                            ));
                        }

                        let mut stream_guard = stream.lock().unwrap();
                        let mut buffer = vec![0u8; *size as usize];
                        match stream_guard.read(&mut buffer) {
                            Ok(read_size) => {
                                buffer.truncate(read_size);
                                match String::from_utf8(buffer.clone()) {
                                    Ok(text) => Value::Str(Arc::new(text)),
                                    Err(_) => Value::Bytes(buffer),
                                }
                            }
                            Err(error) => Value::ErrorObject {
                                message: format!("Failed to receive data from TCP: {}", error),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "tcp_receive requires (TcpStream, int_size) arguments".to_string(),
                    ),
                }
            }
        }

        "tcp_close" => {
            if arg_values.len() != 1 {
                Value::Error("tcp_close requires a TcpStream or TcpListener argument".to_string())
            } else {
                match arg_values.first() {
                    Some(Value::TcpStream { .. }) | Some(Value::TcpListener { .. }) => {
                        Value::Bool(true)
                    }
                    _ => Value::Error(
                        "tcp_close requires a TcpStream or TcpListener argument".to_string(),
                    ),
                }
            }
        }

        "tcp_set_nonblocking" => {
            if arg_values.len() != 2 {
                Value::Error(
                    "tcp_set_nonblocking requires (TcpStream/TcpListener, bool) arguments"
                        .to_string(),
                )
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::TcpStream { stream, .. }), Some(Value::Bool(nonblocking))) => {
                        let stream_guard = stream.lock().unwrap();
                        match stream_guard.set_nonblocking(*nonblocking) {
                            Ok(_) => Value::Bool(true),
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to set TCP stream non-blocking mode: {}",
                                    error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    (Some(Value::TcpListener { listener, .. }), Some(Value::Bool(nonblocking))) => {
                        let listener_guard = listener.lock().unwrap();
                        match listener_guard.set_nonblocking(*nonblocking) {
                            Ok(_) => Value::Bool(true),
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to set TCP listener non-blocking mode: {}",
                                    error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "tcp_set_nonblocking requires (TcpStream/TcpListener, bool) arguments"
                            .to_string(),
                    ),
                }
            }
        }

        "udp_bind" => {
            if arg_values.len() != 2 {
                Value::Error("udp_bind requires (string_host, int_port) arguments".to_string())
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::Str(host)), Some(Value::Int(port))) => {
                        let address = format!("{}:{}", host.as_ref(), port);
                        match std::net::UdpSocket::bind(&address) {
                            Ok(socket) => Value::UdpSocket {
                                socket: Arc::new(Mutex::new(socket)),
                                addr: address,
                            },
                            Err(error) => Value::ErrorObject {
                                message: format!(
                                    "Failed to bind UDP socket on '{}': {}",
                                    address, error
                                ),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "udp_bind requires (string_host, int_port) arguments".to_string(),
                    ),
                }
            }
        }

        "udp_send_to" => {
            if arg_values.len() != 4 {
                return Some(Value::Error(
                    "udp_send_to requires (UdpSocket, string_or_bytes_data, string_host, int_port) arguments"
                        .to_string(),
                ));
            }

            match (
                arg_values.first(),
                arg_values.get(1),
                arg_values.get(2),
                arg_values.get(3),
            ) {
                (
                    Some(Value::UdpSocket { socket, .. }),
                    Some(Value::Str(data)),
                    Some(Value::Str(host)),
                    Some(Value::Int(port)),
                ) => {
                    let address = format!("{}:{}", host.as_ref(), port);
                    let socket_guard = socket.lock().unwrap();
                    match socket_guard.send_to(data.as_ref().as_bytes(), &address) {
                        Ok(sent_size) => Value::Int(sent_size as i64),
                        Err(error) => Value::ErrorObject {
                            message: format!("Failed to send UDP datagram to '{}': {}", address, error),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                }
                (
                    Some(Value::UdpSocket { socket, .. }),
                    Some(Value::Bytes(data)),
                    Some(Value::Str(host)),
                    Some(Value::Int(port)),
                ) => {
                    let address = format!("{}:{}", host.as_ref(), port);
                    let socket_guard = socket.lock().unwrap();
                    match socket_guard.send_to(data, &address) {
                        Ok(sent_size) => Value::Int(sent_size as i64),
                        Err(error) => Value::ErrorObject {
                            message: format!("Failed to send UDP datagram to '{}': {}", address, error),
                            stack: Vec::new(),
                            line: None,
                            cause: None,
                        },
                    }
                }
                _ => Value::Error(
                    "udp_send_to requires (UdpSocket, string_or_bytes_data, string_host, int_port) arguments"
                        .to_string(),
                ),
            }
        }

        "udp_receive_from" => {
            if arg_values.len() != 2 {
                Value::Error(
                    "udp_receive_from requires (UdpSocket, int_size) arguments".to_string(),
                )
            } else {
                match (arg_values.first(), arg_values.get(1)) {
                    (Some(Value::UdpSocket { socket, .. }), Some(Value::Int(size))) => {
                        if *size <= 0 {
                            return Some(Value::Error(
                                "udp_receive_from size must be positive".to_string(),
                            ));
                        }

                        let socket_guard = socket.lock().unwrap();
                        let mut buffer = vec![0u8; *size as usize];
                        match socket_guard.recv_from(&mut buffer) {
                            Ok((read_size, source_address)) => {
                                buffer.truncate(read_size);
                                let data_value = match String::from_utf8(buffer.clone()) {
                                    Ok(text) => Value::Str(Arc::new(text)),
                                    Err(_) => Value::Bytes(buffer),
                                };

                                let mut result = DictMap::default();
                                result.insert(Arc::<str>::from("data"), data_value);
                                result.insert(
                                    Arc::<str>::from("from"),
                                    Value::Str(Arc::new(source_address.to_string())),
                                );
                                result
                                    .insert(Arc::<str>::from("size"), Value::Int(read_size as i64));
                                Value::Dict(Arc::new(result))
                            }
                            Err(error) => Value::ErrorObject {
                                message: format!("Failed to receive UDP datagram: {}", error),
                                stack: Vec::new(),
                                line: None,
                                cause: None,
                            },
                        }
                    }
                    _ => Value::Error(
                        "udp_receive_from requires (UdpSocket, int_size) arguments".to_string(),
                    ),
                }
            }
        }

        "udp_close" => {
            if arg_values.len() != 1 {
                Value::Error("udp_close requires a UdpSocket argument".to_string())
            } else {
                if let Some(Value::UdpSocket { .. }) = arg_values.first() {
                    Value::Bool(true)
                } else {
                    Value::Error("udp_close requires a UdpSocket argument".to_string())
                }
            }
        }

        _ => return None,
    };

    Some(result)
}
