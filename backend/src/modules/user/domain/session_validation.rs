use actix_session::Session;
use eyre::Context;

#[derive(thiserror::Error, Debug)]
pub enum SessionValidationError {
    #[error("User is not logged in.")]
    NotLoggedIn,

    #[error(transparent)]
    UnexpectedError(#[from] eyre::Error),
}

pub fn validate_session(session: &Session) -> Result<i32, SessionValidationError> {
    let user_id: Option<i32> = session
        .get("user_id")
        .wrap_err("Session get error")
        .map_err(SessionValidationError::UnexpectedError)?;

    match user_id {
        Some(id) => {
            // Keep session alive
            session.renew();
            Ok(id)
        }
        None => Err(SessionValidationError::NotLoggedIn),
    }
}
