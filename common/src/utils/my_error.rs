#[derive(Debug)]
pub enum MyError {
    Io(std::io::Error),
    Json(serde_json::Error),
    Other(String),
}

impl From<String> for MyError {
    fn from(err: String) -> Self {
        MyError::Other(err)
    }
}
impl From<std::io::Error> for MyError {
    fn from(err: std::io::Error) -> Self {
        MyError::Io(err)
    }
}

impl From<serde_json::Error> for MyError {
    fn from(err: serde_json::Error) -> Self {
        MyError::Json(err)
    }
}
