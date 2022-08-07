use std::fmt;

pub enum Never {}

impl fmt::Debug for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl fmt::Display for Never {
    fn fmt(&self, _f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Ok(())
    }
}

impl std::error::Error for Never {}

impl serde::ser::Error for Never {
    fn custom<T: fmt::Display>(_msg: T) -> Self {
        unreachable!()
    }
}
