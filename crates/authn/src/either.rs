use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A: Display, B: Display> Display for Either<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Left(a) => a.fmt(f),
            Self::Right(b) => b.fmt(f),
        }
    }
}

impl<A: Error + 'static, B: Error + 'static> Error for Either<A, B> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Left(error) => Some(error),
            Self::Right(error) => Some(error),
        }
    }
}
