# Qdrant Embedded 库实现分析

## 当前状态

### 已完成的修复

1. **client.rs 修复**
   - 修复了类型不匹配错误 (E0308)
   - 修复了缺失 trait bound (E0277)
   - 修复了字段缺失错误 (E0061, E0599)
   - 修复了未导入类型错误 (E0432)
   - 正确配置了 `StorageConfig` 和 `PerformanceConfig`
   - 正确初始化了 `TableOfContent`

2. **operations.rs 部分修复**
   - 修复了 `VectorsConfig` 类型不匹配问题
   - 修复了 `update` 方法调用参数
   - 修复了 `HwMeasurementAcc::disposable()` 调用

### 待修复的问题

#### 1. Search/Query/Scroll 方法调用

**问题分析：**

当前代码尝试直接调用 `TableOfContent` 的 `search`、`query`、`scroll` 方法，但这些方法不存在或签名不匹配。

**正确的 API 路径：**

根据代码分析，正确的调用路径应该是：

```
TableOfContent::core_search_batch() -> Vec<Vec<ScoredPoint>>
TableOfContent::query_batch() -> Vec<Vec<ScoredPoint>>
TableOfContent::scroll() -> ScrollResult
```

**关键类型：**

- `CoreSearchRequestBatch` (来自 `shard::search`)
  - 包含 `Vec<CoreSearchRequest>`
  - `CoreSearchRequest` 包含 `QueryEnum`、`Filter`、`SearchParams` 等

- `CollectionQueryRequest` (来自 `collection::operations::query`)
  - 用于新的查询 API

- `ScrollRequestInternal` (来自 `collection::operations::scroll_by`)
  - 用于滚动浏览

#### 2. API 类型转换

**问题：**

`api::SearchRequest`、`api::QueryRequest`、`api::ScrollRequest` 是 gRPC/REST API 类型，需要转换为内部类型。

**转换路径：**

```
api::grpc::qdrant::SearchPoints -> CoreSearchRequestBatch
api::grpc::qdrant::QueryPoints -> CollectionQueryRequest
api::grpc::qdrant::ScrollPoints -> ScrollRequestInternal
```

**现有转换实现位置：**

- `lib/shard/src/search.rs`: `TryFrom<SearchRequestInternal> for CoreSearchRequest`
- `lib/collection/src/operations/conversions.rs`: 各种类型转换

## 实现方案

### 方案一：直接使用内部类型（推荐）

修改 `operations.rs`，使用内部类型而不是 API 类型：

```rust
// 使用内部类型
pub async fn search_points(
    &self,
    collection_name: &str,
    request: CoreSearchRequestBatch,
    read_consistency: Option<ReadConsistency>,
    timeout: Option<Duration>,
) -> Result<Vec<Vec<ScoredPoint>>> {
    self.toc.core_search_batch(
        collection_name,
        request,
        read_consistency,
        ShardSelectorInternal::All,
        Access::full("search"),
        timeout,
        HwMeasurementAcc::disposable(),
    ).await
}
```

**优点：**

- 直接使用底层 API，性能最优
- 不需要复杂的类型转换
- 与 Qdrant 内部实现一致

**缺点：**

- API 不如 gRPC/REST 友好
- 用户需要了解内部类型

### 方案二：实现 API 类型转换

为 `api::SearchRequest` 等类型实现转换：

```rust
impl TryFrom<api::SearchRequest> for CoreSearchRequestBatch {
    fn try_from(value: api::SearchRequest) -> Result<Self> {
        // 实现转换逻辑
    }
}
```

**优点：**

- 提供友好的 API
- 与 gRPC/REST API 一致

**缺点：**

- 需要实现大量转换代码
- 可能引入性能开销

### 方案三：提供 Builder 模式

提供 Builder 模式来构建请求：

```rust
let request = SearchRequestBuilder::new()
    .vector(vec![1.0, 2.0, 3.0])
    .limit(10)
    .filter(filter)
    .build();

let results = embedded.search_points("collection", request).await?;
```

## 关键依赖和类型

### 必需的导入

```rust
// Search 相关
use shard::search::{CoreSearchRequest, CoreSearchRequestBatch};
use shard::query::query_enum::QueryEnum;
use segment::types::{Filter, SearchParams, WithPayloadInterface, WithVector};

// Query 相关
use collection::operations::query::CollectionQueryRequest;

// Scroll 相关
use collection::operations::scroll_by::ScrollRequestInternal;

// 通用
use collection::operations::consistency_params::ReadConsistency;
use storage::content_manager::shards::shard_selector::ShardSelectorInternal;
use common::counter::hardware_accumulator::HwMeasurementAcc;
```

### TableOfContent 方法签名

```rust
// Search
pub async fn core_search_batch(
    &self,
    collection_name: &str,
    request: CoreSearchRequestBatch,
    read_consistency: Option<ReadConsistency>,
    shard_selection: ShardSelectorInternal,
    access: Access,
    timeout: Option<Duration>,
    hw_measurement_acc: HwMeasurementAcc,
) -> StorageResult<Vec<Vec<ScoredPoint>>>

// Query
pub async fn query_batch(
    &self,
    collection_name: &str,
    requests: Vec<(CollectionQueryRequest, ShardSelectorInternal)>,
    read_consistency: Option<ReadConsistency>,
    access: Access,
    timeout: Option<Duration>,
    hw_measurement_acc: HwMeasurementAcc,
) -> StorageResult<Vec<Vec<ScoredPoint>>>

// Scroll
pub async fn scroll(
    &self,
    collection_name: &str,
    request: ScrollRequestInternal,
    read_consistency: Option<ReadConsistency>,
    timeout: Option<Duration>,
    shard_selection: ShardSelectorInternal,
    access: Access,
    hw_measurement_acc: HwMeasurementAcc,
) -> StorageResult<ScrollResult>
```

## 下一步行动

1. **短期方案**：移除 `search_points`、`query_points`、`scroll_points` 方法，先让代码编译通过
2. **中期方案**：实现方案一，使用内部类型提供完整功能
3. **长期方案**：实现方案二或三，提供更友好的 API

## 参考文件

- `lib/storage/src/content_manager/toc/point_ops.rs`: TableOfContent 的操作方法
- `lib/shard/src/search.rs`: CoreSearchRequest 和 CoreSearchRequestBatch 定义
- `lib/collection/src/operations/query.rs`: CollectionQueryRequest 定义
- `lib/collection/src/operations/scroll_by.rs`: ScrollRequestInternal 定义
- `lib/collection/src/operations/conversions.rs`: 类型转换实现
