use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ReceiveError {
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
