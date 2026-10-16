use authn::prelude::{Either, VerifyCredentials};
use authn_identity::prelude::*;
use authn_oidc::prelude::*;
use std::{
    io,
    sync::Arc,
    time::{Duration, SystemTime},
};

fn claims(issuer: &str, subject: &str) -> OidcIdentity {
    OidcIdentity {
        issuer: issuer.into(),
        subject: subject.into(),
        email: Some("same@example.com".into()),
        email_verified: true,
    }
}

#[tokio::test]
async fn issuer_and_subject_map_to_local_users_independently_of_email() {
    let issuer = HttpsIssuerUrl::new("https://issuer.example").unwrap();
    let id = UserId::new("local-user").unwrap();
    let mut users = OidcUsers::default();
    users.insert(&issuer, "external-subject", id.clone());
    let sessions = Sessions::default();
    let expires = SystemTime::now() + Duration::from_secs(60);
    let verifier = OidcVerifier::new(sessions.clone(), Arc::new(users));
    let valid = sessions
        .issue(claims(issuer.as_str(), "external-subject"), expires)
        .unwrap();
    assert_eq!(
        verifier.verify(&valid).await.unwrap(),
        Some(Identity::User(User { id }))
    );
    for (issuer, subject) in [
        ("https://other.example", "external-subject"),
        (issuer.as_str(), "unknown"),
    ] {
        let id = sessions.issue(claims(issuer, subject), expires).unwrap();
        assert!(verifier.verify(&id).await.unwrap().is_none());
    }
    sessions.revoke(&valid).unwrap();
    assert!(verifier.verify(&valid).await.unwrap().is_none());
}

struct Unavailable;
impl OidcIdentityStore for Unavailable {
    type Error = io::Error;
    async fn lookup(&self, _: &OidcIdentity) -> io::Result<Option<Identity>> {
        Err(io::Error::other("users unavailable"))
    }
}
#[tokio::test]
async fn identity_store_failures_are_preserved() {
    let sessions = Sessions::default();
    let id = sessions
        .issue(
            claims("https://issuer.example", "subject"),
            SystemTime::now() + Duration::from_secs(60),
        )
        .unwrap();
    let verifier = OidcVerifier::new(sessions, Unavailable);
    let Either::Right(error) = verifier.verify(&id).await.unwrap_err() else {
        panic!("expected mapping failure")
    };
    assert_eq!(error.to_string(), "users unavailable");
}
