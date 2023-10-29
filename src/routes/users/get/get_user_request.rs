use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct GetUsersRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub user_id: Option<Uuid>
}