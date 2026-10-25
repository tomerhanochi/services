use authn_identity::prelude::*;

#[test]
fn principal_ids_parse_and_serialize_without_losing_invariants() {
    for value in ["", "bad\nuser", "bad\0user"] {
        assert!(UserId::new(value).is_err());
        assert!(MachineId::new(value).is_err());
        let json = serde_json::to_string(value).unwrap();
        assert!(serde_json::from_str::<UserId>(&json).is_err());
        assert!(serde_json::from_str::<MachineId>(&json).is_err());
    }
    let user = Identity::User(User {
        id: "local-user-123".parse().unwrap(),
    });
    let machine = Identity::Machine(Machine {
        id: "local-user-123".parse().unwrap(),
    });
    assert_ne!(user, machine);
    for identity in [user, machine] {
        let json = serde_json::to_string(&identity).unwrap();
        assert_eq!(serde_json::from_str::<Identity>(&json).unwrap(), identity);
    }
}
