// Embedded library-specific conversion functions
// Type conversion logic specific to the embedded library schema

use segment::types::{
    QuantizationConfig, ScalarQuantization, ScalarQuantizationConfig, ScalarType,
    ProductQuantization, ProductQuantizationConfig, CompressionRatio,
    BinaryQuantization, BinaryQuantizationConfig, BinaryQuantizationEncoding,
    BinaryQuantizationQueryEncoding,
};

/// Create scalar quantization configuration
///
/// Scalar quantization reduces the precision of vector components to 8-bit integers.
/// This provides a good balance between memory savings and search quality.
///
/// # Arguments
/// * `quantile` - Optional quantile value for quantization (0.5 to 1.0)
/// * `always_ram` - If true, keep quantized data in RAM even if on_disk_payload is true
pub fn scalar_quantization(
    quantile: Option<f32>,
    always_ram: Option<bool>,
) -> QuantizationConfig {
    QuantizationConfig::Scalar(ScalarQuantization {
        scalar: ScalarQuantizationConfig {
            r#type: ScalarType::Int8,
            quantile,
            always_ram,
        },
    })
}

/// Create product quantization configuration with compression
///
/// Product quantization provides high compression ratios by dividing vectors into subvectors
/// and quantizing each subvector independently. This is ideal for large-scale vector search.
///
/// # Arguments
/// * `compression` - Compression ratio (X4, X8, X16, X32, X64)
/// * `always_ram` - If true, keep quantized data in RAM even if on_disk_payload is true
///
/// # Compression Ratios
/// - X4: 4x compression, minimal quality loss
/// - X8: 8x compression, good quality/compression balance
/// - X16: 16x compression, moderate quality loss
/// - X32: 32x compression, significant quality loss
/// - X64: 64x compression, maximum compression, notable quality loss
pub fn product_quantization(
    compression: CompressionRatio,
    always_ram: Option<bool>,
) -> QuantizationConfig {
    QuantizationConfig::Product(ProductQuantization {
        product: ProductQuantizationConfig {
            compression,
            always_ram,
        },
    })
}

/// Create binary quantization configuration
///
/// Binary quantization converts vectors to binary representation (1 bit per dimension).
/// This provides the highest compression but with the most quality loss.
/// Best suited for very large datasets where approximate results are acceptable.
///
/// # Arguments
/// * `always_ram` - If true, keep quantized data in RAM even if on_disk_payload is true
pub fn binary_quantization(always_ram: Option<bool>) -> QuantizationConfig {
    QuantizationConfig::Binary(BinaryQuantization {
        binary: BinaryQuantizationConfig {
            always_ram,
            encoding: None,
            query_encoding: None,
        },
    })
}

/// Create binary quantization configuration with advanced encoding options
///
/// # Arguments
/// * `always_ram` - If true, keep quantized data in RAM
/// * `encoding` - Binary encoding type (OneBit, TwoBits, OneAndHalfBits)
/// * `query_encoding` - Query encoding strategy (Default, Binary, Scalar4Bits, Scalar8Bits)
pub fn binary_quantization_with_encoding(
    always_ram: Option<bool>,
    encoding: Option<BinaryQuantizationEncoding>,
    query_encoding: Option<BinaryQuantizationQueryEncoding>,
) -> QuantizationConfig {
    QuantizationConfig::Binary(BinaryQuantization {
        binary: BinaryQuantizationConfig {
            always_ram,
            encoding,
            query_encoding,
        },
    })
}

/// Create default scalar quantization (8-bit quantization)
pub fn default_scalar_quantization() -> QuantizationConfig {
    scalar_quantization(None, None)
}

/// Create default product quantization with 8x compression
pub fn default_product_quantization() -> QuantizationConfig {
    product_quantization(CompressionRatio::X8, None)
}

/// Create default binary quantization
pub fn default_binary_quantization() -> QuantizationConfig {
    binary_quantization(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_quantization() {
        let config = scalar_quantization(Some(0.99), Some(true));
        match config {
            QuantizationConfig::Scalar(sq) => {
                assert_eq!(sq.scalar.quantile, Some(0.99));
                assert_eq!(sq.scalar.always_ram, Some(true));
            }
            _ => panic!("Expected Scalar quantization"),
        }
    }

    #[test]
    fn test_product_quantization() {
        let config = product_quantization(CompressionRatio::X16, Some(false));
        match config {
            QuantizationConfig::Product(pq) => {
                assert_eq!(pq.product.compression, CompressionRatio::X16);
                assert_eq!(pq.product.always_ram, Some(false));
            }
            _ => panic!("Expected Product quantization"),
        }
    }

    #[test]
    fn test_binary_quantization() {
        let config = binary_quantization(Some(true));
        match config {
            QuantizationConfig::Binary(bq) => {
                assert_eq!(bq.binary.always_ram, Some(true));
            }
            _ => panic!("Expected Binary quantization"),
        }
    }

    #[test]
    fn test_default_quantizations() {
        let _scalar = default_scalar_quantization();
        let _product = default_product_quantization();
        let _binary = default_binary_quantization();
    }
}
