/// The result of parsing credentials at an untrusted input boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Extracted<T> {
    Absent,
    Malformed,
    Present(T),
}

impl<T> Extracted<T> {
    /// Parses a present value, preserving absence and malformed input.
    pub fn parse<U, E>(self, parse: impl FnOnce(T) -> Result<U, E>) -> Extracted<U> {
        match self {
            Self::Absent => Extracted::Absent,
            Self::Malformed => Extracted::Malformed,
            Self::Present(value) => match parse(value) {
                Ok(value) => Extracted::Present(value),
                Err(_) => Extracted::Malformed,
            },
        }
    }
}

pub trait ExtractCredentials {
    type Context;
    type Credentials;
    type Error;

    /// Malformed input is a rejection; `Err` is reserved for operational failures.
    fn extract(&self, ctx: &Self::Context) -> Result<Extracted<Self::Credentials>, Self::Error>;
}
