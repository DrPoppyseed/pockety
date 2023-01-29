#![deny(
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_features,
    unused_import_braces,
    unused_qualifications
)]

pub mod api;
pub mod auth;
mod error;
pub use error::{ApiError, Error, HttpError};
pub mod models;

mod pockety;
pub use pockety::{Auth, Pockety, PocketyUrl};

pub use reqwest;
