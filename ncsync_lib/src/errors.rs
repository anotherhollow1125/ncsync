use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum NcsError {
    #[error("Failed Locking.")]
    LockError,
    #[error("Bad status {0}.")]
    BadStatusError(u16),
    #[error("Invalid XML.")]
    InvalidXMLError,
    #[error("Failed upgrade Weak")]
    WeakUpgradeError,
    #[error("Invalid path. {0}")]
    InvalidPathError(String),
    #[error("Network is offline.")]
    NetworkOfflineError,
    #[error("Not Logged in.")]
    NotLoggedIn,
    #[error("Failed to Access. You would have been logged out.")]
    NotAuthorized,
    #[error("Invalid Path.")]
    BadPath,
    #[error("Profile not found {0}.")]
    ProfileNotFound(String),
    #[error("Invalid Profile. Please check profiles.toml")]
    InvalidProfile,
}
