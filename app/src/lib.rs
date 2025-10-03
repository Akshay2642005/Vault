pub mod cli;
pub mod config;
pub mod crypto;
pub mod storage;
pub mod auth;
pub mod sync;
pub mod error;

pub use error::{VaultError, Result};