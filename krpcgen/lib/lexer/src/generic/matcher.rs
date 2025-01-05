
use super::{Matcher, MatchRule, State, Char, MatcherState};

impl<T: Clone, F: Matcher<T> + 'static, C: FnMut() -> F> MatchRule<T> for C {
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
            let mut group = $crate::generic::matcher::GroupMatcher::new();

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
            let mut sequence = $crate::generic::matcher::SequenceMatcher::new();

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

pub struct CharSequenceMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut() -> T,
    FA: FnMut(Char) -> bool,
{
    chars: Vec<char>,
    last: usize,
    resf: FG,
    allowed: FA,
    cooked: bool,
}

impl<T, FG, FA> CharSequenceMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut() -> T,
    FA: FnMut(Char) -> bool,
{
    pub fn new(value: &str, resf: FG, allowed: FA) -> Self {
        Self {
            chars: value.chars().collect(),
            last: 0,
            resf,
            allowed,
            cooked: false,
        }
    }
}

impl<T, FG, FA> Matcher<T> for CharSequenceMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut() -> T,
    FA: FnMut(Char) -> bool,
{
    fn check(self: &mut Self, c: Char) -> State<T> {
        if self.cooked {
            return State::Rejected
        }

        let res = match self.chars.len().cmp(&self.last) {
            std::cmp::Ordering::Greater => match c {
                Char::EOF => State::Rejected,
                Char::Char(c) => {
                    let target = self.chars.get(self.last).expect("Was checked");
                    self.last += 1;

                    if c == *target {
                        State::Matching
                    } else {
                        State::Rejected
                    }
                }
            },
            std::cmp::Ordering::Equal => {
                if !(self.allowed)(c) {
                    State::Matched((self.resf)())
                } else {
                    State::Rejected
                }
            },
            std::cmp::Ordering::Less => panic!("Index overflow"),
        };

        if let State::Rejected | State::Matched(_) = res {
            self.cooked = true;
        }

        res
    }

    fn reset(self: &mut Self) {
        self.last = 0;
        self.cooked = false;
    }
}

pub struct AllowedCharMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut(&str) -> T,
    FA: FnMut(Char) -> bool,
{
    result: String,
    resf: FG,
    allowed: FA,
    cooked: bool,
}

impl<T, FG, FA> AllowedCharMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut(&str) -> T,
    FA: FnMut(Char) -> bool,
{
    pub fn new(resf: FG, allowed: FA) -> Self {
        Self {
            result: String::new(),
            resf,
            allowed,
            cooked: false,
        }
    }
}

impl<T, FG, FA> Matcher<T> for AllowedCharMatcher<T, FG, FA>
where
    T: Clone,
    FG: FnMut(&str) -> T,
    FA: FnMut(Char) -> bool,
{
    fn check(self: &mut Self, c: Char) -> State<T> {
        if self.cooked {
            return State::Rejected
        }

        let res = if !(self.allowed)(c) {
            if "" == self.result {
                State::Rejected
            } else {
                State::Matched((self.resf)(&self.result))
            }
        } else if let Char::Char(c) = c {
            let mut buf: [u8; 4] = [0; 4];
            self.result += c.encode_utf8(&mut buf);
            State::Matching
        } else {
            State::Rejected
        };

        if let State::Rejected | State::Matched(_) = res {
            self.cooked = true;
        }

        res
    }

    fn reset(self: &mut Self) {
        self.result.clear();
        self.cooked = false;
    }
}

enum RadixState {
    None,
    Pending,
    Set(u8),
}

impl RadixState {
    fn get(self: &Self) -> u32 {
        match self {
            Self::None | Self::Pending => 10,
            Self::Set(r) => *r as u32,
        }
    }
}

pub struct IntegerMatcher<T, F>
where
    T: Clone,
    F: FnMut(i64) -> T
{
    radix: RadixState,
    cooked: bool,
    any: bool,
    number: i64,
    resf: F,
    sign: i64,
}

impl<T, F> IntegerMatcher<T, F>
where
    T: Clone,
    F: FnMut(i64) -> T
{
    pub fn new(resf: F) -> Self {
        Self {
            radix: RadixState::None,
            cooked: false,
            any: false,
            number: 0,
            resf,
            sign: 1,
        }
    }
}

impl<T, F> Matcher<T> for IntegerMatcher<T, F>
where
    T: Clone,
    F: FnMut(i64) -> T
{
    fn check(self: &mut Self, c: Char) -> State<T> {
        if self.cooked {
            return State::Rejected
        }

        let r = self.radix.get();

        let res = match c {
            Char::EOF => {
                if self.any {
                    State::Matched((self.resf)(self.number * self.sign))
                } else {
                    State::Rejected
                }
            },
            Char::Char(c) => {
                if let RadixState::Pending = self.radix {
                    let mut lc = c.to_lowercase();
                    match (lc.next(), lc.next()) {
                        (Some('b'), None) => {
                            self.radix = RadixState::Set(2);
                            State::Matching
                        },
                        (Some('o'), None) => {
                            self.radix = RadixState::Set(8);
                            State::Matching
                        },
                        (Some('x'), None) => {
                            self.radix = RadixState::Set(16);
                            State::Matching
                        },
                        _ => {
                            if let Some(n) = c.to_digit(r) {
                                self.radix = RadixState::None;
                                self.number = n as i64;
                                State::Matching
                            } else {
                                State::Matched((self.resf)(0))
                            }
                        },
                    }
                } else if !self.any {
                    if '-' == c {
                        if 0 > self.sign {
                            State::Rejected
                        } else {
                            self.sign = -1;
                            State::Matching
                        }
                    } else if '0' == c {
                        self.any = true;
                        self.radix = RadixState::Pending;
                        State::Matching
                    } else if let Some(n) = c.to_digit(r) {
                        self.any = true;
                        self.number = n as i64;
                        State::Matching
                    } else {
                        State::Rejected
                    }
                } else {
                    if let Some(n) = c.to_digit(r) {
                        self.number = self.number * (r as i64) + n as i64;
                        State::Matching
                    } else {
                        State::Matched((self.resf)(self.number * self.sign))
                    }
                }
            }
        };

        if let State::Rejected | State::Matched(_) = res {
            self.cooked = true;
        }

        res
    }

    fn reset(self: &mut Self) {
        self.radix = RadixState::None;
        self.cooked = false;
        self.any = false;
        self.number = 0;
        self.sign = 1;
    }
}

