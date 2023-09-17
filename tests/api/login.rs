use gvserver::authentication::AuthParameters;
use gvserver::routes::login::post::LoginData;
use gvserver::routes::signup::post::UserSignUp;
use crate::{spawn_app};

#[tokio::test()]
pub async fn login_allows_genuine_user() {
    let app = spawn_app().await;
    let username = String::from("MentallyDeranged");
    let sign_up_data = UserSignUp {
        email: String::from("mentallyderanged@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;
    assert_eq!(response.status(), 200);

    println!("Checkpoint 1");

    let login_data = LoginData {
        username,
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
        &json_return).expect("JWT Parse");

    let msg: String = auth_permissions.mode.to_string();
    println!("AuthPermissions found: {}", msg);

    assert_eq!(code, 200);
}

#[tokio::test()]
pub async fn login_doesnt_allow_bad_pw() {
    let app = spawn_app().await;
    let username = String::from("HumanWithIQOf40");
    let sign_up_data = UserSignUp {
        email: String::from("humanorsomething@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;
    assert_eq!(response.status(), 200);

    let login_data = LoginData {
        username,
        pw: String::from("$uper$ecurePa$$word!OopsExtra!")
    };

    serde_json::to_string(&login_data)
        .expect("Failed to serialize struct.");

    let response = app.post_login(
        login_data.username, login_data.pw).await;

    assert_ne!(response.status(), 200);
}