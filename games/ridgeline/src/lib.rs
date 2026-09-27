//! Ridgeline: an original tower-defence campaign for Omarchy Arcade.
#[cfg(feature = "desktop")]
pub mod app;
#[cfg(feature = "desktop")]
mod art;
#[cfg(feature = "desktop")]
mod audio;
pub mod battle;
pub mod data;
pub mod storage;
pub mod strategy;
