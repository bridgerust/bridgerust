//! Kabod: Vector Database ORM
//!
//! This crate provides the core implementation of the Kabod ORM.

pub mod adapters;
pub mod client;
pub mod config;
pub mod db;
pub mod error;
pub mod query;
pub mod types;

pub use client::KabodClient;
pub use error::{KabodError, Result};
