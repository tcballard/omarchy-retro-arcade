//! UI-independent Tanks engine. See ../README.md for the versioned rules.
#![forbid(unsafe_code)]

pub mod rules;

pub mod ai;

#[cfg(feature = "desktop")]
pub mod app;
pub mod storage;

#[cfg(feature = "desktop")]
mod audio;
pub mod effects;
