pub trait VerifyCredentials {
    type Credentials;
    type Identity;
    type Error;

    fn verify(
        &self,
        creds: &Self::Credentials,
    ) -> impl Future<Output = Result<Option<Self::Identity>, Self::Error>>;
}
