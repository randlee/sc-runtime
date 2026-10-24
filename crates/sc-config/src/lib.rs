#![doc = include_str!("../README.md")]

mod app_name;
mod error;
mod load;

#[doc(inline)]
pub use app_name::AppName;
#[doc(inline)]
pub use error::ConfigError;
#[doc(inline)]
pub use load::{load, Loader};
