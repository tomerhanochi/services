use authn_identity::prelude::*;
use file_server_core::prelude::*;
use file_server_postgres::prelude::*;

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn roles_are_atomic_and_user_and_machine_ids_are_separate() {
    let config = DatabaseConfig {
        url: std::env::var("TEST_DATABASE_URL").unwrap(),
        max_connections: 2.try_into().unwrap(),
    };
    let store = PostgresAccess::connect(&config).await.unwrap();
    let mount = MountName::new("test-mount").unwrap();
    let user = Identity::User(User {
        id: UserId::new("same-id").unwrap(),
    });
    let machine = Identity::Machine(Machine {
        id: MachineId::new("same-id").unwrap(),
    });
    store
        .sync_access(
            &[
                ("read".into(), mount.clone(), Grant::Viewer),
                ("write".into(), mount.clone(), Grant::Editor),
            ],
            &[
                (user.clone(), vec!["read".into()]),
                (machine.clone(), vec!["write".into()]),
            ],
        )
        .await
        .unwrap();
    assert_eq!(
        store.capabilities(&user, &mount).await.unwrap(),
        Grant::Viewer.capabilities()
    );
    assert_eq!(
        store.capabilities(&machine, &mount).await.unwrap(),
        Grant::Editor.capabilities()
    );
    let duplicate = [
        ("read".into(), mount.clone(), Grant::Viewer),
        ("read".into(), mount.clone(), Grant::Editor),
    ];
    assert!(store.sync_access(&duplicate, &[]).await.is_err());
    assert!(store.capabilities(&machine, &mount).await.unwrap().rename);
    store.sync_access(&[], &[]).await.unwrap();
    assert!(!store.capabilities(&user, &mount).await.unwrap().list);
}
