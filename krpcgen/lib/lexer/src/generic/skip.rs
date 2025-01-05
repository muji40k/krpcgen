
use super::{Skip, SkipRule};

impl<T: Skip + 'static, C: FnMut() -> T> SkipRule for C {
    fn get(self: &mut Self) -> Box<dyn Skip> {
        Box::new(self())
    }
}

impl<T: FnMut(char) -> bool> Skip for T {
    fn is_skipping(self: &mut Self, c: char) -> bool {
        self(c)
    }
}

