use claim::assert_gt;
use validator::HasLen;
use gvserver::routes::pinpoints::get::{GetPinpointRequest, GetPinpointResponse};
use gvserver::routes::pinpoints::post::PostPinpointRequest;
use crate::helpers::{spawn_app};

#[tokio::test]
async fn get_all_pinpoints_allowed_with_custom_credentials() {
    let app = spawn_app().await;
    let jwt = app.create_jwt("TESTUSER").await;
    let request_body = GetPinpointRequest {
        latitude: Some(5.0),
        longitude: Some(5.0),
        proximity: Some(555.0),
        pinpoint_id: None,
        username: None
    };
    let response = app.get_pinpoints(jwt, String::from("TESTUSER"), request_body).await;
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
        proximity: None,
        pinpoint_id: None,
        username: None
    };
    let response = app.get_pinpoints(jwt, String::from("TESTUSER"), request_body).await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
pub async fn get_all_pinpoints_allowed_with_new_user_jwt() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        proximity: None,
        pinpoint_id: None,
        username: None
    };
    let response = app.get_pinpoints(jwt, username.clone(), request_body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
pub async fn post_pinpoint_allowed_with_new_user_jwt() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let request_body = PostPinpointRequest::new(
        5.0, 5.0, String::from(
            "From unit testing"), None, username.clone());

    let response = app.post_pinpoints(jwt, request_body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
pub async fn post_pinpoint_fails_with_invalid_jwt() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let request_body = PostPinpointRequest::new(
        5.0, 5.0, String::from(
            "From unit testing"), None, username.clone());
    let response = app.post_pinpoints(String::from("BadJWTHereLOL"), request_body).await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
pub async fn handle_images_locally() {
    let app = spawn_app().await;
    let input_path = format!("{}/icantdoitsquidward.jpg", app.get_test_input_dir_path());
    let output_path = format!("{}/istillcantdoitsquidward.jpg", app.get_test_output_dir_path());
    let attachment = app.load_img_bytes_at(&input_path).await;
    if attachment.is_some() {
        let b_count = attachment.clone().unwrap().len();
        println!("Loaded byte count: {:?}", b_count);
    }
    let img_bytes = attachment.expect("Failed to load image bytes.");
    let attempt = app.save_img_bytes_at(&output_path, &img_bytes, 50).await;
    assert!(attempt.is_ok());
}

#[tokio::test]
pub async fn post_pinpoint_with_attachment() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let input_path = format!("{}/icantdoitsquidward.jpg", app.get_test_input_dir_path());
    let attachment = app.load_img_bytes_at(&input_path).await;
    let img_bytes = attachment.expect("Failed to load image bytes.");
    let expensive = img_bytes.clone().len();
    println!("Loaded image byte length: {:?}", expensive);

    let request_body = PostPinpointRequest::new(
        5.0, 5.0, String::from(
            "From unit testing"), Some(img_bytes), username.clone());

    let response = app.post_pinpoints(jwt, request_body).await;
    assert_eq!(response.status(), 200);
}

#[tokio::test]
pub async fn post_pinpoint_with_attachment_full_validation() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let input_path = format!("{}/icantdoitsquidward.jpg", app.get_test_input_dir_path());
    let output_path = format!("{}/icanindeeddoitsquidward.jpg", app.get_test_output_dir_path());
    let attachment = app.load_img_bytes_at(&input_path).await;
    let img_bytes = attachment.expect("Failed to load image bytes.");
    let expensive = img_bytes.clone().len();
    println!("Loaded image byte length: {:?}", expensive);
    let request_body = PostPinpointRequest::new(
        12.34, 12.34, String::from(
            "From unit testing"), Some(img_bytes), username.clone());
    let response = app.post_pinpoints(jwt.clone(), request_body).await;
    assert_eq!(response.status(), 200);
    let get_req = GetPinpointRequest {
        latitude: Some(12.34),
        longitude: Some(12.34),
        proximity: Some(0.01),
        pinpoint_id: None,
        username: None,
    };
    let get_back = app.get_pinpoints(
        jwt, username.to_string(), get_req).await;
    assert_eq!(response.status(), 200);
    let json_return = get_back.json::<Vec<GetPinpointResponse>>().await
        .expect("Failed to get a JSON response back.");
    assert_gt!(json_return.length(), 0);
    if !json_return[0].attachment.is_empty() {
        println!("Attachment in GetPinpointResponse: length {}", json_return[0].attachment.len());
    }
    let save_attempt = app.save_img_bytes_at(
        &output_path, &json_return[0].attachment, 50).await;
    assert!(save_attempt.is_ok());
}

#[tokio::test]
pub async fn get_all_pinpoints_not_allowed_with_new_user_faulty_jwt() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
    let mut evil_jwt = jwt.clone();
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
        proximity: None,
        pinpoint_id: None,
        username: None
    };
    let response = app.get_pinpoints(jwt, username.clone(), request_body).await;
    assert_eq!(response.status(), 200);

    let request_body = GetPinpointRequest {
        latitude: None,
        longitude: None,
        proximity: None,
        pinpoint_id: None,
        username: None
    };
    let response = app.get_pinpoints(evil_jwt, username.to_string(), request_body).await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn post_get_pinpoint_allowed_with_generated_user() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let jwt = app.sign_up_test_user(username.clone()).await;
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
        proximity: None,
        pinpoint_id: None,
        username: Some(username.clone())
    };
    let response = app.get_pinpoints(jwt.clone(), username.to_string(), request_body).await;
    assert_eq!(response.status(), 200);
    let json_return = response.json::<Vec<GetPinpointResponse>>().await
        .expect("Failed to get a JSON response back.");
    //println!("Vector returned: {}", json_return.length());
    assert_gt!(json_return.length(), 0);
    // Assert
    assert_eq!(status.as_u16(), 200);
}
