//! Real PostgreSQL, temporary mounts and the complete router built from configuration.
use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use file_server::prelude::*;
use http_body_util::BodyExt;
use std::fs;
use tower::ServiceExt;

fn hash(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher, password_hash::SaltString};
    let salt = SaltString::encode_b64(b"test-salt-16bytes").unwrap();
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}
fn request(method: &str, uri: &str, auth: Option<&str>, body: &str) -> Request<Body> {
    let mut request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded");
    if let Some(auth) = auth {
        request = request.header(header::AUTHORIZATION, auth);
    }
    request.body(Body::from(body.to_owned())).unwrap()
}
async fn text(response: axum::response::Response) -> String {
    String::from_utf8(
        response
            .into_body()
            .collect()
            .await
            .unwrap()
            .to_bytes()
            .to_vec(),
    )
    .unwrap()
}

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to a disposable PostgreSQL database"]
async fn configured_server_browses_downloads_renames_moves_and_enforces_access() {
    let directory = tempfile::tempdir().unwrap();
    fs::create_dir(directory.path().join("users")).unwrap();
    fs::create_dir(directory.path().join("documents")).unwrap();
    fs::create_dir(directory.path().join("documents/folder")).unwrap();
    fs::write(directory.path().join("users/alice"), hash("password")).unwrap();
    fs::write(directory.path().join("users/bob"), hash("password")).unwrap();
    fs::write(directory.path().join("documents/a.txt"), "hello world").unwrap();
    fs::write(directory.path().join("documents/existing.txt"), "keep me").unwrap();
    fs::write(directory.path().join("documents/<unsafe>.txt"), "escaped").unwrap();
    for name in ["c.txt", "d.txt", "e.txt", "folder/d.txt"] {
        fs::write(directory.path().join("documents").join(name), name).unwrap();
    }
    let url = std::env::var("TEST_DATABASE_URL").unwrap();
    let contents = include_str!("../file_server.example.toml").replace(
        "postgres://file_server:development-only@localhost:15432/file_server",
        &url,
    );
    let config_path = directory.path().join("server.toml");
    fs::write(
        &config_path,
        format!("{contents}\n[auth.token]\ndatabase_url = {url:?}\n"),
    )
    .unwrap();
    let config = Config::load(&config_path).unwrap();
    let app = Server::new(config).await.unwrap().router();
    // base64("alice:password") and base64("bob:password")
    let alice = "Basic YWxpY2U6cGFzc3dvcmQ=";
    let bob = "Basic Ym9iOnBhc3N3b3Jk";

    let health = app
        .clone()
        .oneshot(request("GET", "/healthz", None, ""))
        .await
        .unwrap();
    assert_eq!(health.status(), StatusCode::OK);
    let anonymous = app
        .clone()
        .oneshot(request("GET", "/m/documents/", None, ""))
        .await
        .unwrap();
    assert_eq!(anonymous.status(), StatusCode::UNAUTHORIZED);
    let login = app
        .clone()
        .oneshot(request("GET", "/login", None, ""))
        .await
        .unwrap();
    assert_eq!(login.status(), StatusCode::UNAUTHORIZED);
    assert!(login.headers().contains_key(header::WWW_AUTHENTICATE));
    let home = app
        .clone()
        .oneshot(request("GET", "/", Some(alice), ""))
        .await
        .unwrap();
    assert_eq!(home.status(), StatusCode::OK);
    assert!(text(home).await.contains("documents"));
    let listing = app
        .clone()
        .oneshot(request("GET", "/m/documents/", Some(bob), ""))
        .await
        .unwrap();
    assert_eq!(listing.status(), StatusCode::OK);
    let body = text(listing).await;
    assert!(body.contains("a.txt"));
    assert!(body.contains("&#60;unsafe&#62;.txt"));
    assert!(!body.contains("<unsafe>"));
    let file = app
        .clone()
        .oneshot(request("GET", "/m/documents/a.txt", Some(bob), ""))
        .await
        .unwrap();
    assert_eq!(file.status(), StatusCode::OK);
    assert_eq!(text(file).await, "hello world");
    let no_actions = app
        .clone()
        .oneshot(request("GET", "/actions/documents/a.txt", Some(bob), ""))
        .await
        .unwrap();
    assert_eq!(no_actions.status(), StatusCode::FORBIDDEN);
    let actions = app
        .clone()
        .oneshot(request("GET", "/actions/documents/a.txt", Some(alice), ""))
        .await
        .unwrap();
    assert_eq!(actions.status(), StatusCode::OK);
    let body = text(actions).await;
    assert!(body.contains(r#"href="/rename/documents/a.txt""#));
    assert!(body.contains(r#"href="/move/documents/a.txt""#));
    let forbidden = app
        .clone()
        .oneshot(request(
            "POST",
            "/rename/documents/a.txt",
            Some(bob),
            "name=b.txt",
        ))
        .await
        .unwrap();
    assert_eq!(forbidden.status(), StatusCode::FORBIDDEN);
    let clash = app
        .clone()
        .oneshot(request(
            "POST",
            "/rename/documents/a.txt",
            Some(alice),
            "name=existing.txt",
        ))
        .await
        .unwrap();
    assert_eq!(clash.status(), StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        fs::read_to_string(directory.path().join("documents/existing.txt")).unwrap(),
        "keep me"
    );
    let renamed = app
        .clone()
        .oneshot(request(
            "POST",
            "/rename/documents/a.txt",
            Some(alice),
            "name=b.txt",
        ))
        .await
        .unwrap();
    assert_eq!(renamed.status(), StatusCode::SEE_OTHER);
    let moved = app
        .clone()
        .oneshot(request(
            "POST",
            "/move/documents/b.txt",
            Some(alice),
            "to=folder",
        ))
        .await
        .unwrap();
    assert_eq!(moved.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        fs::read_to_string(directory.path().join("documents/folder/b.txt")).unwrap(),
        "hello world"
    );
    let selecting = app
        .clone()
        .oneshot(request("GET", "/m/documents/?select", Some(alice), ""))
        .await
        .unwrap();
    assert_eq!(selecting.status(), StatusCode::OK);
    let body = text(selecting).await;
    assert!(body.contains(r#"name="item" value="c.txt" form="selection""#));
    assert!(body.contains(r#"action="/move-many/documents""#));
    let viewer_selecting = app
        .clone()
        .oneshot(request("GET", "/m/documents/?select", Some(bob), ""))
        .await
        .unwrap();
    assert!(!text(viewer_selecting).await.contains("md-checkbox"));
    let viewer_moves = app
        .clone()
        .oneshot(request(
            "POST",
            "/move-many/documents?item=c.txt",
            Some(bob),
            "to=folder",
        ))
        .await
        .unwrap();
    assert_eq!(viewer_moves.status(), StatusCode::FORBIDDEN);
    let moved_many = app
        .clone()
        .oneshot(request(
            "POST",
            "/move-many/documents?item=c.txt&item=%3Cunsafe%3E.txt",
            Some(alice),
            "to=folder",
        ))
        .await
        .unwrap();
    assert_eq!(moved_many.status(), StatusCode::SEE_OTHER);
    assert_eq!(
        moved_many.headers()[header::LOCATION],
        "/m/documents/folder/?done=moved"
    );
    for name in ["c.txt", "<unsafe>.txt"] {
        assert!(
            directory
                .path()
                .join("documents/folder")
                .join(name)
                .exists()
        );
    }
    let partly_moved = app
        .clone()
        .oneshot(request(
            "POST",
            "/move-many/documents?item=d.txt&item=e.txt",
            Some(alice),
            "to=folder",
        ))
        .await
        .unwrap();
    assert_eq!(partly_moved.status(), StatusCode::CONFLICT);
    let body = text(partly_moved).await;
    assert!(body.contains("Moved 1 of 2. The other 1:"), "{body}");
    // The picker comes back with only what didn't move.
    assert!(
        body.contains(r#"action="/move-many/documents?item=d.txt""#),
        "{body}"
    );
    assert!(directory.path().join("documents/folder/e.txt").exists());
    assert_eq!(
        fs::read_to_string(directory.path().join("documents/d.txt")).unwrap(),
        "d.txt"
    );
    let too_many = (0..201)
        .map(|i| format!("item={i}"))
        .collect::<Vec<_>>()
        .join("&");
    let too_many = app
        .clone()
        .oneshot(request(
            "GET",
            &format!("/move-many/documents?{too_many}"),
            Some(alice),
            "",
        ))
        .await
        .unwrap();
    assert_eq!(too_many.status(), StatusCode::BAD_REQUEST);
    let traversal = app
        .clone()
        .oneshot(request(
            "GET",
            "/m/documents/%2E%2E/secret",
            Some(alice),
            "",
        ))
        .await
        .unwrap();
    assert_eq!(traversal.status(), StatusCode::NOT_FOUND);
    let cross_site = Request::builder()
        .method("POST")
        .uri("/rename/documents/existing.txt")
        .header(header::AUTHORIZATION, alice)
        .header(header::ORIGIN, "https://attacker.example")
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from("name=oops"))
        .unwrap();
    assert_eq!(
        app.clone().oneshot(cross_site).await.unwrap().status(),
        StatusCode::FORBIDDEN
    );
    let css = app
        .clone()
        .oneshot(request("GET", "/assets/app.css", None, ""))
        .await
        .unwrap();
    assert_eq!(css.status(), StatusCode::OK);
    assert_eq!(
        css.headers()[header::CONTENT_TYPE],
        "text/css; charset=utf-8"
    );
    assert!(text(css).await.contains("--md-sys-color-primary"));
    // A database-backed machine token goes through the same file authorization path.
    let token = authn_token::prelude::Token::generate().unwrap();
    let pool = sqlx::PgPool::connect(&url).await.unwrap();
    let identity = authn_identity::prelude::Identity::Machine(authn_identity::prelude::Machine {
        id: authn_identity::prelude::MachineId::new("backup").unwrap(),
    });
    sqlx::query("INSERT INTO authn_api_tokens (digest, identity) VALUES ($1, $2)")
        .bind(token.digest().0.as_slice())
        .bind(sqlx::types::Json(identity))
        .execute(&pool)
        .await
        .unwrap();
    let bearer = format!(
        "Bearer {}",
        std::str::from_utf8(token.expose_secret()).unwrap()
    );
    assert_eq!(
        app.clone()
            .oneshot(request("GET", "/m/documents/", Some(&bearer), ""))
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.clone()
            .oneshot(request(
                "POST",
                "/rename/documents/existing.txt",
                Some(&bearer),
                "name=oops"
            ))
            .await
            .unwrap()
            .status(),
        StatusCode::FORBIDDEN
    );
    sqlx::query("UPDATE authn_api_tokens SET revoked = true WHERE digest = $1")
        .bind(token.digest().0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        app.clone()
            .oneshot(request("GET", "/m/documents/", Some(&bearer), ""))
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
    sqlx::query("DELETE FROM authn_api_tokens WHERE digest = $1")
        .bind(token.digest().0.as_slice())
        .execute(&pool)
        .await
        .unwrap();
    let invalid = app
        .oneshot(request(
            "GET",
            "/m/documents/",
            Some("Basic YWxpY2U6d3Jvbmc="),
            "",
        ))
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::UNAUTHORIZED);
}
