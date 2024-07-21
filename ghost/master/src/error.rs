use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum ShioriError {
  UndefinedVariable,
  ParseIntError,
  SystemTimeError,
  FieldAccessError,
  ArrayAccessError,
  TranslaterNotReadyError,
  TalkNotFound,
  ParseRequestError,
}

impl fmt::Display for ShioriError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      ShioriError::UndefinedVariable => write!(f, "Undefined Variable"),
      ShioriError::ParseIntError => write!(f, "Parse Int Error"),
      ShioriError::SystemTimeError => write!(f, "SystemTime Error"),
      ShioriError::FieldAccessError => write!(f, "Field Access Error"),
      ShioriError::ArrayAccessError => write!(f, "Access Vec Error"),
      ShioriError::TranslaterNotReadyError => write!(f, "Translater Not Ready Error"),
      ShioriError::TalkNotFound => write!(f, "Talk Not Found"),
      ShioriError::ParseRequestError => write!(f, "Parse Request Error"),
    }
  }
}

impl Error for ShioriError {}
