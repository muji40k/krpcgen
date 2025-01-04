use unicode_reader::CodePoints;

use crate::{Error, Result};

pub enum State<T: Clone> {
    Rejected,
    Matching,
    Matched(T),
}

#[derive(Copy, Clone)]
pub enum Char {
    Char(char),
    EOF,
}

pub trait MatchRule<T: Clone> {
    fn get(self: &mut Self) -> Box<dyn Matcher<T>>;
}

pub trait Matcher<T: Clone> {
    fn check(self: &mut Self, c: Char) -> State<T>;
    fn reset(self: &mut Self);
}

pub trait SkipRule {
    fn get(self: &mut Self) -> Box<dyn Skip>;
}

pub trait Skip {
    fn is_skipping(self: &mut Self, c: char) -> bool;
}

impl<T: Clone, F: FnMut(Char) -> State<T> + 'static, C: FnMut() -> F> MatchRule<T> for C {
    fn get(self: &mut Self) -> Box<dyn Matcher<T>> {
        Box::new(self())
    }
}

impl<T: Clone, F: FnMut(Char) -> State<T>> Matcher<T> for F {
    fn check(self: &mut Self, c: Char) -> State<T> {
        self(c)
    }

    fn reset(self: &mut Self) {}
}

impl<T: Fn(char) -> bool + 'static, C: FnMut() -> T> SkipRule for C {
    fn get(self: &mut Self) -> Box<dyn Skip> {
        Box::new(self())
    }
}

impl<T: FnMut(char) -> bool> Skip for T {
    fn is_skipping(self: &mut Self, c: char) -> bool {
        self(c)
    }
}

pub struct Lexer<T> {
    rules: Vec<Box<dyn MatchRule<T>>>,
    skip: Option<Box<dyn SkipRule>>,
}

impl<T: Clone> Lexer<T> {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            skip: None,
        }
    }

    pub fn new_filled(rules: Vec<Box<dyn MatchRule<T>>>, skip: Box<dyn SkipRule>) -> Self {
        Self {
            rules,
            skip: Some(skip),
        }
    }

    pub fn with_skip(self: &mut Self, rule: impl SkipRule + 'static) -> &mut Self {
        self.skip = Some(Box::new(rule));
        self
    }

    pub fn push_front(self: &mut Self, rule: impl MatchRule<T> + 'static) -> &mut Self {
        self.rules.insert(0, Box::new(rule));
        self
    }

    pub fn push_back(self: &mut Self, rule: impl MatchRule<T> + 'static) -> &mut Self {
        self.rules.push(Box::new(rule));
        self
    }
}

impl<T: Clone> crate::Lexer<T> for Lexer<T> {
    fn parse(self: &mut Self, input: impl std::io::Read) -> impl Iterator<Item = Result<T>> {
        TokenIterator::new(
            &mut self.rules,
            self.skip.as_mut(),
            CodePoints::from(input),
        )
    }
}

struct MatcherState<T: Clone> {
    matcher: Box<dyn Matcher<T>>,
    last: State<T>,
}

pub struct TokenIterator<I, T: Clone>
where
    I: Iterator<Item = std::io::Result<char>>,
{
    matchers: Vec<MatcherState<T>>,
    skip: Option<Box<dyn Skip>>,
    chars: I,
    prev: Option<Char>,
}

impl<I, T: Clone> TokenIterator<I, T>
where
    I: Iterator<Item = std::io::Result<char>>,
{
    fn new(
        matchers: &mut [Box<dyn MatchRule<T>>],
        skip: Option<&mut Box<dyn SkipRule>>,
        iter: I,
    ) -> Self {
        Self {
            matchers: matchers
                .iter_mut()
                .map(|m| MatcherState {
                    matcher: m.get(),
                    last: State::Matching,
                })
                .collect(),
            skip: skip.map(|r| r.get()),
            chars: iter,
            prev: None,
        }
    }

    fn next_char(self: &mut Self) -> std::io::Result<Char> {
        if let Some(c) = self.prev.take() {
            Ok(c)
        } else {
            self.chars
                .next()
                .map(|r| r.map(Char::Char))
                .unwrap_or(Ok(Char::EOF))
        }
    }

    fn reset(self: &mut Self) {
        self.matchers.iter_mut().for_each(|m| {
            m.last = State::Matching;
            m.matcher.reset();
        })
    }
}

impl<I, T: Clone> Iterator for TokenIterator<I, T>
where
    I: Iterator<Item = std::io::Result<char>>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut active = self.matchers.len();
        let mut matched = 0;
        let mut error: Option<Error> = None;
        let mut last = Char::EOF;
        let mut empty = false;

        self.reset();

        self.skip = self.skip.take().and_then(|mut skip| {
            let mut start = false;

            while error.is_none() && !start {
                match self.next_char() {
                    Err(err) => error = Some(Error::io(err)),
                    Ok(c) => {
                        last = c;
                        start = match c {
                            Char::EOF => true,
                            Char::Char(c) => !skip.is_skipping(c),
                        };
                    },
                }
            }

            if error.is_none() {
                self.prev = Some(last);
            }

            Some(skip)
        });

        if error.is_none() {
            match self.next_char() {
                Err(err) => {
                    self.prev = None;
                    error = Some(Error::io(err));
                }
                Ok(c) => {
                    self.prev = Some(c);
                    if let Char::EOF = c {
                        empty = true;
                    }
                },
            }

            if empty {
                return None;
            }
        }

        while error.is_none() && 0 == matched {
            match self.next_char() {
                Err(err) => error = Some(Error::io(err)),
                Ok(c) => {
                    last = c;
                    self.matchers.iter_mut()
                        .filter(|m| match m.last {
                            State::Rejected => false,
                            _ => true,
                        })
                        .for_each(|m| {
                            m.last = m.matcher.check(c);
                            match m.last {
                                State::Rejected => active -= 1,
                                State::Matched(_) => {
                                    active -= 1;
                                    matched += 1;
                                }
                                _ => {}
                            }
                        });

                    if 0 == matched && 0 == active {
                        if let Char::EOF = c {
                            error = Some(Error::unexpected_eof());
                        } else {
                            error = Some(Error::unknown_token());
                        }
                    }
                }
            }
        }

        if let Some(error) = error {
            Some(Err(error))
        } else {
            self.prev = Some(last);

            let mut res = self.matchers.iter().filter_map(|m| match &m.last {
                State::Matched(v) => Some(v),
                _ => None,
            });

            if let Some(v) = res.next() {
                Some(Ok(v.clone()))
            } else {
                panic!("State and counter mismatch")
            }
        }
    }
}

#[cfg(test)]
mod test {
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
}
