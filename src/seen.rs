#[derive(Debug, Clone)]
pub enum Seen<T> {
    New(T),
    Old(T),
}

impl<T> Seen<T> {
    pub fn any(self) -> T {
        match self {
            Seen::New(x) => x,
            Seen::Old(x) => x,
        }
    }

    pub fn new(self) -> Option<T> {
        match self {
            Seen::New(x) => Some(x),
            Seen::Old(_) => None,
        }
    }

    pub fn old(self) -> Option<T> {
        match self {
            Seen::New(_) => None,
            Seen::Old(x) => Some(x),
        }
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Seen<U> {
        match self {
            Self::New(x) => Seen::<U>::New(f(x)),
            Self::Old(x) => Seen::<U>::Old(f(x)),
        }
    }
}
