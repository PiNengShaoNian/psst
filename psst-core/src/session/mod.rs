use crate::connection::Credentials;

pub mod access_token;

/// Configuration values needed to open the session connection.
#[derive(Clone)]
pub struct SessionConfig {
    pub login_creds: Credentials,
    pub proxy_url: Option<String>,
}
