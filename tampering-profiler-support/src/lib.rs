use thiserror::Error;

#[derive(Error, Debug)]
pub enum Errors {
    #[error("tapering detected - excessive function execution time")]
    TamperingDetected(String, u64)
}