use thiserror::Error;

#[derive(Error, Debug)]
pub enum ThreadError {
    
}

pub type Result<T> = std::result::Result<T, ThreadError>;
