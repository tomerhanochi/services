use thiserror::Error;

#[derive(Debug, Error)]
pub enum Either<A, B> {
    #[error(transparent)]
    Left(A),
    #[error(transparent)]
    Right(B),
}
