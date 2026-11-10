//! Shared Pixa account state and signed update validation. No billing mutations or installer execution.
#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable
)]
pub mod account;
pub mod help;
pub mod updates;
