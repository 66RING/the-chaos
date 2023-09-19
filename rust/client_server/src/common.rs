use std::io::{self, BufWriter, BufReader}; 
use std::net::TcpStream;
use serde::{Deserialize, Serialize};
use serde_json::StreamDeserializer;
use serde_json::{Deserializer, de::IoRead};

#[derive(Debug)]
pub enum MyError {
    Any(String),
}

pub type Result<T> = std::result::Result<T, MyError>;

impl From<io::Error> for MyError {
    fn from(_: io::Error) -> MyError {
        MyError::Any("io err".to_string())
    }
}

impl From<serde_json::Error> for MyError {
    fn from(_: serde_json::Error) -> MyError {
        MyError::Any("serde err".to_string())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Echo(String),
    Hello,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GetResponse {
    Ok(Option<String>),
    Err(String),
}
