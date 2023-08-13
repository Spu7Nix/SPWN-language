use std::cell::RefCell;
use std::rc::Rc;

use ahash::RandomState;
use itertools::Itertools;
use lasso::{Rodeo, Spur};

use super::{ImmutStr, ImmutStr32, String32};

pub struct Interner(Rc<RefCell<Rodeo<Spur, RandomState>>>);

impl Default for Interner {
    fn default() -> Self {
        Self(Rc::new(RefCell::new(
            Rodeo::with_hasher(RandomState::new()),
        )))
    }
}

impl Interner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_intern<T>(&self, val: T) -> Spur
    where
        T: AsRef<str>,
    {
        self.0.borrow_mut().get_or_intern(val)
    }

    pub fn resolve(&self, s: &Spur) -> ImmutStr {
        self.0.borrow().resolve(s).into()
    }

    pub fn resolve_32(&self, s: &Spur) -> ImmutStr32 {
        String32::from_chars(self.0.borrow().resolve(s).chars().collect_vec()).into()
    }
}

impl Clone for Interner {
    fn clone(&self) -> Self {
        Self(Rc::clone(&self.0))
    }
}
