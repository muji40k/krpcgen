
use super::{Matcher, MatchRule, State, Char, MatcherState};

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

pub struct GroupMatcher<T: Clone> {
    matchers: Vec<MatcherState<T>>,
    cooked: bool,
}

impl<T: Clone> GroupMatcher<T> {
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
            cooked: false,
        }
    }

    pub fn new_matchers(matchers: Vec<Box<dyn Matcher<T>>>) -> Self {
        Self {
            matchers: matchers.into_iter().map(MatcherState::new).collect(),
            cooked: false,
        }
    }

    pub fn add(self: &mut Self, matcher: impl Matcher<T> + 'static) -> &mut Self {
        self.matchers.push(MatcherState::new(Box::new(matcher)));
        self
    }
}

impl<T: Clone> Matcher<T> for GroupMatcher<T> {
    fn check(self: &mut Self, c: Char) -> State<T> {
        match self.cooked {
            true => State::Rejected,
            false => match self.matchers.iter_mut()
                .filter(|m| match m.last {
                    State::Rejected => false,
                    _ => true,
                })
                .map(|m| m.check(c))
                .fold((None, None, false), |mut acc, s| {
                    match (&s, acc.2) {
                        (State::Matched(_), false) => {
                            if acc.0.is_some() {        // Matchers in same
                                acc.2 = true;           // group can't compete,
                            } else {                    // if so - it's broken
                                acc.0 = Some(s);        // grammar
                            }
                        },
                        (State::Matching, false) => {
                            acc.1 = acc.1.or(Some(s))
                        },
                        _ => {},
                    };

                    acc
                }) {
                    (Some(s), _, false) => {
                        self.cooked = true;
                        s
                    },
                    (_, Some(s), false) => s,
                    _ => {
                        self.cooked = true;
                        State::Rejected
                    },
                }
        }
    }

    fn reset(self: &mut Self) {
        self.cooked = false;
        self.matchers.iter_mut().for_each(MatcherState::reset);
    }
}

pub struct SequenceMatcher<T: Clone> {
    matchers: Vec<Box<dyn Matcher<T>>>,
    last: usize,
    status: Option<State<T>>,
    cooked: bool,
}

impl<T: Clone> SequenceMatcher<T> {
    pub fn new() -> Self {
        Self {
            matchers: Vec::new(),
            last: 0,
            status: None,
            cooked: false,
        }
    }

    pub fn new_matchers(matchers: Vec<Box<dyn Matcher<T>>>) -> Self {
        Self {
            matchers,
            last: 0,
            status: None,
            cooked: false,
        }
    }

    pub fn add(self: &mut Self, matcher: impl Matcher<T> + 'static) -> &mut Self {
        self.matchers.push(Box::new(matcher));
        self
    }

    fn clear(self: &mut Self) {
        self.last = 0;
        self.cooked = false;
        self.status = None;
        self.matchers.iter_mut().for_each(|m| m.reset());
    }
}

impl<T: Clone> Matcher<T> for SequenceMatcher<T> {
    fn check(self: &mut Self, c: Char) -> State<T> {
        self.status = Some(if self.cooked {
            State::Rejected
        } else if self.last >= self.matchers.len() {
            self.cooked = true;
            State::Rejected
        } else {
            let state = self.matchers.get_mut(self.last)
                .expect("Index was checked")
                .check(c);

            match state {
                State::Matched(_) => self.last += 1,
                State::Rejected => self.cooked = true,
                _ => {},
            }

            state
        });
        self.status.as_ref().expect("Was set upper").clone()
    }

    fn reset(self: &mut Self) {
        if self.cooked || self.last >= self.matchers.len() {
            self.clear()
        } else if let Some(State::Matching) = self.status {
            self.clear()
        }
    }
}

#[macro_export]
macro_rules! creator {
    () => {
        panic!("Empty creator")
    };
    ( $matcher: expr ) => {
        || $matcher
    }
}

#[macro_export]
/// Handy macro to create Matcher group matcher
///
/// # Usage
/// ```,ignore
/// group! {
///     matcher1,
///     matcher2,
///     group! {   // Can be folded
///         matcher3,
///         matcher4,
///         ...
///     },
///     ...
/// };
/// ```
macro_rules! group {
    () => {
        panic!("Empty group")
    };
    ( $($input:tt)+ ) => {
        {
            let mut group = GroupMatcher::new();

            $crate::group_unpack!(group, $($input)+);

            group
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! group_unpack {
    ( $group:ident, ) => {};
    ( $group:ident, $matcher:expr, $( $rest:tt )* ) => {
        $group.add($matcher);
        $crate::group_unpack!($group, $($rest)*);
    };
}

#[macro_export]
/// Handy macro to create Matcher sequence matcher
///
/// # Usage
/// ```,ignore
/// sequence! {
///     matcher1,
///     matcher2,
///     sequence! {   // Can be folded
///         matcher3,
///         matcher4,
///         ...
///     },
///     ...
/// };
/// ```
macro_rules! sequence {
    () => {
        panic!("Empty sequence")
    };
    ( $($input:tt)+ ) => {
        {
            let mut sequence = SequenceMatcher::new();

            $crate::sequence_unpack!(sequence, $($input)+);

            group
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! sequence_unpack {
    ( $sequence:ident, ) => {};
    ( $sequence:ident, $matcher:expr, $( $rest:tt )* ) => {
        $sequence.add($matcher);
        $crate::sequence_unpack!($sequence, $($rest)*);
    };
}

