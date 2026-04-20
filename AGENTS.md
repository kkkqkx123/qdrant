# Qdrant Vector Search Engine - Project Context

## Project Overview

Qdrant is a high-performance vector similarity search engine and vector database written in Rust. It provides a production-ready service with a convenient API to store, search, and manage points (vectors with additional payload). Qdrant is tailored for extended filtering support, making it useful for neural-network or semantic-based matching, faceted search, and other AI applications.

### Key Features
- **Vector Search**: High-performance similarity search with support for dense and sparse vectors
- **Filtering**: Advanced filtering capabilities with JSON payloads and complex query conditions
- **Distributed Deployment**: Horizontal scaling through sharding and replication
- **Hybrid Search**: Support for both dense and sparse vectors (BM25/TF-IDF)
- **Vector Quantization**: Memory-efficient storage with up to 97% RAM reduction
- **Multiple APIs**: REST and gRPC interfaces
- **GPU Acceleration**: Optional GPU support for faster indexing
- **SIMD Acceleration**: Hardware acceleration for better performance

## Architecture

The project is organized into several key libraries:

- `api`: REST and gRPC API implementations
- `collection`: Collection management and operations
- `common`: Shared utilities and common functionality
- `segment`: Core vector storage and indexing
- `shard`: Sharding and distribution logic
- `storage`: Storage management and persistence
- `gpu`: GPU acceleration components
- `sparse`: Sparse vector support

## Building and Running

### Prerequisites
- Rust 1.89 or later
- Cargo

### Building from Source
```bash
# Clone the repository
git clone https://github.com/qdrant/qdrant.git
cd qdrant

# Build in release mode
cargo build --release

# Build with specific features (e.g., GPU support)
cargo build --release --features=gpu
```

### Running Qdrant
```bash
# Run with default settings
./target/release/qdrant

# Run with custom configuration
./target/release/qdrant --config-path /path/to/config.yaml

# Run in distributed mode
./target/release/qdrant --bootstrap http://bootstrap-peer:6335 --uri http://this-peer:6335
```

### Docker Deployment
```bash
# Pull the official image
docker pull qdrant/qdrant

# Run with default settings
docker run -p 6333:6333 qdrant/qdrant

# Run with persistent storage
docker run -p 6333:6333 -v $(pwd)/qdrant_storage:/qdrant/storage qdrant/qdrant
```

### Configuration

Qdrant uses a hierarchical configuration system:
1. Compile-time defaults (embedded in binary)
2. Main config file (`config/config.yaml`)
3. Environment-specific config (`config/{RUN_MODE}.yaml`)
4. Local config (`config/local.yaml`)
5. Custom config via `--config-path`
6. Environment variables (prefixed with `QDRANT_`)

Key configuration options:
- `storage.storage_path`: Directory for storing data
- `storage.snapshots_path`: Directory for storing snapshots
- `service.http_port`: HTTP API port (default: 6333)
- `service.grpc_port`: gRPC API port (default: 6334)
- `cluster.enabled`: Enable distributed mode (default: false)

## Development Conventions

### Code Style
- Rust code follows standard Rust formatting (rustfmt)
- Clippy lints are enforced with specific project settings
- Code should be documented with Rust doc comments

### Testing
- Unit tests are located in `tests/` directories within each module
- Integration tests are in the `tests/` directory at project root
- Run tests with `cargo test`

### Features
Qdrant uses Rust features for optional functionality:
- `gpu`: GPU acceleration support
- `tracing`: Advanced tracing capabilities
- `console`: Tokio console support
- `rocksdb`: RocksDB storage backend
- `service_debug`: Additional debugging tools

## API Endpoints

Qdrant provides both REST and gRPC APIs:
- **REST API**: Available at `http://localhost:6333`
- **gRPC API**: Available at `http://localhost:6334`
- **OpenAPI Documentation**: Available online at https://api.qdrant.tech/

## Client Libraries

Qdrant offers multiple client libraries:
- Official: Python, JavaScript/TypeScript, Go, Rust, .NET/C#, Java
- Community: Elixir, PHP, Ruby

## Key Directories and Files

- `src/`: Main application source code
- `lib/`: Core libraries (api, collection, segment, storage, etc.)
- `config/`: Default configuration files
- `docs/`: Documentation
- `openapi/`: API specification files
- `Cargo.toml`: Project dependencies and features
- `Dockerfile`: Container build instructions

## Performance Considerations

- Uses jemalloc as the global allocator for better memory management
- Implements async I/O with Tokio runtime
- Supports SIMD hardware acceleration
- Provides vector quantization to reduce memory usage
- Optimized HNSW index for fast similarity search

## Security

- Supports TLS encryption for both client and inter-cluster communication
- API key authentication
- JWT-based role-based access control (RBAC)
- Configurable CORS settings

## Monitoring and Telemetry

- Prometheus metrics endpoint
- Optional telemetry reporting to Qdrant developers
- Request profiling capabilities
- Hardware utilization reporting (experimental)

## Recovery and Maintenance

- Write-ahead logging (WAL) for data durability
- Snapshot and recovery mechanisms
- Automatic optimization of segments
- Support for recovery mode in case of failures