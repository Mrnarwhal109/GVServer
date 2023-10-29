use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
pub struct DeletePinpointRequest {
    pub pinpoint_id: Option<Uuid>,
    pub username: Option<String>
}