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

mod api;
mod auth;
mod error;
mod models;
mod pockety;

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// pub struct PocketGetResponse {
//     list: Vec<PocketItem>, // must be Vec
//     status: u16,
//     complete: bool, // must be bool
//     error: Option<String>,
//     //search_meta: PocketSearchMeta,
//     since: Timestamp,
// }

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// pub struct PocketSendRequest {
//     pocket: Pocket,
//     actions: &[PocketActionName],
// }

// #[derive(serde::Serialize, serde::Deserialize, Debug)]
// pub struct PocketSendResponse {
//     status: u16,
//     action_results: Vec<bool>,
// }
