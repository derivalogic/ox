use rustatlas::prelude::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScriptingError {
    #[error("Invalid Syntax: {0}")]
    InvalidSyntax(String),
    #[error("Invalid Token: {0}")]
    InvalidToken(String),
    #[error("Error while parsing: {0}")]
    ParsingError(#[from] std::num::ParseFloatError),
    #[error("Unexpected token")]
    UnexpectedToken(String),
    #[error("Error while evaluating: {0}")]
    EvaluationError(String),
    #[error("AtlasError: {0}")]
    AtlasError(#[from] AtlasError),
}

pub type Result<T> = std::result::Result<T, ScriptingError>;

impl From<ScriptingError> for String {
    fn from(e: ScriptingError) -> Self {
        e.to_string()
    }
}
