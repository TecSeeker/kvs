use failure::Fail;

#[derive(Fail, Debug)]
pub enum KvError {
    #[fail(display = "Key not found")]
    KeyNotFound,
    #[fail(display = "IO error: {}", _0)]
    Io(#[cause] std::io::Error),
    #[fail(display = "Serde error: {}", _0)]
    Serde(#[cause] serde_json::Error),
    
}

impl From<std::io::Error> for KvError {
    fn from(err: std::io::Error) -> KvError {
        KvError::Io(err)
    }
}

impl From<serde_json::Error> for KvError {
    fn from(err: serde_json::Error) -> KvError {
        KvError::Serde(err)
    }
}
