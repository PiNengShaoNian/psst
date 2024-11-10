use crate::{connection::{Credentials, Transport}, error::Error};

pub mod access_token;

/// Configuration values needed to open the session connection.
#[derive(Clone)]
pub struct SessionConfig {
    pub login_creds: Credentials,
    pub proxy_url: Option<String>,
}

/// Successful connection through the Spotify Shannon-encrypted TCP channel.
pub struct SessionConnection {
    /// Credentials re-usable in the next authentication (i.e. username and
    /// password are not required anymore).
    pub credentials: Credentials,
    /// I/O codec for the Shannon messages.
    pub transport: Transport,
}

impl SessionConnection {
    /// Synchronously connect to the Spotify servers and authenticate with
    /// credentials provided in `config`.
    pub fn open(config: SessionConfig) -> Result<Self, Error> {
        // Connect to the server and exchange keys.
        let proxy_url = config.proxy_url.as_deref();
        let ap_url = Transport::resolve_ap_with_fallback(proxy_url);
        let mut transport = Transport::connect(&ap_url, proxy_url)?;
        // Authenticate with provided credentials (either username/password, or saved,
        // reusable credential blob from an earlier run).
        let credentials = transport.authenticate(config.login_creds)?;
        Ok(Self {
            credentials,
            transport,
        })
    }
}
