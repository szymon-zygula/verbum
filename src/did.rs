use std::ops::{BitOr, BitOrAssign};

/// Descriptive enum to use instead of `bool` as return type for functions which either do something or not.
pub enum Did {
    Something,
    Nothing,
}

impl Did {
    pub fn did_something(&self) -> bool {
        match self {
            Did::Something => true,
            Did::Nothing => false,
        }
    }

    pub fn did_nothing(&self) -> bool {
        !self.did_something()
    }
}

impl BitOr for Did {
    type Output = Self;

    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign for Did {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Did::Nothing, Did::Nothing) => Did::Nothing,
            _ => Did::Something,
        };
    }
}
