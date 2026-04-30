use crate::lsp_code_actions;
use crate::lsp_completion::{self, CompletionItemKind};
use crate::lsp_definition;
use crate::lsp_diagnostics;
use crate::lsp_hover;
use crate::lsp_references;
use crate::lsp_rename;
use crate::formatter::{self, FormatterOptions};
use crate::lexer::{self, TokenKind};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub struct LspServerConfig {
    pub deterministic_logging: bool,
}

impl Default for LspServerConfig {
    fn default() -> Self {
        Self {
            deterministic_logging: false,
        }
    }
}

pub struct LspServer {
    documents: HashMap<String, String>,
    shutdown_requested: bool,
    exit_requested: bool,
    deterministic_logging: bool,
    log_sequence: u64,
}

impl LspServer {
    pub fn new(config: LspServerConfig) -> Self {
        Self {
            documents: HashMap::new(),
            shutdown_requested: false,
            exit_requested: false,
            deterministic_logging: config.deterministic_logging,
            log_sequence: 0,
        }
    }

    pub fn process_message(&mut self, message: &Value) -> Vec<Value> {
        self.log_json("recv", message);

        let mut outbound = Vec::new();

        if let Some(method) = message.get("method").and_then(Value::as_str) {
            if message.get("id").is_some() {
                let id = message.get("id").cloned().unwrap_or(Value::Null);
                let params = message.get("params");
                outbound.push(self.handle_request(method, id, params));
            } else {
                outbound.extend(self.handle_notification(method, message.get("params")));
            }
        }

        for item in outbound.iter() {
            self.log_json("send", item);
        }

        outbound
    }

    pub fn is_exit_requested(&self) -> bool {
        self.exit_requested
    }

    pub fn exit_code(&self) -> i32 {
        if self.shutdown_requested {
            0
        } else {
            1
        }
    }

    fn handle_request(&mut self, method: &str, id: Value, params: Option<&Value>) -> Value {
        match method {
            "initialize" => json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "capabilities": {
                        "textDocumentSync": 1,
                        "completionProvider": {
                            "resolveProvider": false,
                            "triggerCharacters": ["."]
                        },
                        "hoverProvider": true,
                        "definitionProvider": true,
                        "referencesProvider": true,
                        "renameProvider": true,
                        "codeActionProvider": true,
                        "documentFormattingProvider": true,
                        "documentRangeFormattingProvider": true,
                        "documentSymbolProvider": true,
                        "workspaceSymbolProvider": true
                    },
                    "serverInfo": {
                        "name": "ruff",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            }),
            "shutdown" => {
                self.shutdown_requested = true;
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": Value::Null
                })
            }
            "textDocument/completion" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let (line, column) = match request_position(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing position");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let items = lsp_completion::complete(&source, line, column)
                    .into_iter()
                    .map(|item| {
                        json!({
                            "label": item.label,
                            "kind": completion_item_kind_to_lsp(item.kind),
                        })
                    })
                    .collect::<Vec<Value>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "isIncomplete": false,
                        "items": items,
                    }
                })
            }
            "textDocument/hover" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let (line, column) = match request_position(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing position");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let result = lsp_hover::hover(&source, line, column).map(|info| {
                    json!({
                        "contents": {
                            "kind": "markdown",
                            "value": info.detail,
                        },
                        "range": {
                            "start": {
                                "line": zero_based(info.line),
                                "character": zero_based(info.column),
                            },
                            "end": {
                                "line": zero_based(info.line),
                                "character": zero_based(info.column + info.symbol.chars().count()),
                            }
                        }
                    })
                });

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result.unwrap_or(Value::Null)
                })
            }
            "textDocument/definition" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let (line, column) = match request_position(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing position");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let result = lsp_definition::find_definition(&source, line, column)
                    .map(|location| {
                        json!({
                            "uri": uri,
                            "range": {
                                "start": {
                                    "line": zero_based(location.line),
                                    "character": zero_based(location.column),
                                },
                                "end": {
                                    "line": zero_based(location.line),
                                    "character": zero_based(location.column + location.name.chars().count()),
                                }
                            }
                        })
                    })
                    .unwrap_or(Value::Null);

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": result
                })
            }
            "textDocument/references" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let (line, column) = match request_position(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing position");
                    }
                };

                let include_definition = params
                    .and_then(|value| value.get("context"))
                    .and_then(|value| value.get("includeDeclaration"))
                    .and_then(Value::as_bool)
                    .unwrap_or(true);

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let references = lsp_references::find_references(
                    &source,
                    line,
                    column,
                    include_definition,
                )
                .into_iter()
                .map(|reference| {
                    json!({
                        "uri": uri,
                        "range": {
                            "start": {
                                "line": zero_based(reference.line),
                                "character": zero_based(reference.column),
                            },
                            "end": {
                                "line": zero_based(reference.line),
                                "character": zero_based(reference.column + 1),
                            }
                        }
                    })
                })
                .collect::<Vec<Value>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": references
                })
            }
            "textDocument/rename" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let (line, column) = match request_position(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing position");
                    }
                };
                let new_name = match params
                    .and_then(|value| value.get("newName"))
                    .and_then(Value::as_str)
                {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing newName");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let rename_result = match lsp_rename::rename_symbol(&source, line, column, new_name)
                {
                    Ok(value) => value,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let edits = rename_result
                    .edits
                    .into_iter()
                    .map(|edit| {
                        json!({
                            "range": {
                                "start": {
                                    "line": zero_based(edit.line),
                                    "character": zero_based(edit.column),
                                },
                                "end": {
                                    "line": zero_based(edit.line),
                                    "character": zero_based(edit.column + edit.old_name.chars().count()),
                                }
                            },
                            "newText": edit.new_name,
                        })
                    })
                    .collect::<Vec<Value>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "changes": {
                            uri: edits,
                        }
                    }
                })
            }
            "textDocument/codeAction" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };
                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let actions = lsp_code_actions::code_actions(&source)
                    .into_iter()
                    .map(|action| {
                        let action_uri = uri.clone();
                        json!({
                            "title": action.title,
                            "kind": "quickfix",
                            "edit": {
                                "changes": {
                                    action_uri: [{
                                        "range": {
                                            "start": {
                                                "line": zero_based(action.line),
                                                "character": zero_based(action.column),
                                            },
                                            "end": {
                                                "line": zero_based(action.line),
                                                "character": zero_based(action.column + 1),
                                            }
                                        },
                                        "newText": action.replacement,
                                    }]
                                }
                            }
                        })
                    })
                    .collect::<Vec<Value>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": actions
                })
            }
            "textDocument/formatting" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let options = formatter_options_from_lsp_params(params);
                let formatted = formatter::format_source(&source, &options);

                let edits = if formatted == source {
                    Vec::new()
                } else {
                    vec![full_document_text_edit(&source, formatted)]
                };

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": edits
                })
            }
            "textDocument/rangeFormatting" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let options = formatter_options_from_lsp_params(params);
                let formatted = formatter::format_source(&source, &options);
                let edits = if formatted == source {
                    Vec::new()
                } else {
                    vec![full_document_text_edit(&source, formatted)]
                };

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": edits
                })
            }
            "textDocument/documentSymbol" => {
                let uri = match request_uri(params) {
                    Some(value) => value,
                    None => {
                        return invalid_params_response(id, "Missing textDocument.uri");
                    }
                };

                let source = match self.resolve_document_source(&uri) {
                    Ok(content) => content,
                    Err(message) => {
                        return invalid_params_response(id, &message);
                    }
                };

                let symbols = collect_document_symbols(&source)
                    .into_iter()
                    .map(|symbol| {
                        json!({
                            "name": symbol.name,
                            "kind": symbol.kind,
                            "range": {
                                "start": {
                                    "line": zero_based(symbol.line),
                                    "character": zero_based(symbol.column),
                                },
                                "end": {
                                    "line": zero_based(symbol.line),
                                    "character": zero_based(symbol.column + symbol.length),
                                }
                            },
                            "selectionRange": {
                                "start": {
                                    "line": zero_based(symbol.line),
                                    "character": zero_based(symbol.column),
                                },
                                "end": {
                                    "line": zero_based(symbol.line),
                                    "character": zero_based(symbol.column + symbol.length),
                                }
                            }
                        })
                    })
                    .collect::<Vec<Value>>();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": symbols
                })
            }
            "workspace/symbol" => {
                let query = params
                    .and_then(|value| value.get("query"))
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_lowercase();

                let mut symbols = Vec::new();
                for (uri, source) in self.documents.iter() {
                    for symbol in collect_document_symbols(source).into_iter() {
                        if !query.is_empty() && !symbol.name.to_lowercase().contains(&query) {
                            continue;
                        }

                        symbols.push(json!({
                            "name": symbol.name,
                            "kind": symbol.kind,
                            "location": {
                                "uri": uri,
                                "range": {
                                    "start": {
                                        "line": zero_based(symbol.line),
                                        "character": zero_based(symbol.column),
                                    },
                                    "end": {
                                        "line": zero_based(symbol.line),
                                        "character": zero_based(symbol.column + symbol.length),
                                    }
                                }
                            }
                        }));
                    }
                }

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": symbols
                })
            }
            _ => method_not_found_response(id, method),
        }
    }

    fn handle_notification(&mut self, method: &str, params: Option<&Value>) -> Vec<Value> {
        match method {
            "initialized" => Vec::new(),
            "exit" => {
                self.exit_requested = true;
                Vec::new()
            }
            "textDocument/didOpen" => {
                let uri = params
                    .and_then(|value| value.get("textDocument"))
                    .and_then(|value| value.get("uri"))
                    .and_then(Value::as_str)
                    .map(|value| value.to_string());
                let text = params
                    .and_then(|value| value.get("textDocument"))
                    .and_then(|value| value.get("text"))
                    .and_then(Value::as_str)
                    .map(|value| value.to_string());

                match (uri, text) {
                    (Some(uri_value), Some(text_value)) => {
                        self.documents.insert(uri_value.clone(), text_value);
                        vec![self.publish_diagnostics_notification(&uri_value)]
                    }
                    _ => Vec::new(),
                }
            }
            "textDocument/didChange" => {
                let uri = params
                    .and_then(|value| value.get("textDocument"))
                    .and_then(|value| value.get("uri"))
                    .and_then(Value::as_str)
                    .map(|value| value.to_string());
                let new_text = params
                    .and_then(|value| value.get("contentChanges"))
                    .and_then(Value::as_array)
                    .and_then(|changes| changes.first())
                    .and_then(|change| change.get("text"))
                    .and_then(Value::as_str)
                    .map(|value| value.to_string());

                match (uri, new_text) {
                    (Some(uri_value), Some(text_value)) => {
                        self.documents.insert(uri_value.clone(), text_value);
                        vec![self.publish_diagnostics_notification(&uri_value)]
                    }
                    _ => Vec::new(),
                }
            }
            "textDocument/didClose" => {
                if let Some(uri) = params
                    .and_then(|value| value.get("textDocument"))
                    .and_then(|value| value.get("uri"))
                    .and_then(Value::as_str)
                {
                    self.documents.remove(uri);
                    return vec![json!({
                        "jsonrpc": "2.0",
                        "method": "textDocument/publishDiagnostics",
                        "params": {
                            "uri": uri,
                            "diagnostics": [],
                        }
                    })];
                }

                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    fn publish_diagnostics_notification(&self, uri: &str) -> Value {
        let source = self
            .documents
            .get(uri)
            .cloned()
            .unwrap_or_else(String::new);
        let diagnostics = lsp_diagnostics::diagnose(&source)
            .into_iter()
            .map(|diagnostic| {
                json!({
                    "range": {
                        "start": {
                            "line": zero_based(diagnostic.line),
                            "character": zero_based(diagnostic.column),
                        },
                        "end": {
                            "line": zero_based(diagnostic.line),
                            "character": zero_based(diagnostic.column + 1),
                        }
                    },
                    "severity": 1,
                    "source": "ruff",
                    "message": diagnostic.message,
                })
            })
            .collect::<Vec<Value>>();

        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/publishDiagnostics",
            "params": {
                "uri": uri,
                "diagnostics": diagnostics,
            }
        })
    }

    fn resolve_document_source(&self, uri: &str) -> Result<String, String> {
        if let Some(existing) = self.documents.get(uri) {
            return Ok(existing.clone());
        }

        let path = file_uri_to_path(uri)
            .ok_or_else(|| format!("Unable to resolve URI '{}' to a local file", uri))?;

        fs::read_to_string(&path)
            .map_err(|error| format!("Failed to read '{}': {}", path.display(), error))
    }

    fn log_json(&mut self, direction: &str, value: &Value) {
        if !self.deterministic_logging {
            return;
        }

        let serialized = match serde_json::to_string(value) {
            Ok(content) => content,
            Err(_) => "{\"error\":\"serialization\"}".to_string(),
        };

        eprintln!("lsp-log[{}] {} {}", self.log_sequence, direction, serialized);
        self.log_sequence += 1;
    }
}

pub fn run_stdio_server(config: LspServerConfig) -> i32 {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut reader = BufReader::new(stdin.lock());
    let mut writer = stdout.lock();

    let mut server = LspServer::new(config);

    loop {
        let payload = match read_lsp_message(&mut reader) {
            Ok(Some(message)) => message,
            Ok(None) => break,
            Err(error) => {
                eprintln!("LSP transport read error: {}", error);
                return 1;
            }
        };

        let value: Value = match serde_json::from_str(&payload) {
            Ok(parsed) => parsed,
            Err(error) => {
                eprintln!("Invalid LSP JSON payload: {}", error);
                continue;
            }
        };

        let responses = server.process_message(&value);
        for response in responses.iter() {
            if let Err(error) = write_lsp_message(&mut writer, response) {
                eprintln!("LSP transport write error: {}", error);
                return 1;
            }
        }

        if server.is_exit_requested() {
            return server.exit_code();
        }
    }

    server.exit_code()
}

fn read_lsp_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length: Option<usize> = None;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;

        if read == 0 {
            if content_length.is_none() {
                return Ok(None);
            }
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "EOF before LSP payload body",
            ));
        }

        if line == "\r\n" || line == "\n" {
            break;
        }

        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("Content-Length:") {
            content_length = value.trim().parse::<usize>().ok();
        }
    }

    let size = match content_length {
        Some(value) => value,
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Missing Content-Length header",
            ));
        }
    };

    let mut buffer = vec![0u8; size];
    reader.read_exact(&mut buffer)?;

    String::from_utf8(buffer)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))
}

fn write_lsp_message<W: Write>(writer: &mut W, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_string(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error.to_string()))?;

    write!(writer, "Content-Length: {}\r\n\r\n{}", payload.len(), payload)?;
    writer.flush()
}

fn request_uri(params: Option<&Value>) -> Option<String> {
    params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
        .map(|value| value.to_string())
}

fn request_position(params: Option<&Value>) -> Option<(usize, usize)> {
    let line = params
        .and_then(|value| value.get("position"))
        .and_then(|value| value.get("line"))
        .and_then(Value::as_u64)? as usize;
    let column = params
        .and_then(|value| value.get("position"))
        .and_then(|value| value.get("character"))
        .and_then(Value::as_u64)? as usize;

    Some((line + 1, column + 1))
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    if !uri.starts_with("file://") {
        return None;
    }

    let encoded_path = &uri[7..];
    let decoded = urlencoding::decode(encoded_path).ok()?;

    #[cfg(windows)]
    {
        let normalized = decoded.trim_start_matches('/').replace('/', "\\");
        return Some(PathBuf::from(normalized));
    }

    #[cfg(not(windows))]
    {
        Some(PathBuf::from(decoded.into_owned()))
    }
}

fn zero_based(one_based: usize) -> usize {
    one_based.saturating_sub(1)
}

fn completion_item_kind_to_lsp(kind: CompletionItemKind) -> u8 {
    match kind {
        CompletionItemKind::Builtin => 3,
        CompletionItemKind::Function => 3,
        CompletionItemKind::Variable => 6,
    }
}

#[derive(Debug, Clone)]
struct SymbolEntry {
    name: String,
    line: usize,
    column: usize,
    length: usize,
    kind: u8,
}

fn formatter_options_from_lsp_params(params: Option<&Value>) -> FormatterOptions {
    let tab_size = params
        .and_then(|value| value.get("options"))
        .and_then(|value| value.get("tabSize"))
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or(4);

    FormatterOptions {
        indent_width: tab_size.max(1),
        line_length: 100,
        sort_imports: true,
    }
}

fn full_document_text_edit(source: &str, new_text: String) -> Value {
    let (end_line, end_character) = source_end_position(source);
    json!({
        "range": {
            "start": {
                "line": 0,
                "character": 0,
            },
            "end": {
                "line": end_line,
                "character": end_character,
            }
        },
        "newText": new_text,
    })
}

fn source_end_position(source: &str) -> (usize, usize) {
    if source.is_empty() {
        return (0, 0);
    }

    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return (0, 0);
    }

    let end_line = lines.len().saturating_sub(1);
    let end_character = lines[end_line].chars().count();
    (end_line, end_character)
}

fn collect_document_symbols(source: &str) -> Vec<SymbolEntry> {
    let mut symbols = Vec::new();
    let tokens = lexer::tokenize(source);
    let mut index = 0;

    while index < tokens.len() {
        let token = &tokens[index];
        match &token.kind {
            TokenKind::Keyword(keyword) if keyword == "func" => {
                if let Some(name_token) = tokens.get(index + 1) {
                    if let TokenKind::Identifier(name) = &name_token.kind {
                        let start = name_token.column.saturating_sub(name.chars().count());
                        if start > 0 {
                            symbols.push(SymbolEntry {
                                name: name.clone(),
                                line: name_token.line,
                                column: start,
                                length: name.chars().count(),
                                kind: 12,
                            });
                        }
                    }
                }
            }
            TokenKind::Keyword(keyword) if keyword == "let" || keyword == "const" => {
                let mut name_index = index + 1;
                if keyword == "let" {
                    if let Some(next_token) = tokens.get(name_index) {
                        if let TokenKind::Keyword(next_keyword) = &next_token.kind {
                            if next_keyword == "mut" {
                                name_index += 1;
                            }
                        }
                    }
                }

                if let Some(name_token) = tokens.get(name_index) {
                    if let TokenKind::Identifier(name) = &name_token.kind {
                        let start = name_token.column.saturating_sub(name.chars().count());
                        if start > 0 {
                            symbols.push(SymbolEntry {
                                name: name.clone(),
                                line: name_token.line,
                                column: start,
                                length: name.chars().count(),
                                kind: 13,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        index += 1;
    }

    symbols
}

fn method_not_found_response(id: Value, method: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32601,
            "message": format!("Method '{}' is not supported", method),
        }
    })
}

fn invalid_params_response(id: Value, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": -32602,
            "message": message,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::{LspServer, LspServerConfig};
    use serde_json::{json, Value};

    #[test]
    fn lifecycle_initialize_shutdown_exit_returns_clean_exit() {
        let mut server = LspServer::new(LspServerConfig::default());

        let initialize_response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }));
        assert_eq!(initialize_response.len(), 1);
        assert!(initialize_response[0].get("result").is_some());

        let initialized_response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }));
        assert!(initialized_response.is_empty());

        let shutdown_response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "shutdown",
            "params": Value::Null
        }));
        assert_eq!(shutdown_response.len(), 1);
        assert!(shutdown_response[0].get("result").is_some());

        let exit_response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "exit"
        }));
        assert!(exit_response.is_empty());
        assert!(server.is_exit_requested());
        assert_eq!(server.exit_code(), 0);
    }

    #[test]
    fn lifecycle_exit_without_shutdown_returns_failure_exit() {
        let mut server = LspServer::new(LspServerConfig::default());
        let response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "exit"
        }));

        assert!(response.is_empty());
        assert!(server.is_exit_requested());
        assert_eq!(server.exit_code(), 1);
    }

    #[test]
    fn did_open_publishes_diagnostics_for_document() {
        let mut server = LspServer::new(LspServerConfig::default());

        let notifications = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/test.ruff",
                    "text": "print((1 + 2)\n"
                }
            }
        }));

        assert_eq!(notifications.len(), 1);
        assert_eq!(
            notifications[0].get("method").and_then(|value| value.as_str()),
            Some("textDocument/publishDiagnostics")
        );
        assert!(notifications[0]
            .get("params")
            .and_then(|value| value.get("diagnostics"))
            .and_then(|value| value.as_array())
            .map(|items| !items.is_empty())
            .unwrap_or(false));
    }

    #[test]
    fn completion_request_uses_shared_analysis_logic() {
        let mut server = LspServer::new(LspServerConfig::default());

        let _ = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/complete.ruff",
                    "text": "let printer := 1\npr\n"
                }
            }
        }));

        let response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "textDocument/completion",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/complete.ruff"
                },
                "position": {
                    "line": 1,
                    "character": 2
                }
            }
        }));

        assert_eq!(response.len(), 1);
        let items = response[0]
            .get("result")
            .and_then(|value| value.get("items"))
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();

        assert!(items.iter().any(|item| {
            item.get("label").and_then(|value| value.as_str()) == Some("print")
        }));
        assert!(items.iter().any(|item| {
            item.get("label").and_then(|value| value.as_str()) == Some("printer")
        }));
    }

    #[test]
    fn formatting_request_returns_text_edit_when_source_changes() {
        let mut server = LspServer::new(LspServerConfig::default());

        let _ = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/format.ruff",
                    "text": "let value:=1\n"
                }
            }
        }));

        let response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "textDocument/formatting",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/format.ruff"
                },
                "options": {
                    "tabSize": 4,
                    "insertSpaces": true
                }
            }
        }));

        let edits = response[0]
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert_eq!(edits.len(), 1);
        assert!(edits[0]
            .get("newText")
            .and_then(Value::as_str)
            .map(|text| text.contains("let value := 1"))
            .unwrap_or(false));
    }

    #[test]
    fn document_symbol_returns_function_and_variable_entries() {
        let mut server = LspServer::new(LspServerConfig::default());

        let _ = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/symbols.ruff",
                    "text": "func greet(name) {\n    return name\n}\nlet value := 1\n"
                }
            }
        }));

        let response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "textDocument/documentSymbol",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/symbols.ruff"
                }
            }
        }));

        let symbols = response[0]
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert!(symbols.iter().any(|symbol| {
            symbol.get("name").and_then(Value::as_str) == Some("greet")
        }));
        assert!(symbols.iter().any(|symbol| {
            symbol.get("name").and_then(Value::as_str) == Some("value")
        }));
    }

    #[test]
    fn workspace_symbol_filters_results_by_query() {
        let mut server = LspServer::new(LspServerConfig::default());

        let _ = server.process_message(&json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": "file:///tmp/workspace_symbols.ruff",
                    "text": "func render_page() {\n    return 1\n}\nfunc build_site() {\n    return 2\n}\n"
                }
            }
        }));

        let response = server.process_message(&json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "workspace/symbol",
            "params": {
                "query": "render"
            }
        }));

        let symbols = response[0]
            .get("result")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();

        assert!(symbols.iter().any(|symbol| {
            symbol.get("name").and_then(Value::as_str) == Some("render_page")
        }));
        assert!(!symbols.iter().any(|symbol| {
            symbol.get("name").and_then(Value::as_str) == Some("build_site")
        }));
    }
}
