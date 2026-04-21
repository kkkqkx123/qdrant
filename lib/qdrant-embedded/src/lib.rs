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

// Re-export quantization types for user convenience
pub use segment::types::{
    QuantizationConfig,
    CompressionRatio,
    ScalarType,
    ScalarQuantization,
    ScalarQuantizationConfig,
    ProductQuantization,
    ProductQuantizationConfig,
    BinaryQuantization,
    BinaryQuantizationConfig,
    BinaryQuantizationEncoding,
    BinaryQuantizationQueryEncoding,
};

// Re-export conversion helper functions
pub use conversions::{
    scalar_quantization,
    product_quantization,
    binary_quantization,
    binary_quantization_with_encoding,
    default_scalar_quantization,
    default_product_quantization,
    default_binary_quantization,
};
