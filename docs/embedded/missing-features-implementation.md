# 嵌入库缺失功能实现方案

本文档记录了 Qdrant 嵌入库尚未实现的功能及其实现方案。

## 概述

当前嵌入库已实现约 50% 的 REST API 功能。本文档详细说明剩余功能的实现方案，包括：
- 别名管理
- 快照操作
- 集群管理
- 分片管理

---

## 1. 别名管理功能

### 1.1 功能描述

别名管理允许为集合创建别名，便于应用层切换集合而不修改代码。

### 1.2 REST API 端点

```
GET    /aliases                        # 列出所有别名
GET    /collections/{name}/aliases     # 列出集合的别名
POST   /collections/aliases            # 创建/更新/删除别名
```

### 1.3 实现方案

#### 1.3.1 列出所有别名

```rust
/// 列出所有别名
pub async fn list_aliases(&self) -> Result<Vec<AliasDescription>> {
    let access = Access::full("list_aliases");
    let aliases = self.toc.get_aliases(&access).await?;
    Ok(aliases)
}
```

#### 1.3.2 列出集合的别名

```rust
/// 列出集合的别名
pub async fn list_collection_aliases(
    &self,
    collection_name: &str,
) -> Result<Vec<String>> {
    let access = Access::full("list_collection_aliases");
    let collection_pass = access.check_collection_access(
        collection_name,
        AccessRequirements::new()
    )?;
    let aliases = self.toc.collection_aliases(&collection_pass, &access).await?;
    Ok(aliases)
}
```

#### 1.3.3 更新别名

```rust
/// 更新别名（创建、删除）
pub async fn update_aliases(
    &self,
    operations: ChangeAliasesOperation,
) -> Result<()> {
    self.dispatcher
        .submit_collection_meta_op(
            CollectionMetaOperations::ChangeAliases(operations),
            Access::full("update_aliases"),
            None,
        )
        .await?;
    Ok(())
}
```

### 1.4 所需导入

```rust
use storage::content_manager::collection_meta_ops::ChangeAliasesOperation;
use api::rest::models::AliasDescription;
```

### 1.5 实现难度

**难度：低**

别名管理功能主要涉及元数据操作，不涉及数据存储，实现简单。

---

## 2. 快照操作功能

### 2.1 功能描述

快照功能用于备份和恢复集合数据，支持完整快照和集合快照。

### 2.2 REST API 端点

```
POST   /collections/{name}/snapshots              # 创建集合快照
GET    /collections/{name}/snapshots              # 列出集合快照
DELETE /collections/{name}/snapshots/{snapshot}   # 删除集合快照
PUT    /collections/{name}/snapshots/upload       # 上传快照
GET    /collections/{name}/snapshots/{snapshot}   # 下载快照
PUT    /collections/{name}/snapshots/recover      # 从快照恢复

POST   /snapshots/upload                         # 上传完整快照
GET    /snapshots                                # 列出完整快照
DELETE /snapshots/{snapshot}                     # 删除完整快照
GET    /snapshots/{snapshot}                     # 下载完整快照
PUT    /snapshots/recover                        # 从完整快照恢复
```

### 2.3 实现方案

#### 2.3.1 创建集合快照

```rust
/// 创建集合快照
pub async fn create_snapshot(
    &self,
    collection_name: &str,
    wait: bool,
) -> Result<SnapshotDescription> {
    let access = Access::full("create_snapshot");
    let collection_pass = access.check_collection_access(
        collection_name,
        AccessRequirements::new().manage()
    )?;
    
    let snapshot_name = self.toc
        .create_snapshot(&collection_pass, wait)
        .await?;
    
    Ok(snapshot_name)
}
```

#### 2.3.2 列出集合快照

```rust
/// 列出集合快照
pub async fn list_snapshots(
    &self,
    collection_name: &str,
) -> Result<Vec<SnapshotDescription>> {
    let access = Access::full("list_snapshots");
    let collection_pass = access.check_collection_access(
        collection_name,
        AccessRequirements::new()
    )?;
    
    let snapshots = self.toc
        .list_snapshots(&collection_pass)
        .await?;
    
    Ok(snapshots)
}
```

#### 2.3.3 删除集合快照

```rust
/// 删除集合快照
pub async fn delete_snapshot(
    &self,
    collection_name: &str,
    snapshot_name: &str,
    wait: bool,
) -> Result<()> {
    let access = Access::full("delete_snapshot");
    let collection_pass = access.check_collection_access(
        collection_name,
        AccessRequirements::new().manage()
    )?;
    
    self.toc
        .delete_snapshot(&collection_pass, snapshot_name, wait)
        .await?;
    
    Ok(())
}
```

#### 2.3.4 从快照恢复

```rust
/// 从快照恢复集合
pub async fn recover_from_snapshot(
    &self,
    collection_name: &str,
    snapshot_location: SnapshotRecover,
    wait: bool,
) -> Result<()> {
    let access = Access::full("recover_from_snapshot");
    
    self.toc
        .recover_from_snapshot(collection_name, snapshot_location, wait, &access)
        .await?;
    
    Ok(())
}
```

### 2.4 所需导入

```rust
use collection::operations::snapshot_ops::{
    SnapshotDescription, SnapshotRecover, SnapshotPriority,
};
use storage::content_manager::snapshots::{
    do_create_full_snapshot, do_delete_collection_snapshot,
    do_list_full_snapshots, do_recover_from_snapshot,
};
```

### 2.5 实现难度

**难度：中等**

快照操作涉及文件系统操作和异步处理，需要处理：
- 文件路径管理
- 异步文件读写
- 快照完整性验证
- 恢复时的并发控制

---

## 3. 集群管理功能

### 3.1 功能描述

集群管理功能用于分布式部署场景，管理集群节点和分片分布。

### 3.2 REST API 端点

```
GET    /cluster                        # 获取集群状态
POST   /cluster/recover                # 恢复集群
DELETE /cluster/peer/{peer_id}         # 移除节点
GET    /cluster/metadata/keys          # 获取集群元数据键
```

### 3.3 实现方案

#### 3.3.1 获取集群状态

```rust
/// 获取集群状态
pub async fn cluster_status(&self) -> Result<ClusterStatus> {
    let status = self.dispatcher.cluster_status();
    Ok(status)
}
```

#### 3.3.2 恢复集群

```rust
/// 恢复集群
pub async fn recover_cluster(&self) -> Result<()> {
    let access = Access::full("recover_cluster");
    access.check_global_access(AccessRequirements::new().manage())?;
    
    let pass = new_unchecked_verification_pass();
    self.toc.request_snapshot(&pass)?;
    
    Ok(())
}
```

#### 3.3.3 移除节点

```rust
/// 移除集群节点
pub async fn remove_peer(
    &self,
    peer_id: u64,
    force: bool,
    timeout: Option<Duration>,
) -> Result<()> {
    let access = Access::full("remove_peer");
    access.check_global_access(AccessRequirements::new().manage())?;
    
    let pass = new_unchecked_verification_pass();
    let toc = self.dispatcher.toc(&access, &pass);
    
    // 检查节点是否有分片
    let has_shards = toc.peer_has_shards(peer_id).await;
    if !force && has_shards {
        return Err(StorageError::BadRequest {
            description: format!("Cannot remove peer {peer_id} as there are shards on it"),
        }.into());
    }
    
    // 提交移除节点操作
    match self.dispatcher.consensus_state() {
        Some(consensus_state) => {
            consensus_state
                .propose_consensus_op_with_await(
                    ConsensusOperations::RemovePeer(peer_id),
                    timeout,
                )
                .await?;
        }
        None => {
            return Err(StorageError::BadRequest {
                description: "Distributed mode disabled.".to_string(),
            }.into());
        }
    }
    
    Ok(())
}
```

#### 3.3.4 获取集合集群信息

```rust
/// 获取集合的集群分布信息
pub async fn get_collection_cluster_info(
    &self,
    collection_name: &str,
) -> Result<CollectionClusterInfo> {
    let access = Access::full("get_collection_cluster_info");
    let collection_pass = access.check_collection_access(
        collection_name,
        AccessRequirements::new()
    )?;
    
    let collection = self.toc.get_collection(&collection_pass).await?;
    let peer_id = self.toc.this_peer_id;
    let cluster_info = collection.cluster_info(peer_id).await?;
    
    Ok(cluster_info)
}
```

### 3.4 所需导入

```rust
use storage::content_manager::consensus_ops::ConsensusOperations;
use collection::operations::verification::new_unchecked_verification_pass;
use storage::rbac::AccessRequirements;
```

### 3.5 实现难度

**难度：高**

集群管理功能涉及：
- 分布式共识协议
- 节点间通信
- 分片迁移
- 故障恢复
- 并发控制

**注意：** 对于单机嵌入使用场景，集群管理功能通常不需要。

---

## 4. 分片管理功能

### 4.1 功能描述

分片管理用于控制集合的分片分布，支持自定义分片和分片迁移。

### 4.2 REST API 端点

```
POST   /collections/{name}/shards                    # 创建分片
DELETE /collections/{name}/shards/{shard_id}         # 删除分片
PATCH  /collections/{name}/shards                    # 更新分片配置
POST   /collections/{name}/shards/move               # 迁移分片
```

### 4.3 实现方案

#### 4.3.1 创建分片

```rust
/// 创建自定义分片
pub async fn create_shard(
    &self,
    collection_name: &str,
    shard_id: ShardId,
    shard_key: Option<ShardKey>,
) -> Result<()> {
    let operation = ClusterOperations::CreateShard {
        collection_name: collection_name.to_string(),
        shard_id,
        shard_key,
    };
    
    self.submit_cluster_operation(operation).await
}
```

#### 4.3.2 删除分片

```rust
/// 删除分片
pub async fn delete_shard(
    &self,
    collection_name: &str,
    shard_id: ShardId,
) -> Result<()> {
    let operation = ClusterOperations::DeleteShard {
        collection_name: collection_name.to_string(),
        shard_id,
    };
    
    self.submit_cluster_operation(operation).await
}
```

#### 4.3.3 迁移分片

```rust
/// 迁移分片到另一个节点
pub async fn move_shard(
    &self,
    collection_name: &str,
    shard_id: ShardId,
    from_peer: PeerId,
    to_peer: PeerId,
    method: Option<ShardTransferMethod>,
) -> Result<()> {
    let operation = ClusterOperations::MoveShard {
        collection_name: collection_name.to_string(),
        shard_id,
        from_peer,
        to_peer,
        method,
    };
    
    self.submit_cluster_operation(operation).await
}
```

#### 4.3.4 辅助方法

```rust
/// 提交集群操作
async fn submit_cluster_operation(&self, operation: ClusterOperations) -> Result<()> {
    let access = Access::full("cluster_operation");
    access.check_global_access(AccessRequirements::new().manage())?;
    
    match self.dispatcher.consensus_state() {
        Some(consensus_state) => {
            consensus_state
                .propose_consensus_op_with_await(
                    ConsensusOperations::ClusterOperation(operation),
                    None,
                )
                .await?;
        }
        None => {
            return Err(StorageError::BadRequest {
                description: "Distributed mode disabled.".to_string(),
            }.into());
        }
    }
    
    Ok(())
}
```

### 4.4 所需导入

```rust
use collection::operations::cluster_ops::ClusterOperations;
use collection::shards::shard::ShardId;
use collection::shards::transfer::ShardTransferMethod;
use segment::types::ShardKey;
```

### 4.5 实现难度

**难度：高**

分片管理涉及：
- 分片创建和删除
- 分片迁移和复制
- 数据一致性保证
- 节点间协调
- 资源管理

**注意：** 对于单机嵌入使用场景，分片管理功能通常不需要。

---

## 5. 实现优先级建议

### 5.1 高优先级（建议实现）

1. **别名管理** - 实现简单，对应用开发有帮助
2. **快照操作** - 数据备份恢复是生产环境必需功能

### 5.2 低优先级（可选实现）

3. **集群管理** - 仅分布式部署需要
4. **分片管理** - 仅分布式部署需要

---

## 6. 实现注意事项

### 6.1 访问控制

所有功能都需要正确的访问控制：

```rust
// 读取操作
let access = Access::full("operation_name");
let collection_pass = access.check_collection_access(
    collection_name,
    AccessRequirements::new()
)?;

// 管理操作
let access = Access::full("operation_name");
let collection_pass = access.check_collection_access(
    collection_name,
    AccessRequirements::new().manage()
)?;
```

### 6.2 错误处理

使用统一的错误类型：

```rust
use crate::error::Result;
use storage::content_manager::errors::StorageError;

// 转换错误
.map_err(|err| err.into())
```

### 6.3 异步操作

长时间运行的操作需要支持等待参数：

```rust
pub async fn operation(
    &self,
    // ... 其他参数
    wait: bool,
    timeout: Option<Duration>,
) -> Result<...>
```

### 6.4 硬件计数器

所有操作都应该使用硬件计数器：

```rust
HwMeasurementAcc::disposable()  // 一次性操作
// 或
hw_measurement_acc  // 传入的计数器
```

---

## 7. 测试建议

### 7.1 单元测试

为每个新功能编写单元测试：

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_alias_operations() {
        // 创建临时目录
        let temp_dir = TempDir::new().unwrap();
        let config = EmbeddedConfig::builder()
            .storage_path(temp_dir.path())
            .build();
        
        let client = QdrantEmbedded::new(config).unwrap();
        
        // 创建集合
        client.create_collection("test".to_string(), /* config */).await.unwrap();
        
        // 测试别名操作
        // ...
    }
}
```

### 7.2 集成测试

测试功能之间的交互：

```rust
#[tokio::test]
async fn test_snapshot_workflow() {
    // 创建集合
    // 插入数据
    // 创建快照
    // 删除数据
    // 从快照恢复
    // 验证数据
}
```

---

## 8. 文档建议

### 8.1 API 文档

为每个公开方法添加详细的文档注释：

```rust
/// 功能简述
///
/// 详细描述
///
/// # 参数
///
/// * `param1` - 参数1说明
/// * `param2` - 参数2说明
///
/// # 返回值
///
/// 返回值说明
///
/// # 错误
///
/// 可能的错误情况
///
/// # 示例
///
/// ```rust
/// // 示例代码
/// ```
pub async fn method_name(&self, ...) -> Result<...>
```

### 8.2 使用示例

创建示例文件展示如何使用新功能：

```rust
// examples/snapshot_example.rs

use qdrant_embedded::{QdrantEmbedded, EmbeddedConfig};

#[tokio::main]
async fn main() {
    // 初始化
    let config = EmbeddedConfig::builder()
        .storage_path("./data")
        .build();
    let client = QdrantEmbedded::new(config).unwrap();
    
    // 创建快照
    let snapshot = client.create_snapshot("my_collection", true).await.unwrap();
    println!("Created snapshot: {:?}", snapshot);
    
    // 恢复快照
    client.recover_from_snapshot("my_collection", snapshot, true).await.unwrap();
    println!("Recovered from snapshot");
}
```

---

## 9. 总结

本文档详细说明了剩余功能的实现方案。建议按以下顺序实现：

1. **别名管理** - 最简单，可快速实现
2. **快照操作** - 重要且实用
3. **集群管理** - 仅在需要分布式功能时实现
4. **分片管理** - 仅在需要分布式功能时实现

实现这些功能后，嵌入库将覆盖约 80% 的 REST API 功能，足以满足大多数应用场景。
