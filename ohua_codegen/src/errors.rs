use std::error::Error;
use std::fmt;
use std::io;
use syn::parse::Error as ParseError;

#[derive(Debug)]
pub enum TypeExtractionError {
    IOError(io::Error),
    ParsingError(ParseError),
}

impl fmt::Display for TypeExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TypeExtractionError::*;
        match *self {
            IOError(ref err) => write!(f, "a filesystem related error occured: {}", err),
            ParsingError(ref err) => write!(f, "could not parse file: {}", err),
        }
    }
}

impl From<io::Error> for TypeExtractionError {
    fn from(e: io::Error) -> Self {
        TypeExtractionError::IOError(e)
    }
}

impl From<ParseError> for TypeExtractionError {
    fn from(e: ParseError) -> Self {
        TypeExtractionError::ParsingError(e)
    }
}

impl Error for TypeExtractionError {
    fn description(&self) -> &str {
        // TODO
        ""
    }
}
