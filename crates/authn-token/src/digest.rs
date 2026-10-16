/// SHA-256 of a high-entropy API token, suitable for an indexed store lookup.
/// Generate tokens using cryptographically random secrets, not passwords.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenDigest(pub [u8; 32]);
