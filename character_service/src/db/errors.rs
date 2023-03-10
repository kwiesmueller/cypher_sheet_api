use std::fmt::Display;

use grpcio::{RpcStatus, RpcStatusCode};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorCode {
    Unknown,
    NotFound,
    Internal,
    Unauthorized,
    OutOfOrder,
    Exists,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Error {
    code: ErrorCode,
    message: String,
}

impl Error {
    pub fn new(code: ErrorCode, message: &str) -> Error {
        Error {
            code,
            message: message.to_owned(),
        }
    }

    pub fn message(&self) -> String {
        self.message.to_owned()
    }

    pub fn code(&self) -> ErrorCode {
        self.code
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.message)
    }
}

impl From<Error> for RpcStatus {
    fn from(val: Error) -> Self {
        RpcStatus::with_message(val.code, val.to_string())
    }
}

impl From<ErrorCode> for RpcStatusCode {
    fn from(val: ErrorCode) -> Self {
        match val {
            ErrorCode::Unknown => RpcStatusCode::UNKNOWN,
            ErrorCode::NotFound => RpcStatusCode::NOT_FOUND,
            ErrorCode::Internal => RpcStatusCode::INTERNAL,
            ErrorCode::Unauthorized => RpcStatusCode::PERMISSION_DENIED,
            ErrorCode::OutOfOrder => RpcStatusCode::INVALID_ARGUMENT,
            ErrorCode::Exists => RpcStatusCode::ALREADY_EXISTS,
        }
    }
}
