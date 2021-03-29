#![deny(
bad_style,
trivial_casts,
trivial_numeric_casts,
unsafe_code,
unstable_features
)]
#![cfg_attr(not(debug_assertions), deny(warnings))]

pub mod channel;
pub mod payload;
pub mod utility;
pub mod users;

