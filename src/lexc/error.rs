#[derive(Debug)]
pub enum LexcError {
    ParsingError,
}

impl<'a> nom::error::ParseError<&'a str> for LexcError {
    fn from_error_kind(input: &'a str, kind: nom::error::ErrorKind) -> Self {
        LexcError::ParsingError
    }

    fn append(input: &'a str, kind: nom::error::ErrorKind, sub: Self) -> Self {
        sub
    }
}
