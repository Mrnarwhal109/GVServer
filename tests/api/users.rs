use log::debug;
use crate::helpers::{spawn_app};
use gvserver::database_models::DbUser;
use gvserver::domain::user_sign_up::UserSignUp;
use gvserver::routes::users::get::{GetUsersRequest, UserResponse};
use gvserver::routes::users::post::PostUserRequest;
use crate::TestApp;

#[tokio::test()]
async fn sign_up_persists_users() {
    let app = spawn_app().await;
    let username = String::from("SomeDudeHere");
    let pw = String::from("$uper$ecurePa$$word!");
    let sign_up_data = PostUserRequest {
        email: String::from("somedude@gmail.com"),
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data, username.clone(), pw).await;

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

    sign_up_rejects_a_duplicate_user(&app).await;
}

async fn sign_up_rejects_a_duplicate_user(running_app: &TestApp) {
    let username = String::from("SomeDudeHere");
    let sign_up_data = UserSignUp {
        email: String::from("somedude@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!AsWell!!!")
    };

    let json_data = serde_json::to_string(&sign_up_data.email)
        .expect("Failed to serialize struct.");

    let response = running_app.post_signup(json_data, sign_up_data.username, sign_up_data.pw).await;

    // Assert
    assert_eq!(response.status(), 400);

    let user_rows = sqlx::query_as!(
        DbUser,
        "SELECT U.id AS unique_id, U.email AS email, U.username AS username, \
        U.phash AS phash, U.salt AS salt, R.id AS role_id, R.title AS role_title \
        FROM users U \
        INNER JOIN user_roles UR on UR.user_id = U.id \
        INNER JOIN roles R on UR.role_id = R.id \
        WHERE U.username = $1; ", username).fetch_one(&running_app.db_pool).await;
    //.fetch_all(&app.db_pool).await;

    match user_rows {
        Ok(row) => {
            println!("DbUser returned with username {}, email {}, phash {}, salt {} \
            role_id {}, role_title {}",
                     &row.username, &row.email, &row.phash, &row.salt,
            &row.role_id, &row.role_title);
            assert_eq!(row.username, username)
        }
        Err(e) => {
            println!("Query error: {}", e.to_string());
            panic!();
        }
    }
}

#[tokio::test]
pub async fn get_users_username_only() {
    let app = spawn_app().await;
    let username = "MentallyAbsurd";
    let pw = "$uper$ecurePa$$word!";
    let email = "mentallyabsurd@gmail.com";
    let jwt = app.sign_up_test_user(username, pw, email).await;

    let request_body = GetUsersRequest {
        email: None,
        username: Some(username.to_string()),
        user_id: None,
    };

    let response = app.get_users(jwt, request_body).await;

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
    let pw = "$uper$ecurePa$$word!";
    let email = "mentallyabsurd@gmail.com";
    let jwt = app.sign_up_test_user(username, pw, email).await;

    let request_body = GetUsersRequest {
        email: None,
        username: Some(String::from("IProbablyDoNotExist")),
        user_id: None,
    };

    let response = app.get_users(jwt, request_body).await;
    let code = (&response.status()).clone();
    assert_eq!(response.content_length().unwrap(), 0);
    assert_eq!(code, 200);
}