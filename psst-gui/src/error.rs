use core::{error, fmt};

use druid::Data;

#[derive(Clone, Debug, Data)]
pub enum Error {
    WebApiError(String),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WebApiError(err) => f.write_str(err),
        }
    }
}
