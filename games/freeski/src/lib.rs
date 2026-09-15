//! FreeSki's production simulation is independent of the native frontend.
#[cfg(feature = "ui")]
pub mod app;
#[cfg(feature = "ui")]
mod audio;
pub mod chase;
pub mod collision;
pub mod course;
pub mod endless;
pub mod engine;
#[cfg(feature = "ui")]
mod input;
#[cfg(feature = "ui")]
mod menu;
#[cfg(feature = "ui")]
mod render;
pub mod session;
pub mod storage;
pub mod world;

#[cfg(feature = "ui")]
mod yeti;

#[cfg(feature = "ui")]
mod artwork;
#[cfg(feature = "ui")]
mod geometry;
