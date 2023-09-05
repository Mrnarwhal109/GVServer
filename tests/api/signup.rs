use gvserver::routes::post::SignUpData;
use crate::helpers::{spawn_app};
use gvserver::database_direct_models::DbUser;
use crate::TestApp;

#[tokio::test]
async fn sign_up_persists_users() {
    let app = spawn_app().await;
    let username = String::from("SomeDudeHere");
    let sign_up_data = SignUpData {
        email: String::from("somedude@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = app.post_signup(json_data).await;

    // Assert
    assert_eq!(response.status(), 200);

    let user_rows = sqlx::query_as!(
        DbUser,
        "SELECT U.id AS unique_id, U.email AS email, U.username AS username, \
        U.phash AS phash, U.salt AS salt, R.id AS role_id, R.title AS role_title \
        FROM users U \
        INNER JOIN user_roles UR on UR.user_id = U.id \
        INNER JOIN roles R on UR.role_id = R.id \
        WHERE U.username = $1; ", username).fetch_one(&app.db_pool).await;
        //.fetch_all(&app.db_pool).await;

    match user_rows {
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
    let sign_up_data = SignUpData {
        email: String::from("somedude@gmail.com"),
        username: username.clone(),
        pw: String::from("$uper$ecurePa$$word!AsWell!!!")
    };

    let json_data = serde_json::to_string(&sign_up_data)
        .expect("Failed to serialize struct.");

    let response = running_app.post_signup(json_data).await;

    // Assert
    assert_eq!(response.status(), 500);

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