use std::str::FromStr;

use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, TokenData, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use url::Url;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Region {
    China,
    Europe,
    NorthAmerica,
    AsiaPacific,
    Other,
}

impl Region {
    pub(crate) const fn is_china(self) -> bool {
        matches!(self, Self::China)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Device {
    pub name: String,
    pub online: bool,
    pub dev_id: String,
    pub print_status: String,
    pub nozzle_diameter: f64,
    pub dev_model_name: String,
    pub dev_access_code: String,
    pub dev_product_name: String,
}

impl Device {
    /// Get the streaming URL for the camera on this device.
    ///
    /// # Errors
    ///
    /// This function can return a [`reqwest::Error`] if the request fails.
    pub async fn get_bambu_camera_url(
        &self,
        client: &super::Client,
    ) -> Result<Url, DeviceCameraError> {
        let response = client
            .client
            .post(if client.region.is_china() {
                "https://api.bambulab.cn/v1/iot-service/api/user/ttcode"
            } else {
                "https://api.bambulab.com/v1/iot-service/api/user/ttcode"
            })
            .header(
                "Authorization",
                &format!("Bearer {}", client.auth_token.jwt),
            )
            .header("user-id", client.auth_token.username.clone())
            .json(&json!({ "dev_id": self.dev_id }))
            .send()
            .await?
            .error_for_status()?
            .json::<DeviceCameraResponse>()
            .await?;

        Ok(Url::from_str(&format!(
            "bambu:///{}?authkey={}&passwd={}&region={}",
            response.ttcode, response.authkey, response.passwd, response.region
        ))?)
    }
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: u64,
    pub design_id: u64,
    pub design_title: String,
    pub instance_id: u64,
    pub model_id: String,
    pub title: String,
    pub cover: Url,
    pub status: u64,
    pub feedback_status: u64,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub weight: f64,
    pub length: u64,
    pub cost_time: u64,
    pub profile_id: u64,
    pub plate_index: usize,
    pub plate_name: String,
    pub device_id: String,
    pub ams_detail_mapping: Vec<AMSDetail>,
    pub mode: String,
    pub is_public_profile: bool,
    pub is_printable: bool,
    pub device_model: String,
    pub device_name: String,
    pub bed_type: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AMSDetail {
    #[serde(rename = "ams")]
    pub position: usize,
    pub source_color: String,
    pub target_color: String,
    pub filament_id: String,
    pub filament_type: String,
    pub target_filament_type: String,
    pub weight: f64,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub uid: u64,
    #[serde(rename = "account")]
    pub email: String,
    pub name: String,
    pub avatar: Url,
    pub fan_count: u64,
    pub follow_count: u64,
    pub like_count: u64,
    pub collection_count: u64,
    pub download_count: u64,
    pub product_models: Vec<String>,
    pub my_like_count: u64,
    pub favourites_count: u64,
    pub point: u64,
    pub personal: Personal,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Personal {
    pub bio: String,
    pub links: Vec<Url>,
    pub task_weight_sum: f64,
    pub task_length_sum: u64,
    pub task_time_sum: u64,
    pub background_url: Url,
}

#[derive(Debug)]
pub struct Token {
    pub username: String,
    pub(crate) jwt: String,
}

#[derive(Debug, Deserialize)]
struct JWTData {
    username: String,
}

impl TryFrom<String> for Token {
    type Error = jsonwebtoken::errors::Error;

    fn try_from(jwt: String) -> Result<Self, Self::Error> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.insecure_disable_signature_validation();
        validation.validate_aud = false;

        let token: TokenData<JWTData> =
            jsonwebtoken::decode(&jwt, &DecodingKey::from_secret(&[]), &validation)?;

        Ok(Self {
            jwt,
            username: token.claims.username,
        })
    }
}

#[derive(serde::Deserialize)]
pub struct LoginResponse {
    #[serde(rename = "accessToken")]
    pub(crate) access_token: String,
}

#[derive(serde::Deserialize)]
pub struct DevicesResponse {
    pub devices: Vec<Device>,
}

#[derive(Debug, serde::Deserialize)]
pub struct TasksResponse {
    pub total: usize,
    pub hits: Vec<Task>,
}

#[derive(Debug, serde::Deserialize)]
struct DeviceCameraResponse {
    ttcode: String,
    authkey: String,
    passwd: String,
    region: String,
}

#[derive(Debug, thiserror::Error)]
pub enum DeviceCameraError {
    #[error("failed to get camera URL")]
    Reqwest(#[from] reqwest::Error),

    #[error("failed to parse camera URL")]
    Url(#[from] url::ParseError),
}
