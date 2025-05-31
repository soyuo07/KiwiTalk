pub mod client;
pub mod status;
pub mod xvc;

use reqwest::Method;

use crate::{read_response, read_structured_response, ApiResult};

use self::{
    client::{AuthClient, Device},
    xvc::XvcHasher,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize)]
pub struct AccountForm<'a> {
    pub email: &'a str,
    pub password: &'a str,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Login {
    #[serde(rename = "userId")]
    pub user_id: u64,

    #[serde(rename = "countryIso")]
    pub country_iso: String,
    #[serde(rename = "countryCode")]
    pub country_code: String,

    #[serde(rename = "accountId")]
    pub account_id: u64,

    // pub server_time: u64,

    // #[serde(rename = "resetUserData")]
    // pub reset_user_data: bool,
    // pub story_url: Option<String>,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,

    #[serde(rename = "autoLoginAccountId")]
    pub auto_login_account_id: String,
    #[serde(rename = "displayAccountId")]
    pub display_account_id: String,

    #[serde(rename = "mainDeviceAgentName")]
    pub main_device_agent_name: String,
    #[serde(rename = "mainDeviceAppVersion")]
    pub main_device_app_version: String,
}

impl Login {
    pub async fn request_with_account(
        client: AuthClient<'_, impl XvcHasher>,
        account: AccountForm<'_>,
        forced: bool,
    ) -> ApiResult<Self> {
        #[derive(Serialize)]
        struct Form<'a> {
            #[serde(flatten)]
            device: Device<'a>,

            #[serde(flatten)]
            account: AccountForm<'a>,
            forced: bool,
        }

        let form = Form {
            device: client.device,
            account,
            forced,
        };

        read_structured_response(
            client
                .request(Method::POST, "account/login.json", account.email)?
                .form(&form),
        )
        .await
    }

    pub async fn request_with_token(
        client: AuthClient<'_, impl XvcHasher>,
        email: &str,
        token: &str,
        forced: bool,
        locked: bool,
    ) -> ApiResult<Self> {
        #[derive(Serialize)]
        struct Form<'a> {
            #[serde(flatten)]
            device: Device<'a>,

            email: &'a str,
            password: &'a str,
            auto_login: bool,
            autowithlock: bool,
            forced: bool,
        }

        let form = Form {
            device: client.device,
            email,
            password: token,
            auto_login: true,
            autowithlock: locked,
            forced,
        };

        read_structured_response(
            client
                .request(Method::POST, "account/login.json", email)?
                .form(&form),
        )
        .await
    }
}

pub async fn request_passcode(
    client: AuthClient<'_, impl XvcHasher>,
    account: AccountForm<'_>,
) -> ApiResult<()> {
    #[derive(Serialize)]
    struct Form<'a> {
        email: &'a str,
        password: &'a str,
        permanent: bool,
        device: Device<'a>,
    }

    let form = Form {
        email: account.email,
        password: account.password,
        permanent: true,
        device: client.device,
    };
    let json_string = serde_json::to_string(&form)?;

    read_response(
        client
            .request(Method::POST, "account/passcodeLogin/generate", account.email)?
            .header("Content-Type", "application/json; charset=UTF-8")
            .body(json_string),
    )
    .await?;

    Ok(())
}

pub async fn register_device(
    client: AuthClient<'_, impl XvcHasher>,
    account: AccountForm<'_>,
) -> ApiResult<()> {
    #[derive(Serialize, Debug)]
    struct DeviceInfo<'a> {
        uuid: &'a str,
    }

    #[derive(Serialize, Debug)]
    struct Form<'a> {
        email: &'a str,
        password: &'a str,
        device: DeviceInfo<'a>,
    }

    let form = Form {
        email: account.email,
        password: account.password,
        device: DeviceInfo {
            uuid: client.device.uuid,
        },
    };

    read_response(
        client
            .request(Method::POST, "account/passcodeLogin/registerDevice", account.email)?
            .json(&form),
    )
    .await?;

    Ok(())
}