mod types;

use serde_json::json;

pub use types::{Account, Device, Region, Task};
use types::{DevicesResponse, LoginResponse, TasksResponse, Token};

#[derive(Debug)]
pub struct Client {
    region: Region,
    client: reqwest::Client,
    pub(crate) auth_token: Token,
}

#[derive(Debug, thiserror::Error)]
pub enum LoginError {
    #[error("failed to send login request")]
    Reqwest(#[from] reqwest::Error),

    #[error("failed to parse login response")]
    Decode(#[from] jsonwebtoken::errors::Error),
}

impl Client {
    /// Create a new client by logging in with the provided credentials.
    ///
    /// # Errors
    ///
    /// This function can return a [`LoginError`] if the login request fails or the response cannot be decoded.
    pub async fn login(region: Region, email: &str, password: &str) -> Result<Self, LoginError> {
        let client = reqwest::Client::new();

        let response = client
            .post(if region.is_china() {
                "https://api.bambulab.cn/v1/user-service/user/login"
            } else {
                "https://api.bambulab.com/v1/user-service/user/login"
            })
            .json(&json!({ "account": email, "password": password }))
            .send()
            .await?
            .error_for_status()?
            .json::<LoginResponse>()
            .await?;

        Ok(Self {
            region,
            client,
            auth_token: Token::try_from(response.access_token)?,
        })
    }

    /// Get the account profile for the logged-in user.
    ///
    /// # Errors
    ///
    /// This function can return a [`reqwest::Error`] if the request fails.
    pub async fn get_profile(&self) -> Result<Account, reqwest::Error> {
        self.client
            .get(if self.region.is_china() {
                "https://api.bambulab.cn/v1/user-service/my/profile"
            } else {
                "https://api.bambulab.com/v1/user-service/my/profile"
            })
            .header("Authorization", format!("Bearer {}", self.auth_token.jwt))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }

    /// Get a list of devices associated with the account.
    ///
    /// # Errors
    ///
    /// This function can return a [`reqwest::Error`] if the request fails.
    pub async fn get_devices(&self) -> Result<Vec<Device>, reqwest::Error> {
        let response = self
            .client
            .get(if self.region.is_china() {
                "https://api.bambulab.cn/v1/iot-service/api/user/bind"
            } else {
                "https://api.bambulab.com/v1/iot-service/api/user/bind"
            })
            .header("Authorization", format!("Bearer {}", self.auth_token.jwt))
            .send()
            .await?
            .error_for_status()?
            .json::<DevicesResponse>()
            .await?;

        Ok(response.devices)
    }

    /// Get a list of tasks associated with the account.
    ///
    /// # Errors
    ///
    /// This function can return a [`reqwest::Error`] if the request fails.
    pub async fn get_tasks(
        &self,
        only_device: Option<String>,
    ) -> Result<Vec<Task>, reqwest::Error> {
        let response = self
            .client
            .get(if self.region.is_china() {
                "https://api.bambulab.cn/v1/user-service/my/tasks"
            } else {
                "https://api.bambulab.com/v1/user-service/my/tasks"
            })
            .query(&[
                ("limit", "500".to_string()),
                ("deviceId", only_device.unwrap_or_default()),
            ])
            .header("Authorization", format!("Bearer {}", self.auth_token.jwt))
            .send()
            .await?
            .error_for_status()?
            .json::<TasksResponse>()
            .await?;

        Ok(response.hits)
    }

    /// Get the MQTT host for the client's region.
    #[must_use]
    pub const fn mqtt_host(&self) -> &str {
        if self.region.is_china() {
            "cn.mqtt.bambulab.com"
        } else {
            "us.mqtt.bambulab.com"
        }
    }
}
