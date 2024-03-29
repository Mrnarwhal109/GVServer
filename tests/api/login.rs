use gvserver::authentication::AuthParameters;
use gvserver::domain::user_sign_up::UserSignUp;
use gvserver::routes::login::post::LoginData;
use gvserver::routes::users::post::PostUserRequest;
use crate::{spawn_app};

#[tokio::test()]
pub async fn login_allows_genuine_user() {
    let app = spawn_app().await;
    let username = String::from("MentallyDeranged");
    let pw = String::from("$uper$ecurePa$$word!");
    let sign_up_data = PostUserRequest {
        email: String::from("mentallyderanged@gmail.com"),
        contents_description: None,
        contents_attachment: None
    };
    let response = app.post_users(sign_up_data, username.clone(), pw).await;
    assert_eq!(response.status(), 200);
    let login_data = LoginData {
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };
    serde_json::to_string(&login_data)
        .expect("Failed to serialize struct.");
    let response = app.post_login(
        login_data.username, login_data.pw).await;
    let code = response.status().as_u16();
    let json_return =
        response.json::<AuthParameters>().await.expect("Failure to get JWT");
    let auth_permissions = app.auth_service.validate_request(
        &json_return).expect("JWT parsing problem");
    assert_eq!(code, 200);
}

#[tokio::test()]
pub async fn login_doesnt_allow_bad_pw() {
    let app = spawn_app().await;
    let username = String::from("MentallyDeranged");
    let pw = String::from("$uper$ecurePa$$word!");
    let sign_up_data = PostUserRequest {
        email: String::from("mentallyderanged@gmail.com"),
        contents_description: None,
        contents_attachment: None
    };
    let response = app.post_users(sign_up_data, username.clone(), pw).await;
    assert_eq!(response.status(), 200);
    let login_data = LoginData {
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!OopsExtra!")
    };
    serde_json::to_string(&login_data)
        .expect("Failed to serialize struct.");
    let response = app.post_login(
        login_data.username, login_data.pw).await;
    assert_ne!(response.status(), 200);
}