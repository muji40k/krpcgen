
use lexer::{
    group,
    generic::{
        Matcher,
        State,
        Char,
        matcher::{
            CharSequenceMatcher,
            IntegerMatcher,
        },
    },
};

use crate::token;

pub fn allowed_chars(c: Char) -> bool {
    match c {
        Char::EOF => false,
        Char::Char(c) => char::is_alphabetic(c)
            || '_' == c
            || char::is_digit(c, 10),
    }
}

pub fn comment_matcher() -> impl Matcher<token::Token> {
    CommentMatcher::new()
}

pub fn identifier_matcher() -> impl Matcher<token::Token> {
    IdentifierMatcher::new()
}

pub fn literal_matcher() -> impl Matcher<token::Token> {
    IntegerMatcher::new(|n| token::Token::Literal(token::Literal::Integer(n)))
}

pub fn bracket_matcher() -> impl Matcher<token::Token> {
    group! {
        sbracket_matcher(token::Bracket::Left),
        sbracket_matcher(token::Bracket::Right),
        sbracket_matcher(token::Bracket::LeftCurly),
        sbracket_matcher(token::Bracket::RightCurly),
        sbracket_matcher(token::Bracket::LeftSquare),
        sbracket_matcher(token::Bracket::RightSquare),
        sbracket_matcher(token::Bracket::LeftTriangle),
        sbracket_matcher(token::Bracket::RightTriangle),
    }
}

fn sbracket_matcher(value: token::Bracket) -> impl Matcher<token::Token> {
    CharSequenceMatcher::new(
        &value.to_string(),
        move || token::Token::Bracket(value),
        |_| false,
    )
}

pub fn type_matcher() -> impl Matcher<token::Token> {
    group! {
        stype_matcher(token::Type::Void),
        stype_matcher(token::Type::Unsigned),
        stype_matcher(token::Type::Integer),
        stype_matcher(token::Type::Hyper),
        stype_matcher(token::Type::Float),
        stype_matcher(token::Type::Double),
        stype_matcher(token::Type::Boolean),
        stype_matcher(token::Type::Quadruple),
        stype_matcher(token::Type::String),
        stype_matcher(token::Type::Opaque),
        stype_matcher(token::Type::Pointer),
        stype_matcher(token::Type::Enum),
        stype_matcher(token::Type::Struct),
        stype_matcher(token::Type::Union),
    }
}

fn stype_matcher(value: token::Type) -> impl Matcher<token::Token> {
    CharSequenceMatcher::new(
        &value.to_string(),
        move || token::Token::Type(value),
        match value {
            token::Type::Pointer => |_| false,
            _ => allowed_chars,
        }
    )
}

pub fn operator_matcher() -> impl Matcher<token::Token> {
    soperator_matcher(token::Operator::Assign)
}

fn soperator_matcher(value: token::Operator) -> impl Matcher<token::Token> {
    CharSequenceMatcher::new(
        &value.to_string(),
        move || token::Token::Operator(value),
        |_| false,
    )
}

pub fn separator_matcher() -> impl Matcher<token::Token> {
    group! {
        sseparator_matcher(token::Separator::Semicolon),
        sseparator_matcher(token::Separator::Colon),
        sseparator_matcher(token::Separator::Comma),
    }
}

fn sseparator_matcher(value: token::Separator) -> impl Matcher<token::Token> {
    CharSequenceMatcher::new(
        &value.to_string(),
        move || token::Token::Separator(value),
        |_| false,
    )
}

pub fn keyword_matcher() -> impl Matcher<token::Token> {
    group! {
        skeyword_matcher(token::Keyword::Const),
        skeyword_matcher(token::Keyword::Case),
        skeyword_matcher(token::Keyword::Switch),
        skeyword_matcher(token::Keyword::Default),
        skeyword_matcher(token::Keyword::Typedef),
        skeyword_matcher(token::Keyword::Program),
        skeyword_matcher(token::Keyword::Version),
        skeyword_matcher(token::Keyword::Procedure),
    }
}

fn skeyword_matcher(value: token::Keyword) -> impl Matcher<token::Token> {
    CharSequenceMatcher::new(
        &value.to_string(),
        move || token::Token::Keyword(value),
        allowed_chars
    )
}

struct IdentifierMatcher {
    current: String,
    cooked: bool,
}

impl IdentifierMatcher {
    fn new() -> Self {
        Self {
            current: String::new(),
            cooked: false,
        }
    }
}

impl Matcher<token::Token> for IdentifierMatcher {
    fn check(self: &mut Self, c: Char) -> State<token::Token> {
        if self.cooked {
            return State::Rejected
        }

        let res = match c {
            Char::EOF => {
                if "" == self.current {
                    State::Rejected
                } else {
                    State::Matched(
                        token::Token::Identifier(self.current.clone())
                    )
                }
            },
            Char::Char(c) => {
                if "" == self.current {
                    if !char::is_alphanumeric(c) {
                        State::Rejected
                    } else {
                        let mut buf: [u8; 4] = [0; 4];
                        self.current += c.encode_utf8(&mut buf);
                        State::Matching
                    }
                } else if allowed_chars(Char::Char(c)) {
                    let mut buf: [u8; 4] = [0; 4];
                    self.current += c.encode_utf8(&mut buf);
                    State::Matching
                } else if "" != self.current {
                    State::Matched(
                        token::Token::Identifier(self.current.clone())
                    )
                } else {
                    State::Rejected
                }
            },
        };

        if let State::Rejected | State::Matched(_) = res {
            self.cooked = true
        }

        res
    }

    fn reset(self: &mut Self) {
        self.current.clear();
        self.cooked = false;
    }
}

enum CommentType {
    None,
    Pending,
    Block,
    PendingBlockEnd,
    Line,
    Finished,
}

struct CommentMatcher {
    cooked: bool,
    content: String,
    ctype: CommentType,
}

impl CommentMatcher {
    fn new() -> Self {
        Self {
            cooked: false,
            content: String::new(),
            ctype: CommentType::None,
        }
    }
}

impl Matcher<token::Token> for CommentMatcher {
    fn check(self: &mut Self, c: Char) -> State<token::Token> {
        if self.cooked {
            return State::Rejected
        }

        let res = match (c, &self.ctype) {
            (_, CommentType::Finished) => State::Matched(
                token::Token::Comment(self.content.clone())
            ),
            (
                Char::EOF,
                CommentType::None | CommentType::Pending
                | CommentType::Block | CommentType::PendingBlockEnd
            ) => State::Rejected,
            (Char::EOF, CommentType::Line) => State::Matched(
                token::Token::Comment(self.content.clone())
            ),
            (Char::Char('\r' | '\n'), CommentType::Line) => {
                self.ctype = CommentType::Finished;
                State::Matching
            },
            (Char::Char('/'), CommentType::None) => {
                self.ctype = CommentType::Pending;
                State::Matching
            },
            (Char::Char('/'), CommentType::Pending) => {
                self.ctype = CommentType::Line;
                State::Matching
            },
            (Char::Char('*'), CommentType::Pending) => {
                self.ctype = CommentType::Block;
                State::Matching
            },
            (_, CommentType::Pending) => {
                State::Rejected
            },
            (Char::Char('*'), CommentType::Block) => {
                self.ctype = CommentType::PendingBlockEnd;
                State::Matching
            },
            (Char::Char('/'), CommentType::PendingBlockEnd) => {
                self.ctype = CommentType::Finished;
                State::Matching
            },
            (Char::Char(c), CommentType::PendingBlockEnd) => {
                let mut buf: [u8; 4] = [0; 4];
                self.content += "*";
                self.content += c.encode_utf8(&mut buf);
                self.ctype = CommentType::Block;
                State::Matching
            },
            (Char::Char(c), CommentType::Block | CommentType::Line) => {
                let mut buf: [u8; 4] = [0; 4];
                self.content += c.encode_utf8(&mut buf);
                State::Matching
            },
            _ => State::Rejected,
        };

        if let State::Rejected | State::Matched(_) = res {
            self.cooked = true;
        }

        res
    }

    fn reset(self: &mut Self) {
        self.cooked = false;
        self.content.clear();
        self.ctype = CommentType::None;
    }
}

