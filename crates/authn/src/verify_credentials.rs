pub trait VerifyCredentials {
    type Credentials;
    type Identity;
    type Error;

    /// Returns `Ok(None)` for invalid credentials and `Err` for operational failures.
    fn verify(
        &self,
        creds: &Self::Credentials,
    ) -> impl Future<Output = Result<Option<Self::Identity>, Self::Error>> + Send;
}
