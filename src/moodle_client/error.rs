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

pub fn request_error_fromr_reqe(err:reqwest::Error) ->MoodleClientError
{
    println!("b:{},r:{},st:{},time:{},req:{},body:{},dec:{},stat2:{:?}",err.is_builder(),err.is_redirect(),err.is_status(),err.is_timeout(),err.is_request(),err.is_body(),err.is_decode(),err.status());
    println!("{}-total error",err);
    MoodleClientError::RequestError
}