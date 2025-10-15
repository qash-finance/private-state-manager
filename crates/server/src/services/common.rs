use crate::auth::{Auth, Credentials};

pub type ServiceResult<T> = Result<T, ServiceError>;

#[derive(Debug, Clone)]
pub struct ServiceError {
    pub message: String,
}

impl ServiceError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

/// Verify credentials and authorization for a request
pub(crate) fn verify_request_auth(
    auth: &Auth,
    account_id: &str,
    credentials: &Credentials,
) -> ServiceResult<()> {
    auth.verify(account_id, credentials)
        .map_err(|e| ServiceError::new(format!("Authentication failed: {e}")))
}
