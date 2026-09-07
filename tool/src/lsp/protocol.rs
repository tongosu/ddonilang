use serde_json::{Map, Value};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum RpcMessage {
    Request {
        id: Value,
        method: String,
        params: Option<Value>,
    },
    Notification {
        method: String,
        params: Option<Value>,
    },
    Response {
        id: Value,
        result: Result<Value, RpcError>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<Value>,
}

impl RpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }

    pub fn with_data(code: i32, message: impl Into<String>, data: Value) -> Self {
        Self {
            code,
            message: message.into(),
            data: Some(data),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    MissingContentLength,
    DuplicateContentLength,
    InvalidHeader,
    InvalidContentLength,
    TruncatedBody,
    InvalidUtf8,
    InvalidJson,
    InvalidEnvelope,
}

impl fmt::Display for FrameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::MissingContentLength => "Content-Length header가 없습니다",
            Self::DuplicateContentLength => "Content-Length header가 중복되거나 충돌합니다",
            Self::InvalidHeader => "잘못된 JSON-RPC header입니다",
            Self::InvalidContentLength => "Content-Length가 십진수가 아닙니다",
            Self::TruncatedBody => "JSON-RPC body가 잘렸습니다",
            Self::InvalidUtf8 => "JSON-RPC body가 UTF-8이 아닙니다",
            Self::InvalidJson => "JSON-RPC body가 JSON이 아닙니다",
            Self::InvalidEnvelope => "잘못된 JSON-RPC envelope입니다",
        };
        f.write_str(message)
    }
}

impl std::error::Error for FrameError {}

#[derive(Debug, Default, Clone)]
pub struct FrameDecoder {
    buffer: Vec<u8>,
}

impl FrameDecoder {
    pub fn push(&mut self, bytes: &[u8]) -> Result<Vec<RpcMessage>, FrameError> {
        self.buffer.extend_from_slice(bytes);
        let mut messages = Vec::new();
        loop {
            let Some(header_end) = find_header_end(&self.buffer) else {
                break;
            };
            let content_length = parse_headers(&self.buffer[..header_end])?;
            let body_start = header_end + 4;
            let body_end = body_start
                .checked_add(content_length)
                .ok_or(FrameError::InvalidContentLength)?;
            if self.buffer.len() < body_end {
                break;
            }
            let body = self.buffer[body_start..body_end].to_vec();
            self.buffer.drain(..body_end);
            messages.push(parse_message(&body)?);
        }
        Ok(messages)
    }

    pub fn finish(&self) -> Result<(), FrameError> {
        if self.buffer.is_empty() {
            Ok(())
        } else {
            Err(FrameError::TruncatedBody)
        }
    }

    pub fn buffered_len(&self) -> usize {
        self.buffer.len()
    }
}

pub fn encode_message(message: &RpcMessage) -> Result<Vec<u8>, FrameError> {
    let body =
        serde_json::to_vec(&message_to_value(message)).map_err(|_| FrameError::InvalidJson)?;
    let mut frame = format!("Content-Length: {}\r\n\r\n", body.len()).into_bytes();
    frame.extend_from_slice(&body);
    Ok(frame)
}

pub fn parse_message(body: &[u8]) -> Result<RpcMessage, FrameError> {
    let value: Value = serde_json::from_slice(body).map_err(|error| {
        if error.is_data() {
            FrameError::InvalidUtf8
        } else {
            FrameError::InvalidJson
        }
    })?;
    let Value::Object(object) = value else {
        return Err(FrameError::InvalidEnvelope);
    };
    if object.get("jsonrpc") != Some(&Value::String("2.0".to_string())) {
        return Err(FrameError::InvalidEnvelope);
    }

    let has_method = object.contains_key("method");
    let has_result = object.contains_key("result");
    let has_error = object.contains_key("error");
    if has_method && (has_result || has_error) {
        return Err(FrameError::InvalidEnvelope);
    }
    if has_method {
        let Some(Value::String(method)) = object.get("method") else {
            return Err(FrameError::InvalidEnvelope);
        };
        if method.is_empty() || !valid_params(object.get("params")) {
            return Err(FrameError::InvalidEnvelope);
        }
        let params = object.get("params").cloned();
        return if let Some(id) = object.get("id") {
            Ok(RpcMessage::Request {
                id: id.clone(),
                method: method.clone(),
                params,
            })
        } else {
            Ok(RpcMessage::Notification {
                method: method.clone(),
                params,
            })
        };
    }

    if has_result == has_error || !object.contains_key("id") {
        return Err(FrameError::InvalidEnvelope);
    }
    let id = object
        .get("id")
        .cloned()
        .ok_or(FrameError::InvalidEnvelope)?;
    if has_error {
        let Some(Value::Object(error)) = object.get("error") else {
            return Err(FrameError::InvalidEnvelope);
        };
        let Some(code) = error.get("code").and_then(Value::as_i64) else {
            return Err(FrameError::InvalidEnvelope);
        };
        let Some(message) = error.get("message").and_then(Value::as_str) else {
            return Err(FrameError::InvalidEnvelope);
        };
        Ok(RpcMessage::Response {
            id,
            result: Err(RpcError {
                code: i32::try_from(code).map_err(|_| FrameError::InvalidEnvelope)?,
                message: message.to_string(),
                data: error.get("data").cloned(),
            }),
        })
    } else {
        Ok(RpcMessage::Response {
            id,
            result: Ok(object
                .get("result")
                .cloned()
                .ok_or(FrameError::InvalidEnvelope)?),
        })
    }
}

fn parse_headers(headers: &[u8]) -> Result<usize, FrameError> {
    let headers = std::str::from_utf8(headers).map_err(|_| FrameError::InvalidHeader)?;
    let mut content_length = None;
    for line in headers.split("\r\n") {
        if line.is_empty() {
            return Err(FrameError::InvalidHeader);
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(FrameError::InvalidHeader);
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value
                .trim()
                .parse::<usize>()
                .map_err(|_| FrameError::InvalidContentLength)?;
            if let Some(previous) = content_length {
                if previous != parsed {
                    return Err(FrameError::DuplicateContentLength);
                }
            } else {
                content_length = Some(parsed);
            }
        }
    }
    content_length.ok_or(FrameError::MissingContentLength)
}

fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

fn valid_params(params: Option<&Value>) -> bool {
    params
        .map(|value| value.is_array() || value.is_object())
        .unwrap_or(true)
}

fn message_to_value(message: &RpcMessage) -> Value {
    let mut object = Map::new();
    object.insert("jsonrpc".to_string(), Value::String("2.0".to_string()));
    match message {
        RpcMessage::Request { id, method, params } => {
            object.insert("id".to_string(), id.clone());
            object.insert("method".to_string(), Value::String(method.clone()));
            if let Some(params) = params {
                object.insert("params".to_string(), params.clone());
            }
        }
        RpcMessage::Notification { method, params } => {
            object.insert("method".to_string(), Value::String(method.clone()));
            if let Some(params) = params {
                object.insert("params".to_string(), params.clone());
            }
        }
        RpcMessage::Response { id, result } => {
            object.insert("id".to_string(), id.clone());
            match result {
                Ok(result) => {
                    object.insert("result".to_string(), result.clone());
                }
                Err(error) => {
                    let mut error_object = Map::new();
                    error_object.insert("code".to_string(), Value::from(error.code));
                    error_object
                        .insert("message".to_string(), Value::String(error.message.clone()));
                    if let Some(data) = &error.data {
                        error_object.insert("data".to_string(), data.clone());
                    }
                    object.insert("error".to_string(), Value::Object(error_object));
                }
            }
        }
    }
    Value::Object(object)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn partial_and_multiple_frames_are_deterministic() {
        let first = encode_message(&RpcMessage::Notification {
            method: "initialized".into(),
            params: Some(json!({})),
        })
        .unwrap();
        let second = encode_message(&RpcMessage::Request {
            id: json!(7),
            method: "shutdown".into(),
            params: None,
        })
        .unwrap();
        let mut decoder = FrameDecoder::default();
        let mut bytes = first.clone();
        bytes.extend_from_slice(&second);
        assert!(decoder.push(&bytes[..5]).unwrap().is_empty());
        assert_eq!(decoder.push(&bytes[5..]).unwrap().len(), 2);
        assert_eq!(decoder.finish(), Ok(()));
    }

    #[test]
    fn malformed_headers_fail_closed() {
        let mut decoder = FrameDecoder::default();
        assert_eq!(
            decoder.push(b"Content-Length: nope\r\n\r\n{}").unwrap_err(),
            FrameError::InvalidContentLength
        );
        let mut decoder = FrameDecoder::default();
        assert_eq!(
            decoder.push(b"X-Test: 1\r\n\r\n{}").unwrap_err(),
            FrameError::MissingContentLength
        );
    }
}
