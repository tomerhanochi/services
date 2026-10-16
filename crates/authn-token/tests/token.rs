use authn_token::prelude::*;

#[test]
fn token68_is_parsed_before_a_token_can_be_used() {
    for value in ["", "=", "bad token", "a=b", "token\r\n", "é"] {
        assert!(Token::new(value).is_err(), "{value:?}");
    }
    for value in ["abc", "abc==", "AZaz09-._~+/"] {
        let token = Token::new(value).unwrap();
        assert_eq!(token.expose_secret(), value.as_bytes());
        assert!(!format!("{token:?}").contains(value));
    }
}

#[test]
fn generated_tokens_round_trip_through_the_parser() {
    let first = Token::generate().unwrap();
    let second = Token::generate().unwrap();
    assert_eq!(first.expose_secret().len(), 43);
    assert_ne!(first.digest(), second.digest());
    assert_eq!(
        Token::new(first.expose_secret()).unwrap().digest(),
        first.digest()
    );
}
