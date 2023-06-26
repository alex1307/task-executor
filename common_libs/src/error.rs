use std::{fmt::{Display, Formatter}, string::FromUtf8Error};

use serde::Serialize;
use serde_json::to_string_pretty;
                     
use std::fmt::Error;

pub type RuntimeResult<T> = std::result::Result<T, RuntimeError>;
pub type ServiceExuctionResult<T> = std::result::Result<T, ServiceError>;
pub type TestResult<T> = std::result::Result<T, TestError>;
pub type AppResult<T> = std::result::Result<T, AppError>;
pub type FmtResult = std::result::Result<(), Error>;

#[derive(Debug, Serialize, PartialEq)]
#[allow(warnings)]
pub struct DictionaryError {
    pub message: String,
    pub error_kind: DictionaryErrorKind,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct RuntimeError {
    pub message: String,
    pub error_type: RuntimeErrorType,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum RuntimeErrorType {
    InvalidData,
    NotFound,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ServiceError {
    pub message: String,
    pub error_type: ServiceErrorType,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum ServiceErrorType {
    Failure,
    SerializationError,
    ResourceNotFound,
    IOError,
    Unavailable
}
#[derive(Debug, Serialize, PartialEq)]
pub struct TestError {
    pub message: String,
    pub error_type: TestErrorType,
}

#[derive(Debug, Serialize, PartialEq)]
pub enum TestErrorType {
    Timeout,
    WrongAnswer,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct AppError {
    pub message: String,
}

impl AppError {
    pub fn throw(msg: &str) -> Self {
        AppError {
            message: msg.to_lowercase(),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum DictionaryErrorKind {
    NotFound,
    Timeout,
    InvalidData,
    WrongAnswer,
    SerializationError,
    IOError,
}

impl Display for DictionaryError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", to_string_pretty(self).unwrap())
    }
}

impl DictionaryError {
    pub fn throw(msg: &str, error_type: DictionaryErrorKind) -> Self {
        DictionaryError { 
            message: msg.to_lowercase(),
            error_kind: error_type
        }
    }
}

impl From<FromUtf8Error> for DictionaryError {
    fn from(_error: FromUtf8Error) -> Self {
        DictionaryError::throw("utf8.failed", DictionaryErrorKind::SerializationError)
    }
}

impl From<ServiceError> for AppError {
    fn from(error: ServiceError) -> Self {
        let msg = format!("critical.error {}. Aborting the applicaiton...", error.message);
        AppError::throw(&msg)
    }
}

impl From<reqwest::Error> for DictionaryError {
    fn from(error: reqwest::Error) -> Self {
        let msg = format!(
            "Request failed. Status: {}, error: {}",
            error.status().unwrap(),
            error.to_string()
        );

        DictionaryError::throw(&msg, DictionaryErrorKind::NotFound)
    }
}

impl From<std::io::Error> for ServiceError {
    fn from(error: std::io::Error) -> Self {
        let msg = format!("io error: {}", error.to_string());
        ServiceError{
            message: msg,
            error_type: ServiceErrorType::IOError,
        }
    }
}

impl From<FromUtf8Error> for ServiceError {
    fn from(error: FromUtf8Error) -> Self {
        let msg = format!("io error: {}", error.to_string());
        ServiceError{
            message: msg,
            error_type: ServiceErrorType::SerializationError,
        }
    }
}

impl From<reqwest::Error> for ServiceError {
    fn from(error: reqwest::Error) -> Self {
        let msg = format!(
            "Request failed. Status: {}, error: {}",
            error.status().unwrap(),
            error.to_string()
        );

        ServiceError{
            message: msg,
            error_type: ServiceErrorType::ResourceNotFound,
        }
    }
}


impl From<std::io::Error> for DictionaryError {
    fn from(error: std::io::Error) -> Self {
        let msg = format!("io error: {}", error.to_string());
        DictionaryError::throw(&msg, DictionaryErrorKind::IOError)
    }
}

impl From<serde_json::Error> for DictionaryError {
    fn from(error: serde_json::Error) -> Self {
        let msg = format!("json error: {}", error.to_string());
        DictionaryError::throw(&msg, DictionaryErrorKind::SerializationError)
    }
}

impl From<serde_json::Error> for RuntimeError {
    fn from(error: serde_json::Error) -> Self {
        let msg = format!("json error: {}", error.to_string());
        RuntimeError{
            message: msg,
            error_type: RuntimeErrorType::InvalidData
        }
    }
}
