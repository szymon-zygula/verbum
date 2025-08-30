/// Descriptive enum to use instead of `bool` as return type for functions which either do something or not.
pub enum Did {
    Something,
    Nothing,
}

impl Did {
    fn did_something(&self) -> bool {
        match self {
            Did::Something => true,
            Did::Nothing => false,
        }
    }

    fn did_nothing(&self) -> bool {
        !self.did_something()
    }
}
