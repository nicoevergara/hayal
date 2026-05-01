#[derive(Debug)]
pub enum LexcError {
    ParsingError,
}

impl<'a> nom::error::ParseError<&'a str> for LexcError {
    fn from_error_kind(_input: &'a str, _kind: nom::error::ErrorKind) -> Self {
        LexcError::ParsingError
    }

    fn append(_input: &'a str, _kind: nom::error::ErrorKind, sub: Self) -> Self {
        sub
    }
}
