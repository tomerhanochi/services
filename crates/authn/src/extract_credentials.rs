pub trait ExtractCredentials {
    type Context;
    type Credentials;
    type Error;

    fn extract(&self, ctx: &Self::Context) -> Result<Option<Self::Credentials>, Self::Error>;
}
