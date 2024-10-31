#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum AuthenticationType {
    AUTHENTICATION_USER_PASS = 0,
    AUTHENTICATION_STORED_SPOTIFY_CREDENTIALS = 1,
    AUTHENTICATION_STORED_FACEBOOK_CREDENTIALS = 2,
    AUTHENTICATION_SPOTIFY_TOKEN = 3,
    AUTHENTICATION_FACEBOOK_TOKEN = 4,
}

impl Default for AuthenticationType {
    fn default() -> Self {
        AuthenticationType::AUTHENTICATION_USER_PASS
    }
}

impl From<i32> for AuthenticationType {
    fn from(i: i32) -> Self {
        match i {
            0 => AuthenticationType::AUTHENTICATION_USER_PASS,
            1 => AuthenticationType::AUTHENTICATION_STORED_SPOTIFY_CREDENTIALS,
            2 => AuthenticationType::AUTHENTICATION_STORED_FACEBOOK_CREDENTIALS,
            3 => AuthenticationType::AUTHENTICATION_SPOTIFY_TOKEN,
            4 => AuthenticationType::AUTHENTICATION_FACEBOOK_TOKEN,
            _ => Self::default(),
        }
    }
}
