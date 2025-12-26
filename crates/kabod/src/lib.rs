//! Kabod: Vector Database ORM
//! 
//! This crate provides the core implementation of the Kabod ORM.

pub mod config;
pub mod error;
pub mod types;
pub mod db;
pub mod adapters;
pub mod query;
pub mod client;

pub use client::KabodClient;
pub use error::{Result, KabodError};
