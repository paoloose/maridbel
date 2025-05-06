use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq, Eq)]
pub enum ReceiveError {
    Closed,
    Other(String),
}

#[derive(Debug, PartialEq, Eq)]
pub enum SendError {
    Closed,
    Other(String),
}

impl Error for ReceiveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for ReceiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReceiveError::Closed => write!(f, "Channel closed"),
            ReceiveError::Other(msg) => write!(f, "Cannot wait for message: {}", msg),
        }
    }
}

impl Error for SendError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    fn description(&self) -> &str {
        "description() is deprecated; use Display"
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }
}

impl Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendError::Closed => write!(f, "Channel closed"),
            SendError::Other(msg) => write!(f, "Cannot wait for message: {}", msg),
        }
    }
}
