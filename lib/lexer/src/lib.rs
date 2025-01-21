
pub mod generic;

#[derive(Debug)]
pub enum Error {
    BrokenGrammar(Option<String>),
    UnknownToken(Option<String>),
    UnexpectedEOF,
    IO(std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Lexer<T> {
    fn parse(self: &mut Self, input: impl std::io::Read) -> impl Iterator<Item=Result<T>>;

    fn parse_str(self: &mut Self, string: &str) -> impl Iterator<Item=Result<T>> {
        self.parse(std::io::Cursor::new(string))
    }
}

impl Error {
    pub fn broken_grammar() -> Self {
        Self::BrokenGrammar(None)
    }

    pub fn broken_grammar_msg(msg: &str) -> Self {
        Self::BrokenGrammar(Some(String::from(msg)))
    }

    pub fn broken_grammar_string(msg: String) -> Self {
        Self::BrokenGrammar(Some(msg))
    }

    pub fn unknown_token() -> Self {
        Self::UnknownToken(None)
    }

    pub fn unknown_token_str(token: &str) -> Self {
        Self::UnknownToken(Some(String::from(token)))
    }

    pub fn unknown_token_string(token: String) -> Self {
        Self::UnknownToken(Some(token))
    }

    pub fn unexpected_eof() -> Self {
        Self::UnexpectedEOF
    }

    pub fn io(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::IO(err) => Some(err),
            _ => None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::BrokenGrammar(msg) => write!(f,
                "Broken grammar: {}",
                msg.as_ref().map(|msg| msg.as_str()).unwrap_or("<no info>")
            ),
            Error::UnknownToken(token) => write!(f,
                "Unknown token found: {}",
                token.as_ref().map(|token| token.as_str()).unwrap_or("<not provided>")
            ),
            Error::UnexpectedEOF => write!(f,
                "Unexpected end of file"
            ),
            Error::IO(error) => write!(f,
                "Error parsing buffer: {error}"
            ),
        }
    }
}

