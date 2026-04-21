# Qdrant 嵌入库架构分析

## 概述

本文档分析了如何将 Qdrant 项目改造为同时支持服务端和嵌入库两种模式。嵌入库模式允许将 Qdrant 作为库直接集成到应用程序中，无需运行独立的服务进程。

## 当前架构分析

### 项目结构层次

```
qdrant/
├── src/                    # 主服务层
│   ├── actix/             # REST API 服务端
│   ├── tonic/             # gRPC API 服务端
│   └── main.rs            # 服务入口
├── lib/
│   ├── api/               # API 层 - 数据模型和转换
│   ├── storage/           # 存储层 - TableOfContent, Dispatcher
│   ├── collection/        # 集合层 - 单个集合管理
│   ├── shard/             # 分片层 - 分片操作
│   ├── segment/           # 段层 - 核心向量存储和索引
│   └── edge/              # 现有嵌入库实现（简化版）
│       ├── src/
│       └── python/        # Python 绑定
└── Cargo.toml
```

### 各层职责

| 层级 | 职责 | 嵌入库需求 |
|------|------|-----------|
| **主服务层** | 提供 HTTP/gRPC 网络接口 | ❌ 不需要 |
| **API 层** | 定义请求/响应数据模型 | ✅ 需要（共享） |
| **存储层** | 管理集合生命周期和请求分发 | ✅ 需要（核心） |
| **集合层** | 单个集合的 CRUD 操作 | ✅ 需要 |
| **分片层** | 分片管理和 WAL | ✅ 需要 |
| **段层** | 向量存储、索引、搜索 | ✅ 需要 |

### 现有嵌入库实现

项目已有一个简化的嵌入库实现 `lib/edge/`，其特点：

- **直接操作底层**: 直接使用 `Shard` 和 `Segment`，跳过了 `Collection` 和 `Storage` 层
- **功能受限**: 不支持多集合、别名、快照等高级功能
- **Python 绑定**: 通过 PyO3 提供 Python 接口
- **适用场景**: 单集合、高性能、低延迟的嵌入式场景

### API Crate 当前结构

```rust
// lib/api/src/lib.rs
pub mod conversions;
pub mod grpc;      // gRPC 相关
pub mod rest;      // REST 相关

// lib/api/Cargo.toml
[dependencies]
tonic = { workspace = true }
tonic-build = { workspace = true }
prost-build = { workspace = true }
actix-web = "4.11.0"
actix-web-validator = "7.0.0"
```

## 改造方案：扩展 API Crate 为双模式库

### 设计原则

1. **代码复用**: 最大化复用现有的数据模型和转换逻辑
2. **功能对等**: 嵌入库应提供与客户端 API 相同的功能
3. **依赖隔离**: 嵌入库模式不引入网络相关依赖
4. **易于使用**: 提供简洁的高层 API 接口

### 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                        API Crate                            │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │  Server Mode    │  │  Embedded Mode  │                  │
│  │  (feature)      │  │  (feature)      │                  │
│  └─────────────────┘  └─────────────────┘                  │
│         │                      │                            │
│         ▼                      ▼                            │
│  ┌─────────────────┐  ┌─────────────────┐                  │
│  │  grpc/          │  │  embedded/      │                  │
│  │  rest/          │  │  client.rs      │                  │
│  │  conversions/   │  │  operations.rs  │                  │
│  └─────────────────┘  └─────────────────┘                  │
│         │                      │                            │
│         └──────────┬───────────┘                            │
│                    ▼                                        │
│         ┌──────────────────────┐                           │
│         │  conversions/        │  (共享)                    │
│         │  - 数据模型          │                            │
│         │  - 类型转换          │                            │
│         └──────────────────────┘                           │
└─────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────┐
│                    Storage Crate                             │
│  - TableOfContent (集合管理)                                │
│  - Dispatcher (请求分发)                                    │
│  - CollectionMetaOperations                                 │
└─────────────────────────────────────────────────────────────┘
```

## 实现计划

### 阶段 1: 基础架构搭建

#### 1.1 修改 `lib/api/Cargo.toml`

```toml
[package]
name = "api"
version = "1.16.2"

[features]
default = ["server"]
server = [
    "dep:tonic",
    "dep:tonic-build",
    "dep:prost-build",
    "dep:actix-web",
    "dep:actix-web-validator",
]
embedded = []
tracing = ["dep:tracing", "segment/tracing"]

[dependencies]
# 核心依赖（两种模式都需要的）
ahash = { workspace = true }
prost = { workspace = true }
prost-wkt-types = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
schemars = { workspace = true }
uuid = { workspace = true }
tokio = { workspace = true }
rand = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
parking_lot = { workspace = true }
validator = { workspace = true }
itertools = { workspace = true }
ordered-float = { workspace = true }

common = { path = "../common/common" }
segment = { path = "../segment", default-features = false }
sparse = { path = "../sparse" }

# 服务端专用依赖
tonic = { workspace = true, optional = true }
tonic-build = { workspace = true, optional = true }
prost-build = { workspace = true, optional = true }
actix-web = { version = "4.11.0", optional = true }
actix-web-validator = { version = "7.0.0", optional = true }

# 嵌入库专用依赖
tracing = { workspace = true, optional = true }

[build-dependencies]
tonic-build = { workspace = true, optional = true }
prost-build = { workspace = true, optional = true }
common = { path = "../common/common" }
```

#### 1.2 重构 `lib/api/src/lib.rs`

```rust
pub mod conversions;

#[cfg(feature = "server")]
pub mod grpc;

#[cfg(feature = "server")]
pub mod rest;

#[cfg(feature = "embedded")]
pub mod embedded;

// 重新导出常用类型
pub use conversions::rest::models::*;
```

#### 1.3 创建嵌入库模块结构

```
lib/api/src/embedded/
├── mod.rs              # 模块入口
├── client.rs           # 嵌入库客户端
├── operations.rs       # 操作定义
├── conversions.rs      # 嵌入库专用转换
├── config.rs           # 配置管理
└── error.rs            # 错误类型
```

### 阶段 2: 核心功能实现

#### 2.1 配置管理 (`config.rs`)

```rust
use std::path::PathBuf;

/// 嵌入库配置
#[derive(Debug, Clone)]
pub struct EmbeddedConfig {
    /// 存储路径
    pub storage_path: PathBuf,
    /// 快照路径（可选）
    pub snapshots_path: Option<PathBuf>,
    /// 临时文件路径（可选）
    pub temp_path: Option<PathBuf>,
    /// 搜索线程数
    pub search_threads: Option<usize>,
    /// 优化线程数
    pub optimizer_threads: Option<usize>,
    /// IO 限制（可选）
    pub io_limit: Option<usize>,
    /// CPU 限制（可选）
    pub cpu_limit: Option<usize>,
}

impl Default for EmbeddedConfig {
    fn default() -> Self {
        Self {
            storage_path: PathBuf::from("./qdrant_storage"),
            snapshots_path: None,
            temp_path: None,
            search_threads: None,
            optimizer_threads: None,
            io_limit: None,
            cpu_limit: None,
        }
    }
}

impl EmbeddedConfig {
    pub fn builder() -> EmbeddedConfigBuilder {
        EmbeddedConfigBuilder::default()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmbeddedConfigBuilder {
    config: EmbeddedConfig,
}

impl EmbeddedConfigBuilder {
    pub fn storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.storage_path = path.into();
        self
    }

    pub fn snapshots_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.snapshots_path = Some(path.into());
        self
    }

    pub fn temp_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.temp_path = Some(path.into());
        self
    }

    pub fn search_threads(mut self, threads: usize) -> Self {
        self.config.search_threads = Some(threads);
        self
    }

    pub fn optimizer_threads(mut self, threads: usize) -> Self {
        self.config.optimizer_threads = Some(threads);
        self
    }

    pub fn io_limit(mut self, limit: usize) -> Self {
        self.config.io_limit = Some(limit);
        self
    }

    pub fn cpu_limit(mut self, limit: usize) -> Self {
        self.config.cpu_limit = Some(limit);
        self
    }

    pub fn build(self) -> EmbeddedConfig {
        self.config
    }
}
```

#### 2.2 错误类型 (`error.rs`)

```rust
use thiserror::Error;

/// 嵌入库错误类型
#[derive(Error, Debug)]
pub enum EmbeddedError {
    #[error("Storage error: {0}")]
    Storage(#[from] storage::content_manager::errors::StorageError),

    #[error("Operation error: {0}")]
    Operation(#[from] segment::common::operation_error::OperationError),

    #[error("Collection not found: {0}")]
    CollectionNotFound(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, EmbeddedError>;
```

#### 2.3 嵌入库客户端 (`client.rs`)

```rust
use std::path::Path;
use std::sync::Arc;
use tokio::runtime::Runtime;

use storage::content_manager::toc::TableOfContent;
use storage::dispatcher::Dispatcher;
use storage::types::StorageConfig;
use storage::rbac::Access;

use crate::embedded::config::EmbeddedConfig;
use crate::embedded::error::{EmbeddedError, Result};

/// Qdrant 嵌入库客户端
pub struct QdrantEmbedded {
    toc: Arc<TableOfContent>,
    dispatcher: Arc<Dispatcher>,
    _search_runtime: Runtime,
    _update_runtime: Runtime,
    _general_runtime: Runtime,
}

impl QdrantEmbedded {
    /// 创建新的嵌入库实例
    pub fn new(config: EmbeddedConfig) -> Result<Self> {
        Self::new_with_runtime(config, None)
    }

    /// 使用指定的 Tokio runtime 创建实例
    pub fn new_with_runtime(
        config: EmbeddedConfig,
        runtime: Option<Runtime>,
    ) -> Result<Self> {
        // 创建或使用提供的 runtime
        let general_runtime = runtime.unwrap_or_else(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to create runtime")
        });

        let search_threads = config.search_threads.unwrap_or(4);
        let search_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(search_threads)
            .thread_name("search")
            .enable_all()
            .build()
            .expect("Failed to create search runtime");

        let optimizer_threads = config.optimizer_threads.unwrap_or(2);
        let update_runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(optimizer_threads)
            .thread_name("update")
            .enable_all()
            .build()
            .expect("Failed to create update runtime");

        // 转换为 StorageConfig
        let storage_config = StorageConfig {
            storage_path: config.storage_path.to_string_lossy().to_string(),
            snapshots_path: config.snapshots_path
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| {
                    config.storage_path.join("snapshots").to_string_lossy().to_string()
                }),
            temp_path: config.temp_path
                .map(|p| p.to_string_lossy().to_string()),
            performance: storage::types::PerformanceConfig {
                max_search_threads: config.search_threads,
                max_optimization_runtime_threads: config.optimizer_threads,
                optimizer_cpu_budget: config.cpu_limit,
                optimizer_io_budget: config.io_limit,
                ..Default::default()
            },
            ..Default::default()
        };

        // 创建 TableOfContent
        let toc = general_runtime.block_on(async {
            TableOfContent::new(
                &storage_config,
                search_runtime,
                update_runtime,
                general_runtime.handle().clone(),
                Default::default(),
                Default::default(),
                0,
                None,
            )
        });

        let toc = Arc::new(toc);
        let dispatcher = Arc::new(Dispatcher::new(toc.clone()));

        Ok(Self {
            toc,
            dispatcher,
            _search_runtime: search_runtime,
            _update_runtime: update_runtime,
            _general_runtime: general_runtime,
        })
    }

    /// 获取 TableOfContent 引用（用于高级用法）
    pub fn toc(&self) -> &Arc<TableOfContent> {
        &self.toc
    }

    /// 获取 Dispatcher 引用（用于高级用法）
    pub fn dispatcher(&self) -> &Arc<Dispatcher> {
        &self.dispatcher
    }
}
```

#### 2.4 集合操作 (`operations.rs`)

```rust
use std::sync::Arc;

use storage::content_manager::collection_meta_ops::CreateCollection;
use storage::content_manager::toc::TableOfContent;
use storage::dispatcher::Dispatcher;
use storage::rbac::Access;

use crate::embedded::client::QdrantEmbedded;
use crate::embedded::error::Result;
use crate::rest::models::{
    CollectionConfig, CreateCollection as CreateCollectionRequest,
    DeleteCollection, ListCollectionsResponse,
    VectorParams, Distance, PointStruct, PointId,
    SearchRequest, SearchResponse, QueryRequest, QueryResponse,
    ScrollRequest, ScrollResponse, UpdateResult,
    UpsertPoints, DeletePoints, UpdateBatch,
};

impl QdrantEmbedded {
    /// 列出所有集合
    pub async fn list_collections(&self) -> Result<ListCollectionsResponse> {
        let access = Access::full("list_collections");
        let response = self.toc.all_collections(&access).await;
        Ok(ListCollectionsResponse { collections: response })
    }

    /// 获取集合信息
    pub async fn get_collection(&self, collection_name: &str) -> Result<CollectionConfig> {
        let access = Access::full("get_collection");
        let collection = self.toc.get_collection_by_name(collection_name, &access).await?;
        Ok(collection.config().clone())
    }

    /// 创建集合
    pub async fn create_collection(
        &self,
        request: CreateCollectionRequest,
    ) -> Result<()> {
        let operation = CreateCollection::from(request);
        self.dispatcher
            .submit_collection_meta_op(
                storage::content_manager::collection_meta_ops::CollectionMetaOperations::CreateCollection(
                    Box::new(operation),
                ),
                Access::full("create_collection"),
                None,
            )
            .await?;
        Ok(())
    }

    /// 删除集合
    pub async fn delete_collection(&self, request: DeleteCollection) -> Result<()> {
        self.dispatcher
            .submit_collection_meta_op(
                storage::content_manager::collection_meta_ops::CollectionMetaOperations::DeleteCollection(
                    Box::new(request),
                ),
                Access::full("delete_collection"),
                None,
            )
            .await?;
        Ok(())
    }

    /// 插入或更新点
    pub async fn upsert_points(
        &self,
        collection_name: &str,
        points: Vec<PointStruct>,
    ) -> Result<UpdateResult> {
        let request = UpsertPoints {
            collection_name: collection_name.to_string(),
            points,
            ..Default::default()
        };
        self.toc.update(&request, &Access::full("upsert_points")).await?;
        Ok(UpdateResult { operation_id: 0, status: "completed".to_string() })
    }

    /// 删除点
    pub async fn delete_points(
        &self,
        collection_name: &str,
        points: PointId,
    ) -> Result<UpdateResult> {
        let request = DeletePoints {
            collection_name: collection_name.to_string(),
            points: storage::content_manager::toc::point_ops::PointIdsList::from(points),
            ..Default::default()
        };
        self.toc.update(&request, &Access::full("delete_points")).await?;
        Ok(UpdateResult { operation_id: 0, status: "completed".to_string() })
    }

    /// 搜索点
    pub async fn search_points(
        &self,
        collection_name: &str,
        request: SearchRequest,
    ) -> Result<SearchResponse> {
        let request = storage::types::ReadParams::from(request);
        let response = self.toc.search(collection_name, &request, &Access::full("search")).await?;
        Ok(response)
    }

    /// 查询点
    pub async fn query_points(
        &self,
        collection_name: &str,
        request: QueryRequest,
    ) -> Result<QueryResponse> {
        let request = storage::types::ReadParams::from(request);
        let response = self.toc.query(collection_name, &request, &Access::full("query")).await?;
        Ok(response)
    }

    /// 滚动浏览点
    pub async fn scroll_points(
        &self,
        collection_name: &str,
        request: ScrollRequest,
    ) -> Result<ScrollResponse> {
        let request = storage::types::ReadParams::from(request);
        let response = self.toc.scroll(collection_name, &request, &Access::full("scroll")).await?;
        Ok(response)
    }

    /// 批量更新
    pub async fn update_batch(
        &self,
        collection_name: &str,
        operations: Vec<UpdateBatch>,
    ) -> Result<UpdateResult> {
        let operations: Vec<collection::operations::types::UpdateOperation> =
            operations.into_iter().map(|op| op.into()).collect();
        self.toc.update_batch(&operations, &Access::full("update_batch")).await?;
        Ok(UpdateResult { operation_id: 0, status: "completed".to_string() })
    }
}
```

#### 2.5 模块入口 (`mod.rs`)

```rust
pub mod client;
pub mod config;
pub mod error;
pub mod operations;
pub mod conversions;

pub use client::QdrantEmbedded;
pub use config::{EmbeddedConfig, EmbeddedConfigBuilder};
pub use error::{EmbeddedError, Result};
```

### 阶段 3: 集成到主项目

#### 3.1 修改主项目 `Cargo.toml`

```toml
[package]
name = "qdrant"
version = "1.16.2"

[features]
default = ["server"]
server = ["api/server"]
embedded = ["api/embedded"]
# ... 其他 features
```

#### 3.2 更新工作空间成员

```toml
[workspace]
members = [
    "lib/api",
    "lib/collection",
    "lib/common/*",
    "lib/edge",
    "lib/edge/python",
    "lib/gridstore",
    "lib/macros",
    "lib/posting_list",
    "lib/segment",
    "lib/shard",
    "lib/sparse",
    "lib/storage",
]
```

### 阶段 4: 文档和示例

#### 4.1 创建示例 `examples/embedded.rs`

```rust
use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig, VectorParams, Distance, PointStruct, PointId, SearchRequest};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = EmbeddedConfig::builder()
        .storage_path("./qdrant_data")
        .search_threads(4)
        .optimizer_threads(2)
        .build();

    // 初始化嵌入库
    let qdrant = QdrantEmbedded::new(config)?;

    // 创建集合
    let collection_config = qdrant::rest::models::CreateCollection {
        collection_name: "my_collection".to_string(),
        vectors_config: qdrant::rest::models::VectorsConfig {
            config: qdrant::rest::models::VectorParams {
                size: 128,
                distance: Distance::Cosine,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };

    qdrant.create_collection(collection_config).await?;

    // 插入向量
    let points = vec![
        PointStruct {
            id: PointId::Num(1),
            vector: vec![0.1; 128],
            payload: None,
        },
        PointStruct {
            id: PointId::Num(2),
            vector: vec![0.2; 128],
            payload: None,
        },
    ];

    qdrant.upsert_points("my_collection", points).await?;

    // 搜索
    let search_request = SearchRequest {
        vector: vec![0.15; 128],
        limit: 10,
        ..Default::default()
    };

    let results = qdrant.search_points("my_collection", search_request).await?;

    println!("Search results: {:?}", results);

    Ok(())
}
```

#### 4.2 添加 API 文档注释

```rust
/// Qdrant 嵌入库客户端
///
/// 提供与 Qdrant 服务端相同的功能，但作为库直接集成到应用程序中。
/// 适用于需要高性能、低延迟的场景，或不想运行独立服务进程的应用。
///
/// # 示例
///
/// ```no_run
/// use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig};
///
/// let config = EmbeddedConfig::builder()
///     .storage_path("./data")
///     .build();
///
/// let qdrant = QdrantEmbedded::new(config)?;
/// # Ok::<(), qdrant_embedded::EmbeddedError>(())
/// ```
///
/// # 线程安全
///
/// `QdrantEmbedded` 实例是线程安全的，可以在多个线程间共享。
pub struct QdrantEmbedded { ... }
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_list_collections() {
        let temp = tempfile::tempdir().unwrap();
        let config = EmbeddedConfig::builder()
            .storage_path(temp.path())
            .build();

        let qdrant = QdrantEmbedded::new(config).unwrap();

        let create_request = CreateCollection {
            collection_name: "test".to_string(),
            vectors_config: VectorsConfig {
                config: VectorParams {
                    size: 10,
                    distance: Distance::Cosine,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };

        qdrant.create_collection(create_request).await.unwrap();

        let collections = qdrant.list_collections().await.unwrap();
        assert_eq!(collections.collections.len(), 1);
        assert_eq!(collections.collections[0].name, "test");
    }

    #[tokio::test]
    async fn test_upsert_and_search() {
        let temp = tempfile::tempdir().unwrap();
        let config = EmbeddedConfig::builder()
            .storage_path(temp.path())
            .build();

        let qdrant = QdrantEmbedded::new(config).unwrap();

        // Create collection
        let create_request = CreateCollection {
            collection_name: "test".to_string(),
            vectors_config: VectorsConfig {
                config: VectorParams {
                    size: 2,
                    distance: Distance::Cosine,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        };
        qdrant.create_collection(create_request).await.unwrap();

        // Upsert points
        let points = vec![
            PointStruct {
                id: PointId::Num(1),
                vector: vec![1.0, 0.0],
                payload: None,
            },
            PointStruct {
                id: PointId::Num(2),
                vector: vec![0.0, 1.0],
                payload: None,
            },
        ];
        qdrant.upsert_points("test", points).await.unwrap();

        // Search
        let search_request = SearchRequest {
            vector: vec![1.0, 0.0],
            limit: 2,
            ..Default::default()
        };
        let results = qdrant.search_points("test", search_request).await.unwrap();

        assert_eq!(results.result.len(), 2);
        assert_eq!(results.result[0].id, PointId::Num(1));
    }
}
```

### 集成测试

创建 `tests/embedded_integration_test.rs`:

```rust
use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig};
use qdrant::rest::models::*;

#[tokio::test]
async fn test_full_workflow() {
    let temp = tempfile::tempdir().unwrap();
    let config = EmbeddedConfig::builder()
        .storage_path(temp.path())
        .build();

    let qdrant = QdrantEmbedded::new(config).unwrap();

    // 1. Create collection
    let create_request = CreateCollection {
        collection_name: "products".to_string(),
        vectors_config: VectorsConfig {
            config: VectorParams {
                size: 4,
                distance: Distance::Cosine,
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    qdrant.create_collection(create_request).await.unwrap();

    // 2. Insert products
    let products = vec![
        PointStruct {
            id: PointId::Num(1),
            vector: vec![0.05, 0.61, 0.76, 0.74],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("soap".to_string()))]
                        .into_iter()
                        .collect()
                })
            }),
        },
        PointStruct {
            id: PointId::Num(2),
            vector: vec![0.19, 0.81, 0.75, 0.11],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("shampoo".to_string()))]
                        .into_iter()
                        .collect()
                })
            }),
        },
        PointStruct {
            id: PointId::Num(3),
            vector: vec![0.36, 0.55, 0.47, 0.94],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("shampoo".to_string()))]
                        .into_iter()
                        .collect()
                })
            }),
        },
        PointStruct {
            id: PointId::Num(4),
            vector: vec![0.18, 0.01, 0.85, 0.80],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("shampoo".to_string()))]
                        .into_iter()
().collect()
                })
            }),
        },
        PointStruct {
            id: PointId::Num(5),
            vector: vec![0.24, 0.18, 0.22, 0.44],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("soap".to_string()))]
                        .into_iter()
                        .collect()
                })
            }),
        },
        PointStruct {
            id: PointId::Num(6),
            vector: vec![0.35, 0.08, 0.11, 0.44],
            payload: Some(Payload {
                payload: Some(PayloadVariant::OneOf {
                    one_of: vec![("product".to_string(), Payload::String("soap".to_string()))]
                        .into_iter()
                        .collect()
                })
            }),
        },
    ];
    qdrant.upsert_points("products", products).await.unwrap();

    // 3. Search
    let search_request = SearchRequest {
        vector: vec![0.15, 0.55, 0.11, 0.39],
        limit: 2,
        with_payload: Some(WithPayloadInterface::Bool(true)),
        ..Default::default()
    };
    let results = qdrant.search_points("products", search_request).await.unwrap();

    assert_eq!(results.result.len(), 2);

    // 4. Filtered search
    let filter = Filter {
        must: Some(vec![Condition {
            condition_one_of: Some(ConditionOneOf::Field(FieldCondition {
                key: "product".to_string(),
                match: Some(Match::Value("soap".to_string())),
            })),
        })]
    };
    let filtered_search_request = SearchRequest {
        vector: vec![0.15, 0.55, 0.11, 0.39],
        limit: 2,
        filter: Some(filter),
        with_payload: Some(WithPayloadInterface::Bool(true)),
        ..Default::default()
    };
    let filtered_results = qdrant.search_points("products", filtered_search_request).await.unwrap();

    assert_eq!(filtered_results.result.len(), 2);
    assert!(matches!(
        filtered_results.result[0].payload.as_ref().unwrap().payload.as_ref().unwrap(),
        PayloadVariant::OneOf { .. }
    ));
}
```

## 性能考虑

### 内存管理

- **Tokio Runtime**: 嵌入库创建独立的 Tokio runtime，需要合理配置线程数
- **内存映射**: Segment 使用内存映射文件，确保有足够的虚拟内存
- **缓存策略**: 可配置向量缓存和 payload 索引缓存

### 并发控制

- **搜索并发**: 通过 `search_threads` 控制并发搜索任务数
- **更新并发**: 通过 `optimizer_threads` 控制后台优化任务数
- **IO 限制**: 通过 `io_limit` 控制并发 IO 操作数

### 持久化

- **WAL (Write-Ahead Log)**: 所有更新操作先写入 WAL，确保数据持久性
- **Checkpoint**: 定期创建 checkpoint，减少 WAL 大小
- **快照**: 支持创建和恢复集合快照

## 与现有方案的对比

| 特性 | 服务端模式 | Edge 模式 | 嵌入库模式 |
|------|-----------|-----------|-----------|
| **网络接口** | HTTP/gRPC | 无 | 无 |
| **多集合** | ✅ | ❌ | ✅ |
| **别名** | ✅ | ❌ | ✅ |
| **快照** | ✅ | ❌ | ✅ |
| **分布式** | ✅ | ❌ | ❌ |
| **RBAC** | ✅ | ❌ | ✅ |
| **配置** | YAML | Rust API | Rust API |
| **依赖** | 完整 | 最小 | 中等 |
| **性能** | 中等（网络开销） | 最高 | 高 |
| **适用场景** | 多客户端、分布式 | 单集合、嵌入式 | 单机、多集合 |

## 未来扩展

### 1. Python 绑定

```python
from qdrant_embedded import QdrantEmbedded

qdrant = QdrantEmbedded(storage_path="./data")
qdrant.create_collection("test", vectors_config={
    "size": 128,
    "distance": "Cosine"
})
qdrant.upsert("test", points=[
    {"id": 1, "vector": [0.1] * 128}
])
results = qdrant.search("test", query_vector=[0.1] * 128, limit=10)
```

### 2. C/C++ 绑定

使用 `cbindgen` 或 `cxx` 生成 FFI 接口。

### 3. Go 绑定

使用 `cgo` 或直接 CGO 绑定。

### 4. WebAssembly

编译到 WebAssembly，在浏览器中运行。

## 总结

通过扩展 `api` crate 为双模式库，我们可以：

1. **最大化代码复用**: 共享数据模型、转换逻辑和核心功能
2. **保持功能对等**: 嵌入库提供与客户端 API 相同的功能
3. **灵活部署**: 支持服务端和嵌入库两种部署模式
4. **易于维护**: 统一的代码库，减少维护成本

这种设计使 Qdrant 能够适应更广泛的使用场景，从传统的服务端部署到嵌入式应用，满足不同用户的需求。
