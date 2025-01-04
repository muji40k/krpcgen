use crate::Lexer as TLexer;
use super::*;

#[derive(Clone, Debug, PartialEq)]
enum Token {
    Keyword(String),
    Identifier(String),
    Operator(String),
}

fn lexer() -> Lexer<Token> {
    let mut l: Lexer<Token> = Lexer::new();

    l.with_skip(|| |c| char::is_whitespace(c))
     .push_back(|| {
        let mut mem = String::new();

        move |c| {
            match c {
                Char::Char(c) => {
                    if char::is_alphabetic(c) {
                        mem += &String::from(c);
                        State::Matching
                    } else if "" == mem {
                        State::Rejected
                    } else {
                        let out = State::Matched(Token::Identifier(mem.clone()));
                        mem.clear();
                        out
                    }
                },
                Char::EOF => {
                    if "" == mem {
                        State::Rejected
                    } else {
                        let out = State::Matched(Token::Identifier(mem.clone()));
                        mem.clear();
                        out
                    }
                }
            }
        }
    }).push_back(|| {
        let mut mem = String::new();

        move |c| {
            match c {
                Char::Char(c) => {
                    if "" != mem && '=' != c && '+' != c {
                        let out = State::Matched(Token::Operator(mem.clone()));
                        mem.clear();
                        out
                    } else if "" == mem && ('=' == c || '+' == c) {
                        mem = String::from(c);
                        State::Matching
                    } else {
                        State::Rejected
                    }
                },
                Char::EOF => {
                    if "" != mem {
                        let out = State::Matched(Token::Operator(mem.clone()));
                        mem.clear();
                        out
                    } else {
                        State::Rejected
                    }
                }
            }
        }
    });

    l
}

fn lexer_with_kw() -> Lexer<Token> {
    let mut l = lexer();

    l.push_front(|| {
        let rf = ['l', 'e', 't'];
        let mut i = 0;

        move |c| {
            if 3 == i {
                i = 0;
                if let Char::Char(c) = c {
                    if char::is_alphabetic(c) {
                        State::Rejected
                    } else {
                        State::Matched(Token::Keyword("let".to_owned()))
                    }
                } else {
                    State::Matched(Token::Keyword("let".to_owned()))
                }
            } else if let Char::Char(c) = c {
                if rf[i] == c {
                    i += 1;
                    State::Matching
                } else {
                    State::Rejected
                }
            } else {
                i = 0;
                State::Rejected
            }
        }
    });

    l
}

#[test]
fn simple_test() {
    let mut l = lexer();

    let mut res = l.parse_str("a = b + c");

    assert_eq!(Token::Identifier("a".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("=".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("b".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("+".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("c".to_owned()), res.next().unwrap().unwrap());
    assert!(res.next().is_none());
}

#[test]
fn longer_ids() {
    let mut l = lexer();

    let mut res = l.parse_str("sum = first + second");

    assert_eq!(Token::Identifier("sum".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("=".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("first".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("+".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("second".to_owned()), res.next().unwrap().unwrap());
    assert!(res.next().is_none());
}

#[test]
fn more_skips() {
    let mut l = lexer();

    let mut res = l.parse_str("   sum  =first+ second   ");

    assert_eq!(Token::Identifier("sum".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("=".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("first".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("+".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("second".to_owned()), res.next().unwrap().unwrap());
    assert!(res.next().is_none());
}

#[test]
fn with_kerword() {
    let mut l = lexer_with_kw();

    let mut res = l.parse_str("let lets = first + second");

    assert_eq!(Token::Keyword("let".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("lets".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("=".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("first".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("+".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("second".to_owned()), res.next().unwrap().unwrap());
    assert!(res.next().is_none());
}

#[test]
fn unknown_token() {
    let mut l = lexer_with_kw();

    let mut res = l.parse_str("let aboba? = first + second");

    assert_eq!(Token::Keyword("let".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("aboba".to_owned()), res.next().unwrap().unwrap());
    assert!(match res.next().unwrap().unwrap_err() {
        Error::UnknownToken(None) => true,
        _ => false,
    });
    assert_eq!(Token::Operator("=".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("first".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Operator("+".to_owned()), res.next().unwrap().unwrap());
    assert_eq!(Token::Identifier("second".to_owned()), res.next().unwrap().unwrap());
    assert!(res.next().is_none());
}

