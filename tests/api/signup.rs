use gvserver::routes::post::SignUpData;
use crate::helpers::{spawn_app};

#[tokio::test]
async fn sign_up_persists_a_user() {
    let app = spawn_app().await;
    let sign_up_data = SignUpData {
        email: String::from("somedude@gmail.com"),
        username: String::from("SomeDudeHere"),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data).await;

    // Assert
    assert_eq!(response.status(), 200);
}