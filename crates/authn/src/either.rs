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

impl<A: Error, B: Error> Error for Either<A, B> {}
