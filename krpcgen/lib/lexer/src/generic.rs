#[cfg(test)]
mod test;

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

        while error.is_none() && 0 == matched && 0 != active {
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
                }
            }
        }

        if let Some(error) = error {
            Some(Err(error))
        } else if 0 == matched {
            Some(Err(match last {
                Char::EOF => Error::unexpected_eof(),
                _ => Error::unknown_token(),
            }))
        } else {
            self.prev = Some(last);
            Some(Ok(
                self.matchers.iter().find_map(|m| match &m.last {
                    State::Matched(v) => Some(v),
                    _ => None,
                }).expect("Counter isn't 0").clone()
            ))
        }
    }
}

