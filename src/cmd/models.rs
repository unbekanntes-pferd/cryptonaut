#[derive(Debug, PartialEq, thiserror::Error)]
pub enum CryptoNautError {
    #[error("DRACOON API error")]
    Http(#[from] dco3::DracoonClientError),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Failed to create log file")]
    LogFileCreationFailed,
}


#[derive(clap::Parser)]
pub struct CryptoNaut {
    pub target_path: String,
    #[clap(long)]
    pub debug: bool,
    #[clap(long)]
    pub log_file_path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct CryptoNautConfig {
    client_id: String,
    client_secret: String,
    refresh_token: String,
    rescue_key: String
}

impl CryptoNautConfig {
    pub fn get_client_id(&self) -> &str {
        &self.client_id
    }

    pub fn get_client_secret(&self) -> &str {
        &self.client_secret
    }

    pub fn get_refresh_token(&self) -> &str {
        &self.refresh_token
    }

    pub fn get_rescue_key(&self) -> &str {
        &self.rescue_key
    }
}
