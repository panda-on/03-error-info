use error_code::ToErrorInfo;

#[derive(Debug, ToErrorInfo, thiserror::Error)]
#[error_info(app_type = "http::StatusCode", prefix = "01")]
pub enum MyError {
    #[error("Invalid command: {0}")]
    #[error_info(code = "IC", app_code = "400")]
    InvalidCommand(String),

    #[error("Internal argument: {0}")]
    #[error_info(code = "IA", app_code = "400", client_msg = "friendly msg")]
    InternalArgument(String),

    #[error("{0}")]
    #[error_info(code = "RE", app_code = "500")]
    RespError(#[from] std::io::Error),
}

fn main() {
    let err = MyError::InvalidCommand("cmd".to_string());
    let info = err.to_error_info();
    println!("{:?}", info);
}
