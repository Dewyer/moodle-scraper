use thiserror::Error;

#[derive(Error, Debug)]
pub enum MoodleClientError
{
    #[error("FailedToCreateClient")]
    FailedToCreateClient,
    #[error("RequestError")]
    RequestError,
    #[error("LoginError")]
    LoginError,
    #[error("LoadBodyError")]
    LoadBodyError,
    #[error("DataNotFound")]
    DataNotFound,
    #[error("ElementNotFound")]
    ElementNotFound
}