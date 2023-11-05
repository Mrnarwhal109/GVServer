use crate::helpers::{spawn_app};
use gvserver::routes::users::get::{GetUsersRequest, UserResponse};
use gvserver::routes::users::post::PostUserRequest;
use gvserver::routes::users::put::put_user_request::PutUserRequest;

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
    let response = app.post_users(sign_up_data, username.clone(), pw).await;
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
    let response = app.post_users(
        sign_up_data, username.clone(), pw.clone()).await;
    assert_eq!(response.status(), 200);
    let response = app.post_users(
        other_sign_up_data, username.clone(), pw).await;
    assert_eq!(response.status(), 400);
}

#[tokio::test]
pub async fn get_users_username_only() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let jwt = app.sign_up_test_user(username.as_str(),
                                    "mentallyabsurd@something.org", None).await;
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
    let username = String::from("MentallyAbsurd");
    let jwt = app.sign_up_test_user(username.as_str(),
                                    "mentallyabsurd@something.org", None).await;
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
    let username = String::from("MentallyAbsurd");
    let jwt = app.sign_up_test_user(username.as_str(),
                                    "mentallyabsurd@something.org", None).await;
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
pub async fn get_users_full_permissions() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let jwt = app.sign_up_test_user(username.as_str(),
                                    "mentallyabsurd@something.org", None).await;
    let request_body = GetUsersRequest {
        email: None,
        username: Some(String::from(username)),
        user_id: None,
    };
    let response = app.get_users(Some(jwt), request_body).await;
    let code = (&response.status()).clone();
    let json = response.json::<UserResponse>().await.unwrap();
    assert!(json.unique_id.is_some() && json.email.is_some() && json.username.is_some());
    println!("Response json {:?}", json);
    assert_eq!(code, 200);
}

#[tokio::test]
pub async fn get_users_wrong_permissions() {
    let app = spawn_app().await;
    let username = String::from("MentallyAbsurd");
    let jwt = app.sign_up_test_user(username.as_str(),
                                    "mentallyabsurd@something.org", None).await;
    let other_jwt = app.sign_up_test_user(
        "SomeOtherDude", "someguy@nothing.org", None).await;
    let request_body = GetUsersRequest {
        email: None,
        username: Some(String::from(username)),
        user_id: None,
    };
    // The wrong JWT is given here
    let response = app.get_users(Some(other_jwt), request_body).await;
    let code = (&response.status()).clone();
    let json = response.json::<UserResponse>().await.unwrap();
    assert!(json.unique_id.is_none());
    println!("Response json {:?}", json);
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
    let jwt = app.sign_up_test_user_full(username.as_str(),
                                         "someemailagain@asdf.com",
                                         Some("MyPassword10293120!"),
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

#[tokio::test]
pub async fn update_user_email_only() {
    // /users PUT is being tested here manually.
    // In other tests, the helper functions are used instead.
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let email = "testhere@something.net";
    let passwd = "MyBadPassword";
    let replacement_email = "othertesthere@something.net";
    let (jwt, user_obj) = app.sign_up_get_full_user(
        username, email, Some(passwd), None, None).await;
    assert!(user_obj.unique_id.as_ref().is_some());
    assert_eq!(user_obj.email, Some(email.to_string()));
    assert_eq!(user_obj.username, Some(username.to_string()));
    let put_req_email = PutUserRequest {
        username: None,
        email: Some(replacement_email.to_string()),
        password: None,
        contents_description: None,
        contents_attachment: None
    };
    let put_response = app.put_users(jwt.clone(),
                                     user_obj.unique_id.unwrap(), put_req_email).await;
    let put_code = put_response.status();
    assert_eq!(put_code, 200);
    let get_request_body = GetUsersRequest {
        email: None,
        username: Some(String::from(username)),
        user_id: None,
    };
    let get_response = app.get_users(Some(jwt.clone()), get_request_body).await;
    let response_object = get_response.json::<UserResponse>().await.unwrap();
    assert!(response_object.unique_id.as_ref().is_some());
    assert_eq!(response_object.email, Some(replacement_email.to_string()));
    assert_eq!(response_object.username, Some(username.to_string()));
}

#[tokio::test]
pub async fn update_user_username_only() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let email = "testhere@something.net";
    let passwd = "MyBadPassword";
    let replacement_username = "MentallyAbsurd007";
    let (jwt, user_obj) = app.sign_up_get_full_user(
        username, email, Some(passwd), None, None).await;
    assert!(user_obj.unique_id.as_ref().is_some());
    assert_eq!(user_obj.email, Some(email.to_string()));
    assert_eq!(user_obj.username, Some(username.to_string()));
    let (jwt, response_object) = app.put_user_get_user(
        jwt.clone(), user_obj.unique_id.unwrap(),
        username.to_string(), Some(replacement_username.to_string()), None,
        None, None, None).await;
    assert_eq!(response_object.email, Some(email.to_string()));
    assert_eq!(response_object.username, Some(replacement_username.to_string()));
}

#[tokio::test]
pub async fn update_user_username_attachment() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let email = "testhere@something.net";
    let passwd = "MyBadPassword";
    let description_a = "Initial description here!";
    let description_b = "Modified description here!";
    let attachment_a = vec![45u8, 52u8, 0u8, 0u8, 1u8, 22u8, 122u8];
    let attachment_b = vec![0u8, 0u8, 0u8, 0u8, 22u8, 1u8, 23u8];
    let replacement_username = "MentallyAbsurd007";
    let (jwt, user_obj) = app.sign_up_get_full_user(
        username, email, Some(passwd), Some(description_a.to_string()),
        Some(attachment_a.clone())).await;
    assert_eq!(user_obj.email, Some(email.to_string()));
    assert_eq!(user_obj.username, Some(username.to_string()));
    assert_eq!(user_obj.contents_description, Some(description_a.to_string()));
    assert_eq!(user_obj.contents_attachment, Some(attachment_a.clone()));
    let (jwt, response_object) = app.put_user_get_user(
        jwt.clone(), user_obj.unique_id.unwrap(),
        username.to_string(), Some(replacement_username.to_string()), None,
        None, Some(description_b.to_string()),
        Some(attachment_b.clone())).await;
    assert_eq!(response_object.email, Some(email.to_string()));
    assert_eq!(response_object.username, Some(replacement_username.to_string()));
    assert_eq!(response_object.contents_description, Some(description_b.to_string()));
    assert_eq!(response_object.contents_attachment, Some(attachment_b));
}

#[tokio::test]
pub async fn update_user_contents() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let email = "testhere@something.net";
    let passwd = "MyBadPassword";
    let description_a = "Initial description here!";
    let description_b = "Modified description here!";
    let attachment_a = vec![45u8, 52u8, 0u8, 0u8, 1u8, 22u8, 122u8];
    let attachment_b = vec![0u8, 0u8, 0u8, 0u8, 22u8, 1u8, 23u8];
    let (jwt, user_obj) = app.sign_up_get_full_user(
        username, email, Some(passwd),
        Some(description_a.to_string()),
        Some(attachment_a.clone())).await;
    assert_eq!(user_obj.email, Some(email.to_string()));
    assert_eq!(user_obj.username, Some(username.to_string()));
    assert_eq!(user_obj.contents_description, Some(description_a.to_string()));
    assert_eq!(user_obj.contents_attachment, Some(attachment_a.clone()));
    let (jwt, response_object) = app.put_user_get_user(
        jwt.clone(), user_obj.unique_id.unwrap(),
        username.to_string(), None, None,
        None, Some(description_b.to_string()),
        Some(attachment_b.clone())).await;
    assert_eq!(response_object.email, Some(email.to_string()));
    assert_eq!(response_object.username, Some(username.to_string()));
    assert_eq!(response_object.contents_attachment, Some(attachment_b));
    assert_eq!(response_object.contents_description, Some(description_b.to_string()));
}

#[tokio::test]
pub async fn update_user_contents_description_only() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let email = "testhere@something.net";
    let passwd = "MyBadPassword";
    let description_a = "Initial description here!";
    let description_b = "Modified description here!";
    let (jwt, user_obj) = app.sign_up_get_full_user(
        username, email, Some(passwd),
        Some(description_a.to_string()), None).await;
    assert_eq!(user_obj.email, Some(email.to_string()));
    assert_eq!(user_obj.username, Some(username.to_string()));
    assert_eq!(user_obj.contents_description, Some(description_a.to_string()));
    assert_eq!(user_obj.contents_attachment, None);
    let (jwt, response_object) = app.put_user_get_user(
        jwt.clone(), user_obj.unique_id.unwrap(),
        username.to_string(), None, None,
        None, Some(description_b.to_string()),
        None).await;
    assert_eq!(response_object.email, Some(email.to_string()));
    assert_eq!(response_object.username, Some(username.to_string()));
    assert_eq!(response_object.contents_description, Some(description_b.to_string()));
    assert_eq!(response_object.contents_attachment, None);
}