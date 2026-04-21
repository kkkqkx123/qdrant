# 嵌入库 API 快速参考

本文档提供 Qdrant 嵌入库的快速 API 参考。

## 初始化

### 创建客户端

```rust
use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig};

// 基本初始化
let config = EmbeddedConfig::builder()
    .storage_path("./data")
    .build();
let client = QdrantEmbedded::new(config)?;

// 自定义配置
let config = EmbeddedConfig::builder()
    .storage_path("./data")
    .snapshots_path("./snapshots")
    .temp_path("./temp")
    .search_threads(8)
    .optimizer_threads(4)
    .io_limit(1000)
    .cpu_limit(4)
    .build();
let client = QdrantEmbedded::new(config)?;

// 高级配置
use collection::optimizers_builder::OptimizersConfig;
use collection::config::WalConfig;
use segment::types::HnswConfig;

let config = EmbeddedConfig::builder()
    .storage_path("./data")
    // 性能配置
    .search_threads(8)
    .optimizer_threads(4)
    .update_rate_limit(1000)
    .search_timeout_sec(30)
    .async_scorer(true)
    // 存储配置
    .on_disk_payload(true)
    // 高级配置
    .optimizers_config(OptimizersConfig::default())
    .wal_config(WalConfig::default())
    .hnsw_config(HnswConfig::default())
    // 限制配置
    .max_collections(100)
    .update_queue_size(1000)
    .build();
let client = QdrantEmbedded::new(config)?;
```

---

## 集合管理

### 列出集合

```rust
let collections = client.list_collections().await?;
for name in collections {
    println!("Collection: {}", name);
}
```

### 创建集合

```rust
use collection::operations::types::VectorsConfig;
use segment::types::VectorParams;

let vectors_config = VectorsConfig::Single(VectorParams {
    size: 128,
    distance: Distance::Cosine,
    ..Default::default()
});

client.create_collection("my_collection".to_string(), vectors_config).await?;
```

### 获取集合信息

```rust
let info = client.get_collection_info("my_collection").await?;
println!("Points count: {:?}", info.points_count);
println!("Status: {:?}", info.status);
```

### 更新集合配置

```rust
use storage::content_manager::collection_meta_ops::UpdateCollection;

let update = UpdateCollection {
    vectors: None,
    hnsw_config: Some(HnswConfigDiff {
        m: Some(32),
        ef_construct: Some(200),
        ..Default::default()
    }),
    ..Default::default()
};

client.update_collection("my_collection".to_string(), update).await?;
```

### 检查集合是否存在

```rust
let exists = client.collection_exists("my_collection").await?;
if exists {
    println!("Collection exists");
}
```

### 删除集合

```rust
client.delete_collection("my_collection".to_string()).await?;
```

### 别名管理

```rust
use storage::content_manager::collection_meta_ops::{
    ChangeAliasesOperation, AliasOperations, CreateAlias, DeleteAlias,
};

// 列出所有别名
let aliases = client.list_aliases().await?;
for alias in aliases {
    println!("Alias: {} -> Collection: {}", alias.alias_name, alias.collection_name);
}

// 列出集合的别名
let collection_aliases = client.list_collection_aliases("my_collection").await?;
for alias in collection_aliases {
    println!("Alias: {}", alias);
}

// 创建别名
let operations = ChangeAliasesOperation {
    actions: vec![
        AliasOperations::from(CreateAlias {
            collection_name: "my_collection".to_string(),
            alias_name: "my_alias".to_string(),
        }),
    ],
};
client.update_aliases(operations).await?;

// 删除别名
let operations = ChangeAliasesOperation {
    actions: vec![
        AliasOperations::from(DeleteAlias {
            alias_name: "my_alias".to_string(),
        }),
    ],
};
client.update_aliases(operations).await?;
```

---

## 点操作

### 插入点

```rust
use shard::operations::point_ops::PointOperations;
use segment::types::PointIdType;

let points = vec![
    PointStruct {
        id: PointIdType::NumId(1),
        vector: VectorStruct::Single(vec![0.1, 0.2, 0.3, /* ... */]),
        payload: Some(payload!({"field" => "value"})),
    },
    // ... 更多点
];

let operation = PointOperations::UpsertPoints {
    points,
    ..Default::default()
};

client.upsert_points("my_collection".to_string(), operation).await?;
```

### 获取点

```rust
use collection::operations::types::PointRequestInternal;

let request = PointRequestInternal {
    ids: vec![1.into(), 2.into(), 3.into()],
    with_payload: Some(WithPayloadInterface::Bool(true)),
    with_vector: WithVector::Bool(true),
};

let points = client.get_points("my_collection", request, None, None).await?;
for point in points {
    println!("Point ID: {:?}", point.id);
}
```

### 删除点

```rust
use shard::operations::point_ops::PointOperations;

let operation = PointOperations::DeletePoints {
    points: PointsSelector::PointIdsSelector(PointIdsList {
        points: vec![1.into(), 2.into()],
    }),
};

client.delete_points("my_collection".to_string(), operation).await?;
```

### 计数点

```rust
use collection::operations::types::CountRequestInternal;

let request = CountRequestInternal {
    filter: None,  // 可选：添加过滤条件
    exact: true,
};

let result = client.count_points("my_collection", request, None, None).await?;
println!("Count: {}", result.count);
```

---

## 搜索功能

### 向量搜索

```rust
use shard::search::CoreSearchRequestBatch;

let query_vector = vec![0.1, 0.2, 0.3, /* ... */];
let request = CoreSearchRequestBatch {
    searches: vec![CoreSearchRequest {
        vector: NamedVectorStruct::Default(query_vector),
        filter: None,
        params: None,
        limit: 10,
        offset: None,
        with_payload: Some(WithPayloadInterface::Bool(true)),
        with_vector: Some(WithVector::Bool(true)),
        score_threshold: None,
    }],
};

let results = client.search_points("my_collection", request, None, None).await?;
for batch in results {
    for point in batch {
        println!("ID: {:?}, Score: {}", point.id, point.score);
    }
}
```

### 查询点

```rust
use collection::operations::universal_query::collection_query::CollectionQueryRequest;

let request = CollectionQueryRequest {
    query: Some(QueryInterface::Nearest(VectorInput::DenseVector(query_vector))),
    limit: Some(10),
    with_payload: Some(WithPayloadInterface::Bool(true)),
    with_vector: Some(WithVector::Bool(true)),
    ..Default::default()
};

let results = client.query_points(
    "my_collection",
    vec![(request, ShardSelectorInternal::All)],
    None,
    None,
).await?;
```

### 推荐查询

```rust
use collection::operations::types::RecommendRequestInternal;

let request = RecommendRequestInternal {
    positive: vec![1.into(), 2.into()],  // 正例点ID
    negative: vec![3.into()],             // 负例点ID
    strategy: Some(RecommendStrategy::BestScore),
    limit: Some(10),
    with_payload: Some(WithPayloadInterface::Bool(true)),
    with_vector: Some(WithVector::Bool(true)),
    ..Default::default()
};

let results = client.recommend_points("my_collection", request, None, None).await?;
```

### 发现查询

```rust
use collection::operations::types::DiscoverRequestInternal;

let request = DiscoverRequestInternal {
    target: VectorInput::PointId(1.into()),  // 目标点
    context: Some(vec![
        ContextPair {
            positive: VectorInput::PointId(2.into()),
            negative: VectorInput::PointId(3.into()),
        },
    ]),
    limit: Some(10),
    with_payload: Some(WithPayloadInterface::Bool(true)),
    with_vector: Some(WithVector::Bool(true)),
    ..Default::default()
};

let results = client.discover_points("my_collection", request, None, None).await?;
```

---

## Payload 操作

### 设置 Payload

```rust
use shard::operations::payload_ops::SetPayload;

let payload = SetPayload {
    payload: payload!({"category" => "electronics", "price" => 99.99}),
    points: Some(PointsSelector::PointIdsSelector(PointIdsList {
        points: vec![1.into(), 2.into()],
    })),
    filter: None,
    key: None,
};

client.set_payload("my_collection".to_string(), payload).await?;
```

### 删除 Payload

```rust
use shard::operations::payload_ops::PayloadOps;

let operation = PayloadOps::DeletePayload(DeletePayload {
    keys: vec!["old_field".into()],
    points: Some(PointsSelector::PointIdsSelector(PointIdsList {
        points: vec![1.into()],
    })),
    filter: None,
});

client.delete_payload("my_collection".to_string(), operation).await?;
```

### 清除 Payload

```rust
let operation = PayloadOps::ClearPayload {
    points: PointsSelector::PointIdsSelector(PointIdsList {
        points: vec![1.into()],
    }),
};

client.clear_payload("my_collection".to_string(), operation).await?;
```

---

## 向量操作

### 更新向量

```rust
use shard::operations::vector_ops::VectorOperations;

let operation = VectorOperations::UpdateVectors {
    points: vec![PointVectors {
        id: 1.into(),
        vector: VectorStruct::Single(vec![0.4, 0.5, 0.6, /* ... */]),
    }],
};

client.update_vectors("my_collection".to_string(), operation).await?;
```

### 删除向量

```rust
let operation = VectorOperations::DeleteVectors {
    points: PointsSelector::PointIdsSelector(PointIdsList {
        points: vec![1.into()],
    }),
    vectors: Some(vec!["image_vector".into()]),
};

client.delete_vectors("my_collection".to_string(), operation).await?;
```

---

## 高级功能

### 滚动浏览

```rust
use collection::operations::types::ScrollRequestInternal;

let request = ScrollRequestInternal {
    offset: None,
    limit: Some(100),
    with_payload: Some(WithPayloadInterface::Bool(true)),
    with_vector: Some(WithVector::Bool(true)),
    filter: None,
    order_by: None,
};

let result = client.scroll_points("my_collection", request, None, None).await?;
println!("Points: {:?}", result.points);
if let Some(next_offset) = result.next_page_offset {
    println!("Has more data, next offset: {:?}", next_offset);
}
```

### 分组查询

```rust
use collection::grouping::group_by::GroupRequest;

let request = GroupRequest {
    group_by: "category".into(),
    group_size: 3,
    limit: 10,
    with_lookup: None,
    // ... 搜索参数
};

let groups = client.group_points("my_collection", request, None, None).await?;
for group in groups.groups {
    println!("Group: {:?}", group.id);
    for hit in group.hits {
        println!("  Point: {:?}, Score: {}", hit.id, hit.score);
    }
}
```

### Facet 聚合

```rust
use segment::data_types::facets::FacetParams;

let request = FacetParams {
    key: "category".into(),
    limit: Some(10),
    filter: None,
    exact: Some(true),
};

let response = client.facet("my_collection", request, None, None).await?;
for hit in response.hits {
    println!("Value: {:?}, Count: {}", hit.value, hit.count);
}
```

### 距离矩阵

```rust
use collection::collection::distance_matrix::CollectionSearchMatrixRequest;

let request = CollectionSearchMatrixRequest {
    filter: None,
    sample: Some(100),
    limit: Some(5),
    using: None,
};

let matrix = client.search_points_matrix("my_collection", request, None, None).await?;
println!("Distance matrix: {:?}", matrix);
```

### 快照操作

```rust
use collection::operations::snapshot_ops::SnapshotRecover;
use url::Url;

// 创建快照
let snapshot = client.create_snapshot("my_collection").await?;
println!("Snapshot created: {}", snapshot.name);

// 列出快照
let snapshots = client.list_snapshots("my_collection").await?;
for snapshot in snapshots {
    println!("Snapshot: {}, Size: {}", snapshot.name, snapshot.size);
}

// 删除快照
let deleted = client.delete_snapshot("my_collection", &snapshot.name).await?;
if deleted {
    println!("Snapshot deleted successfully");
}

// 从快照恢复
let recover = SnapshotRecover {
    location: Url::parse("file:///path/to/snapshot.snapshot")?,
    priority: None,
    checksum: None,
    api_key: None,
};
let recovered = client.recover_from_snapshot("my_collection", recover).await?;
if recovered {
    println!("Collection recovered from snapshot");
}
```

---

## 过滤器

### 基本过滤

```rust
use segment::types::Filter;

let filter = Filter::must(Condition::Field(FieldCondition {
    key: "category".into(),
    condition: field_condition::Condition::Match(Match::Value(MatchValue {
        value: "electronics".into(),
    })),
}));

// 在搜索中使用
let request = CoreSearchRequest {
    vector: NamedVectorStruct::Default(query_vector),
    filter: Some(filter),
    // ...
};
```

### 复合过滤

```rust
let filter = Filter::must_all(vec![
    Condition::Field(FieldCondition {
        key: "category".into(),
        condition: field_condition::Condition::Match(Match::Value(MatchValue {
            value: "electronics".into(),
        })),
    }),
    Condition::Field(FieldCondition {
        key: "price".into(),
        condition: field_condition::Condition::Range(Range {
            lt: Some(100.0),
            gte: Some(10.0),
            ..Default::default()
        }),
    }),
]);
```

---

## 一致性和超时

### 读取一致性

```rust
use collection::operations::consistency_params::ReadConsistency;

// 强一致性
let consistency = Some(ReadConsistency::Strong);

// 最终一致性
let consistency = Some(ReadConsistency::Eventual);

// 可仲裁一致性
let consistency = Some(ReadConsistency::Quorum);

let results = client.search_points(
    "my_collection",
    request,
    consistency,
    None,
).await?;
```

### 超时设置

```rust
use std::time::Duration;

let timeout = Some(Duration::from_secs(5));

let results = client.search_points(
    "my_collection",
    request,
    None,
    timeout,
).await?;
```

---

## 错误处理

### 基本错误处理

```rust
use qdrant_embedded::EmbeddedError;

match client.get_collection_info("my_collection").await {
    Ok(info) => println!("Info: {:?}", info),
    Err(EmbeddedError::CollectionNotFound { name }) => {
        println!("Collection {} not found", name);
    }
    Err(e) => println!("Error: {}", e),
}
```

---

## 性能提示

### 批量操作

```rust
// 批量插入比单次插入快得多
let points: Vec<PointStruct> = (0..1000)
    .map(|i| PointStruct {
        id: PointIdType::NumId(i),
        vector: VectorStruct::Single(generate_vector()),
        payload: None,
    })
    .collect();

let operation = PointOperations::UpsertPoints { points, ..Default::default() };
client.upsert_points("my_collection".to_string(), operation).await?;
```

### 异步并发

```rust
// 并发执行多个搜索
let futures: Vec<_> = queries
    .into_iter()
    .map(|query| client.search_points("my_collection", query, None, None))
    .collect();

let results = futures::future::try_join_all(futures).await?;
```

---

## 完整示例

```rust
use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig};
use collection::operations::types::{VectorsConfig, VectorParams};
use segment::types::{Distance, PointIdType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化
    let config = EmbeddedConfig::builder()
        .storage_path("./data")
        .search_threads(4)
        .build();
    let client = QdrantEmbedded::new(config)?;
    
    // 创建集合
    let vectors_config = VectorsConfig::Single(VectorParams {
        size: 128,
        distance: Distance::Cosine,
        ..Default::default()
    });
    client.create_collection("docs".to_string(), vectors_config).await?;
    
    // 插入点
    let points = vec![
        PointStruct {
            id: PointIdType::NumId(1),
            vector: VectorStruct::Single(vec![0.1; 128]),
            payload: Some(payload!({"title" => "Document 1"})),
        },
        PointStruct {
            id: PointIdType::NumId(2),
            vector: VectorStruct::Single(vec![0.2; 128]),
            payload: Some(payload!({"title" => "Document 2"})),
        },
    ];
    
    let operation = PointOperations::UpsertPoints { points, ..Default::default() };
    client.upsert_points("docs".to_string(), operation).await?;
    
    // 搜索
    let query = vec![0.15; 128];
    let request = CoreSearchRequestBatch {
        searches: vec![CoreSearchRequest {
            vector: NamedVectorStruct::Default(query),
            limit: 10,
            with_payload: Some(WithPayloadInterface::Bool(true)),
            with_vector: Some(WithVector::Bool(false)),
            ..Default::default()
        }],
    };
    
    let results = client.search_points("docs", request, None, None).await?;
    for batch in results {
        for point in batch {
            println!("Found: {:?}, Score: {}", point.id, point.score);
        }
    }
    
    // 清理
    client.delete_collection("docs".to_string()).await?;
    
    Ok(())
}
```

---

## 参考文档

- [功能对比详细文档](./feature-comparison.md)
- [缺失功能实现方案](./missing-features-implementation.md)
- [嵌入库分析报告](./embedded-library-analysis.md)
