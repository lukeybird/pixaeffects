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
pub mod guard;
pub mod help;
pub mod recovery;
pub mod updates;

pub mod account_worker;
pub mod credentials;

pub mod release_feed;
