pub mod client;
pub mod config;
pub mod error;
pub mod operations;
pub mod conversions;

#[cfg(test)]
mod tests;

pub use client::QdrantEmbedded;
pub use config::{EmbeddedConfig, EmbeddedConfigBuilder};
pub use error::{EmbeddedError, Result};
