pub mod ffi;
pub mod x509;
pub mod secret;
pub mod engine;
pub mod server;
mod error;
mod alert;

use std;

pub use self::error::Error;
pub use self::alert::Alert;
pub type Result<T> = std::result::Result<T, Error>;
