# 嵌入库配置参考

本文档详细说明 Qdrant 嵌入库的所有配置选项。

## 配置结构

### EmbeddedConfig

```rust
pub struct EmbeddedConfig {
    // 基础配置
    pub storage_path: PathBuf,
    pub snapshots_path: Option<PathBuf>,
    pub temp_path: Option<PathBuf>,
    
    // 性能配置
    pub search_threads: Option<usize>,
    pub optimizer_threads: Option<usize>,
    pub io_limit: Option<usize>,
    pub cpu_limit: Option<usize>,
    pub update_rate_limit: Option<usize>,
    pub search_timeout_sec: Option<usize>,
    pub async_scorer: Option<bool>,
    
    // 存储配置
    pub on_disk_payload: Option<bool>,
    
    // 高级配置
    pub optimizers_config: Option<OptimizersConfig>,
    pub wal_config: Option<WalConfig>,
    pub hnsw_config: Option<HnswConfig>,
    pub collection_defaults: Option<CollectionConfigDefaults>,
    
    // 限制配置
    pub max_collections: Option<usize>,
    pub update_queue_size: Option<usize>,
}
```

---

## 基础配置

### storage_path

**类型**: `PathBuf`  
**必需**: 是  
**默认值**: `./qdrant_storage`

数据存储的根目录路径。所有集合数据、索引和元数据都将存储在此目录下。

```rust
let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    .build();
```

### snapshots_path

**类型**: `Option<PathBuf>`  
**默认值**: `{storage_path}/snapshots`

快照文件的存储路径。如果未指定,将在 storage_path 下创建 snapshots 子目录。

```rust
let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    .snapshots_path("/backup/qdrant-snapshots")
    .build();
```

### temp_path

**类型**: `Option<PathBuf>`  
**默认值**: 系统临时目录

临时文件的存储路径。用于优化过程中的中间文件和快照创建。

```rust
let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    .temp_path("/tmp/qdrant")
    .build();
```

---

## 性能配置

### search_threads

**类型**: `Option<usize>`  
**默认值**: `4`

搜索操作使用的线程数。增加此值可以提高并发搜索性能,但也会增加 CPU 使用。

**建议**:
- 小数据集: 2-4 线程
- 中等数据集: 4-8 线程
- 大数据集: 8-16 线程

```rust
let config = EmbeddedConfig::builder()
    .search_threads(8)
    .build();
```

### optimizer_threads

**类型**: `Option<usize>`  
**默认值**: `2`

索引优化操作使用的线程数。优化操作包括段合并、索引构建等。

**建议**:
- 写入密集: 2-4 线程
- 平衡场景: 4-8 线程
- 优化密集: 8-16 线程

```rust
let config = EmbeddedConfig::builder()
    .optimizer_threads(4)
    .build();
```

### io_limit

**类型**: `Option<usize>`  
**默认值**: `0` (自动)

IO 操作的并发限制。控制同时进行的磁盘 IO 操作数量。

**建议**:
- HDD: 1-2
- SSD: 4-8
- NVMe: 8-16

```rust
let config = EmbeddedConfig::builder()
    .io_limit(8)
    .build();
```

### cpu_limit

**类型**: `Option<usize>`  
**默认值**: `0` (自动)

优化操作可使用的 CPU 核心数限制。

**建议**:
- 保留 1-2 核心给其他操作
- 例如 8 核系统: 设置为 6-7

```rust
let config = EmbeddedConfig::builder()
    .cpu_limit(6)
    .build();
```

### update_rate_limit

**类型**: `Option<usize>`  
**默认值**: `None` (无限制)

每秒最大更新操作数。用于限制写入速率,防止过载。

```rust
let config = EmbeddedConfig::builder()
    .update_rate_limit(1000)  // 每秒最多 1000 次更新
    .build();
```

### search_timeout_sec

**类型**: `Option<usize>`  
**默认值**: `None` (无超时)

搜索操作的超时时间(秒)。超时后搜索将返回部分结果。

```rust
let config = EmbeddedConfig::builder()
    .search_timeout_sec(30)  // 30 秒超时
    .build();
```

### async_scorer

**类型**: `Option<bool>`  
**默认值**: `false`

是否使用异步评分器。启用可以提高搜索性能,但可能增加内存使用。

```rust
let config = EmbeddedConfig::builder()
    .async_scorer(true)
    .build();
```

---

## 存储配置

### on_disk_payload

**类型**: `Option<bool>`  
**默认值**: `false`

是否将 payload 存储在磁盘上而不是内存中。

**优点**:
- 减少内存使用
- 支持更大的 payload

**缺点**:
- 增加 IO 操作
- 降低检索速度

**建议**:
- 小 payload (< 1KB): 内存存储 (false)
- 大 payload (> 1KB): 磁盘存储 (true)
- 内存受限: 磁盘存储 (true)

```rust
let config = EmbeddedConfig::builder()
    .on_disk_payload(true)
    .build();
```

---

## 高级配置

### optimizers_config

**类型**: `Option<OptimizersConfig>`  
**默认值**: 默认优化器配置

索引优化器的详细配置。

```rust
use collection::optimizers_builder::OptimizersConfig;

let optimizers = OptimizersConfig {
    deleted_threshold: 0.2,           // 删除阈值
    vacuum_min_vector_number: 1000,   // 最小向量数
    default_segment_number: 5,        // 默认段数
    indexing_threshold: Some(50_000), // 索引阈值
    flush_interval_sec: 30,           // 刷新间隔
    ..Default::default()
};

let config = EmbeddedConfig::builder()
    .optimizers_config(optimizers)
    .build();
```

### wal_config

**类型**: `Option<WalConfig>`  
**默认值**: 默认 WAL 配置

Write-Ahead Log 配置,用于数据持久化和恢复。

```rust
use collection::config::WalConfig;

let wal = WalConfig {
    wal_capacity_mb: 64,    // WAL 容量
    wal_segments_ahead: 2,  // 预分配段数
};

let config = EmbeddedConfig::builder()
    .wal_config(wal)
    .build();
```

### hnsw_config

**类型**: `Option<HnswConfig>`  
**默认值**: 默认 HNSW 配置

HNSW 索引的配置参数。

```rust
use segment::types::HnswConfig;

let hnsw = HnswConfig {
    m: 32,              // 连接数
    ef_construct: 200,  // 构建时的 ef
    full_scan_threshold: 10000,  // 全扫描阈值
    ..Default::default()
};

let config = EmbeddedConfig::builder()
    .hnsw_config(hnsw)
    .build();
```

### collection_defaults

**类型**: `Option<CollectionConfigDefaults>`  
**默认值**: `None`

新集合的默认配置。

```rust
use segment::data_types::collection_defaults::CollectionConfigDefaults;
use segment::types::VectorsConfigDefaults;

let defaults = CollectionConfigDefaults {
    vectors: Some(VectorsConfigDefaults {
        size: 128,
        distance: Distance::Cosine,
    }),
    shard_number: Some(3),
    replication_factor: Some(2),
    ..Default::default()
};

let config = EmbeddedConfig::builder()
    .collection_defaults(defaults)
    .build();
```

---

## 限制配置

### max_collections

**类型**: `Option<usize>`  
**默认值**: `None` (无限制)

最大集合数量限制。用于防止创建过多集合。

```rust
let config = EmbeddedConfig::builder()
    .max_collections(100)  // 最多 100 个集合
    .build();
```

### update_queue_size

**类型**: `Option<usize>`  
**默认值**: `None` (自动)

更新操作队列的大小。控制待处理更新操作的数量。

```rust
let config = EmbeddedConfig::builder()
    .update_queue_size(1000)  // 队列大小 1000
    .build();
```

---

## 配置示例

### 基础配置

适用于开发和测试环境:

```rust
let config = EmbeddedConfig::builder()
    .storage_path("./data")
    .search_threads(4)
    .optimizer_threads(2)
    .build();
```

### 生产配置

适用于生产环境:

```rust
let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    .snapshots_path("/backup/qdrant-snapshots")
    .temp_path("/tmp/qdrant")
    // 性能配置
    .search_threads(8)
    .optimizer_threads(4)
    .io_limit(8)
    .cpu_limit(6)
    .update_rate_limit(1000)
    .search_timeout_sec(30)
    // 存储配置
    .on_disk_payload(false)
    // 限制配置
    .max_collections(100)
    .update_queue_size(1000)
    .build();
```

### 大数据集配置

适用于大规模数据集:

```rust
use segment::types::HnswConfig;

let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    // 性能配置
    .search_threads(16)
    .optimizer_threads(8)
    .io_limit(16)
    .cpu_limit(14)
    .async_scorer(true)
    // 存储配置
    .on_disk_payload(true)
    // HNSW 配置
    .hnsw_config(HnswConfig {
        m: 48,
        ef_construct: 400,
        ..Default::default()
    })
    .build();
```

### 内存受限配置

适用于内存受限环境:

```rust
let config = EmbeddedConfig::builder()
    .storage_path("/data/qdrant")
    // 性能配置
    .search_threads(2)
    .optimizer_threads(1)
    .io_limit(2)
    // 存储配置
    .on_disk_payload(true)  // 减少内存使用
    .build();
```

---

## 配置验证

### 常见问题

1. **线程数过多**
   ```rust
   // 错误: 线程数超过 CPU 核心数
   .search_threads(100)
   .optimizer_threads(100)
   ```

2. **路径不存在**
   ```rust
   // 错误: 存储路径不存在
   .storage_path("/nonexistent/path")
   ```

3. **配置冲突**
   ```rust
   // 警告: 同时限制 CPU 和 IO 可能降低性能
   .cpu_limit(2)
   .io_limit(2)
   ```

### 最佳实践

1. **根据硬件配置调整**
   - CPU 核心数: `search_threads + optimizer_threads < cpu_cores`
   - 内存大小: 大内存可禁用 `on_disk_payload`
   - 磁盘类型: SSD 可增加 `io_limit`

2. **根据使用场景调整**
   - 读密集: 增加 `search_threads`
   - 写密集: 增加 `optimizer_threads`
   - 混合场景: 平衡配置

3. **监控和调优**
   - 监控 CPU、内存、IO 使用
   - 根据监控结果调整配置
   - 进行性能测试验证

---

## 配置迁移

### 从配置文件加载

```rust
use std::fs;
use serde_json;

// 从 JSON 文件加载
let config_json = fs::read_to_string("config.json")?;
let config: EmbeddedConfig = serde_json::from_str(&config_json)?;
```

### 从环境变量加载

```rust
use std::env;

let config = EmbeddedConfig::builder()
    .storage_path(env::var("QDRANT_STORAGE_PATH").unwrap_or("./data".to_string()))
    .search_threads(env::var("QDRANT_SEARCH_THREADS")
        .ok()
        .and_then(|v| v.parse().ok()))
    .build();
```

---

## 总结

### 配置优先级

1. **必需配置**: `storage_path`
2. **推荐配置**: `search_threads`, `optimizer_threads`
3. **可选配置**: 其他所有配置项

### 配置建议

- **开发环境**: 使用默认配置或简单配置
- **测试环境**: 根据测试需求调整
- **生产环境**: 根据硬件和使用场景优化
- **特殊场景**: 使用高级配置微调

### 下一步

- 参考性能调优指南
- 进行配置测试
- 监控运行状态
