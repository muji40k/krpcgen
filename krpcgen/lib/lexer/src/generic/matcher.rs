
use super::{Matcher, MatchRule, State, Char};

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

