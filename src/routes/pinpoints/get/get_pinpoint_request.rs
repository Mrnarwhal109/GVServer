use std::fmt::{Display, Formatter};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct GetPinpointRequest {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub proximity: Option<f64>,
    pub pinpoint_id: Option<Uuid>,
    pub username: Option<String>
}

impl Display for GetPinpointRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let lat = match &self.latitude {
            None => String::from("NONE"),
            Some(x) => x.to_string()
        };
        let lng = match &self.longitude {
            None => String::from("NONE"),
            Some(x) => x.to_string()
        };
        let prox = match &self.proximity {
            None => String::from("NONE"),
            Some(x) => x.to_string()
        };
        let ppid = match &self.pinpoint_id {
            None => String::from("NONE"),
            Some(x) => x.to_string()
        };
        let usrn = match &self.username {
            None => String::from("NONE"),
            Some(x) => x.to_string()
        };
        write!(f, "GetPinpointRequest(lat is {}, lng is {}, prox is {}, pinpoint_id is {}, usrn is {}).",
               lat, lng, prox, ppid, usrn)
    }
}