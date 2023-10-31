use claim::assert_gt;
use crate::helpers::{spawn_app};
use gvserver::routes::users::get::{GetUsersRequest, UserResponse};
use gvserver::routes::users::post::PostUserRequest;

#[tokio::test()]
async fn sign_up_persists_users() {
    let app = spawn_app().await;
    let username = String::from("MentallyDeranged");
    let pw = String::from("$uper$ecurePa$$word!");
    let sign_up_data = PostUserRequest {
        email: String::from("mentallyderanged@gmail.com"),
        contents_description: None,
        contents_attachment: None
    };
    let response = app.post_signup(sign_up_data, username.clone(), pw).await;
    assert_eq!(response.status(), 200);
    let get_user_attempt = app.select_one_user(username.clone()).await;
    match get_user_attempt {
        Ok(row) => {
            println!("DbUser returned with username {}, email {}, phash {}, salt {}",
                     &row.username, &row.email, &row.phash, &row.salt);
            assert_eq!(row.username, username)
        }
        Err(e) => {
            println!("Query error: {}", e.to_string());
            panic!();
        }
    }
}

#[tokio::test()]
async fn sign_up_rejects_a_duplicate_user() {
    let app = spawn_app().await;
    let username = String::from("MentallyDeranged");
    let pw = String::from("$uper$ecurePa$$word!");
    let sign_up_data = PostUserRequest {
        email: String::from("mentallyderanged@gmail.com"),
        contents_description: None,
        contents_attachment: None
    };
    let other_sign_up_data = PostUserRequest {
        email: String::from("mentallyderanged@gmail.com"),
        contents_description: None,
        contents_attachment: None
    };
    let response = app.post_signup(
        sign_up_data, username.clone(), pw.clone()).await;
    assert_eq!(response.status(), 200);
    let response = app.post_signup(
        other_sign_up_data, username.clone(), pw).await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
pub async fn get_users_username_only() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let jwt = app.sign_up_test_user(username.to_string()).await;
    let request_body = GetUsersRequest {
        email: None,
        username: Some(username.to_string()),
        user_id: None,
    };
    let response = app.get_users(Some(jwt), request_body).await;
    let code = (&response.status()).clone();
    let json = response.json::<UserResponse>().await.unwrap();
    println!("Response json {:?}", json);
    assert!(json.username.is_some());
    assert_eq!(code, 200);
}

#[tokio::test]
pub async fn get_users_username_only_no_auth() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    app.sign_up_test_user(username.to_string()).await;
    let request_body = GetUsersRequest {
        email: None,
        username: Some(username.to_string()),
        user_id: None,
    };
    let response = app.get_users(None, request_body).await;
    let code = (&response.status()).clone();
    let json = response.json::<UserResponse>().await.unwrap();
    println!("Response json {:?}", json);
    assert!(json.username.is_some());
    assert_eq!(code, 200);
}

#[tokio::test]
pub async fn get_users_nonexistent_but_ok() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let jwt = app.sign_up_test_user(username.to_string()).await;
    let request_body = GetUsersRequest {
        email: None,
        username: Some(String::from("IProbablyDoNotExist")),
        user_id: None,
    };
    let response = app.get_users(Some(jwt), request_body).await;
    let code = (&response.status()).clone();
    assert_eq!(response.content_length().unwrap(), 0);
    assert_eq!(code, 200);
}

#[tokio::test]
pub async fn user_attachment_full_validation() {
    let app = spawn_app().await;
    let username = String::from("TestGeneratedUser");
    let description = String::from("This description was generated by automatic testing!");
    let input_path = format!("{}/icantdoitsquidward.jpg",
                             app.get_test_input_dir_path());
    let output_path = format!("{}/icandoitfortheusersquidward.jpg",
                              app.get_test_output_dir_path());
    let attachment = app.load_img_bytes_at(&input_path).await;
    assert!(attachment.is_some());
    let img_bytes = attachment.expect("Failed to load image bytes.");
    let expensive = img_bytes.clone().len();
    println!("Loaded image byte length: {:?}", expensive);
    let jwt = app.sign_up_test_user_full(username.clone(),
                                         Some(description.clone()),
                                         Some(img_bytes)).await;
    let user_request = GetUsersRequest {
        email: None,
        username: Some(username.clone()),
        user_id: None,
    };
    let get_back = app.get_users(
        Some(jwt), user_request).await;
    assert_eq!(get_back.status(), 200);
    let mut json_return = get_back.json::<UserResponse>().await
        .expect("Failed to get a JSON response back.");
    //assert_gt!((&json_return).len(), 0);
    let response_body = &json_return.contents_attachment.as_mut().unwrap();

    let save_attempt = app.save_img_bytes_at(
        &output_path, &response_body, 50).await;
    assert!(save_attempt.is_ok());
}