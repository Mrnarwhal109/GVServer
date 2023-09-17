use chrono::Utc;
use uuid::Uuid;
use validator::HasLen;
use gvserver::authentication::AuthParameters;
use gvserver::domain::{Pinpoint, PostPinpointRequest};
use gvserver::domain::pinpoint::{GetPinpointRequest, GetPinpointResponse};
use gvserver::routes::login::post::LoginData;
use gvserver::routes::signup::post::UserSignUp;
use crate::helpers::{spawn_app};

#[tokio::test]
async fn get_all_pinpoints_allowed_with_custom_credentials() {
    let app = spawn_app().await;

    let jwt = app.create_jwt("TESTUSER").await;

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: None
    };

    let response = app.get_pinpoints(jwt, request_body).await;

    let status = &response.status();

    let json_return = response.json::<Vec<GetPinpointResponse>>().await
        .expect("Failed to get a JSON response back.");

    println!("Vector returned: {}", json_return.length());


    // Assert
    assert_eq!(status.as_u16(), 200);
}

#[tokio::test]
async fn get_all_pinpoints_fails_with_invalid_jwts() {
    let app = spawn_app().await;
    let jwt = String::from("yyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWI\
    iOiJURVNUVVNFUiIsImV4cCI6MTY5NDQ3MTY1OH0.CvMGrTd1IwWGTowzDrPdjnFZC5pF9a1oLBBlthOgIx8");
    let jwt = jwt.replace(' ', "").trim().to_string();
    println!("Formatted jwt: {}", &jwt);

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: None
    };

    let response = app.get_pinpoints(jwt, request_body).await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
pub async fn get_all_pinpoints_allowed_with_new_user_jwt() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let sign_up_data = UserSignUp {
        email: String::from("mentallyabsurd@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;
    assert_eq!(response.status(), 200);

    let login_data = LoginData {
        username,
        pw: String::from("$uper$ecurePa$$word!")
    };

   serde_json::to_string(&login_data)
        .expect("Failed to serialize struct.");

    let response = app.post_login(
        login_data.username, login_data.pw).await;

    let code = response.status().as_u16();

    let json_return = response.json::<AuthParameters>().await
        .expect("Failed to get a JSON response back.");

    println!("JWT response received: {}", json_return.jwt);

    assert_eq!(code, 200);

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: None
    };

    let response = app.get_pinpoints(json_return.jwt, request_body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
pub async fn get_all_pinpoints_not_allowed_with_new_user_faulty_jwt() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let sign_up_data = UserSignUp {
        email: String::from("mentallyabsurd@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;
    assert_eq!(response.status(), 200);

    let login_data = LoginData {
        username,
        pw: String::from("$uper$ecurePa$$word!")
    };

    serde_json::to_string(&login_data)
        .expect("Failed to serialize struct.");

    let response = app.post_login(
        login_data.username, login_data.pw).await;

    let code = response.status().as_u16();

    let json_return = response.json::<AuthParameters>().await
        .expect("Failed to get a JSON response back.");

    println!("JWT response received: {}", json_return.jwt);

    assert_eq!(code, 200);

    let mut evil_jwt = json_return.jwt.clone();
    evil_jwt.pop();
    evil_jwt.pop();
    evil_jwt.pop();
    evil_jwt.pop();
    evil_jwt.push('E');
    evil_jwt.push('V');
    evil_jwt.push('I');
    evil_jwt.push('L');

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: None
    };

    let response = app.get_pinpoints(json_return.jwt, request_body).await;
    assert_eq!(response.status(), 200);

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: None
    };

    let response = app.get_pinpoints(evil_jwt, request_body).await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn post_get_pinpoint_allowed_with_generated_user() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let sign_up_data = UserSignUp {
        email: String::from("mentallyabsurd@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;
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

    let json_return = response.json::<AuthParameters>().await
        .expect("Failed to get a JSON response back.");

    let jwt = json_return.jwt.clone();

    assert_eq!(code, 200);

    let pinpoint_request_body = PostPinpointRequest::new(
        123.0,
        123.0,
        String::from("Description: This pinpoint was added from unit testing."),
        None,
        username.clone()
    );

    let response = app.post_pinpoints(jwt.clone(), pinpoint_request_body).await;

    let status = &response.status();
    // Assert
    assert_eq!(status.as_u16(), 200);

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        radius: None,
        pinpoint_id: None,
        username: Some(username)
    };

    let response = app.get_pinpoints(jwt.clone(), request_body).await;
    assert_eq!(response.status(), 200);

    let json_return = response.json::<Vec<GetPinpointResponse>>().await
        .expect("Failed to get a JSON response back.");
    println!("Vector returned: {}", json_return.length());

    // Assert
    assert_eq!(status.as_u16(), 200);
}
