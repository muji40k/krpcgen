#[cfg(test)]
mod test;
pub mod matcher;
pub mod skip;

use unicode_reader::CodePoints;

use crate::{Error, Result};

#[derive(Clone)]
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

impl<T: Clone> MatcherState<T> {
    fn new(matcher: Box<dyn Matcher<T>>) -> Self {
        Self {
            matcher,
            last: State::Matching,
        }
    }
}

impl<T: Clone> Matcher<T> for MatcherState<T> {
    fn check(self: &mut Self, c: Char) -> State<T> {
        self.last = self.matcher.check(c);
        self.last.clone()
    }

    fn reset(self: &mut Self) {
        self.last = State::Matching;
        self.matcher.reset();
    }
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
            matchers: matchers.iter_mut()
                .map(|m| MatcherState::new(m.get()))
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
            self.chars.next()
                .map(|r| r.map(Char::Char))
                .unwrap_or(Ok(Char::EOF))
        }
    }

    fn reset(self: &mut Self) {
        self.matchers.iter_mut().for_each(MatcherState::reset)
    }
}

enum MatchLock {
    None,
    Matching,
    Matched,
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
            let mut matching = MatchLock::None;

            match self.next_char() {
                Err(err) => error = Some(Error::io(err)),
                Ok(c) => {
                    last = c;
                    self.matchers.iter_mut()
                        .filter(|m| match m.last {
                            State::Rejected => false,
                            _ => true,
                        })
                        .for_each(|m| match m.check(c) {
                            State::Rejected => active -= 1,
                            State::Matched(_) => {
                                if let MatchLock::Matching = matching {
                                    error = Some(Error::broken_grammar_msg(
                                        "Unreacheable higher order rule found"
                                    ));
                                } else {
                                    matching = MatchLock::Matched;
                                    active -= 1;
                                    matched += 1;
                                }
                            },
                            State::Matching => {
                                if let MatchLock::None = matching {
                                    matching = MatchLock::Matching;
                                }
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

