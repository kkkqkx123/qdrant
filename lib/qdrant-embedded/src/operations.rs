use std::time::Duration;

use storage::content_manager::collection_meta_ops::{CreateCollection, CreateCollectionOperation, DeleteCollectionOperation};
use storage::rbac::Access;
use collection::operations::shard_selector_internal::ShardSelectorInternal;
use common::counter::hardware_accumulator::HwMeasurementAcc;

use crate::client::QdrantEmbedded;
use crate::error::Result;
use collection::operations::point_ops::WriteOrdering;
use collection::operations::types::{VectorsConfig, ScrollRequestInternal, ScrollResult};
use collection::operations::consistency_params::ReadConsistency;
use collection::operations::universal_query::collection_query::CollectionQueryRequest;
use collection::operations::{OperationWithClockTag, CollectionUpdateOperations};
use segment::types::ScoredPoint;
use shard::search::CoreSearchRequestBatch;
use shard::operations::point_ops::PointOperations;
use shard::operations::payload_ops::{PayloadOps, SetPayload, SetPayloadOp};
use shard::operations::vector_ops::VectorOperations;

impl QdrantEmbedded {
    /// 列出所有集合
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let access = Access::full("list_collections");
        let response = self.toc.all_collections(&access).await;
        Ok(response.into_iter().map(|c| c.name().to_string()).collect())
    }

    /// 创建集合
    pub async fn create_collection(
        &self,
        collection_name: String,
        vectors_config: VectorsConfig,
    ) -> Result<()> {
        let operation = CreateCollection {
            vectors: vectors_config,
            shard_number: None,
            sharding_method: None,
            replication_factor: None,
            write_consistency_factor: None,
            on_disk_payload: None,
            hnsw_config: None,
            wal_config: None,
            optimizers_config: None,
            quantization_config: None,
            sparse_vectors: None,
            strict_mode_config: None,
            uuid: None,
            metadata: None,
        };
        let create_collection_op = CreateCollectionOperation::new(collection_name, operation)?;
        self.dispatcher
            .submit_collection_meta_op(
                storage::content_manager::collection_meta_ops::CollectionMetaOperations::CreateCollection(
                    create_collection_op,
                ),
                Access::full("create_collection"),
                None,
            )
            .await?;
        Ok(())
    }

    /// 删除集合
    pub async fn delete_collection(&self, collection_name: String) -> Result<()> {
        self.dispatcher
            .submit_collection_meta_op(
                storage::content_manager::collection_meta_ops::CollectionMetaOperations::DeleteCollection(
                    DeleteCollectionOperation(collection_name)
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
        collection_name: String,
        operation: PointOperations,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::PointOperation(operation)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("upsert_points"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 删除点
    pub async fn delete_points(
        &self,
        collection_name: String,
        points: PointOperations,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::PointOperation(points)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("delete_points"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 搜索点（批量）
    /// 
    /// 使用 CoreSearchRequestBatch 进行批量搜索
    pub async fn search_points(
        &self,
        collection_name: &str,
        request: CoreSearchRequestBatch,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Vec<ScoredPoint>>> {
        let result = self.toc.core_search_batch(
            collection_name,
            request,
            read_consistency,
            ShardSelectorInternal::All,
            Access::full("search"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// 查询点（批量）
    /// 
    /// 使用 CollectionQueryRequest 进行查询
    pub async fn query_points(
        &self,
        collection_name: &str,
        requests: Vec<(CollectionQueryRequest, ShardSelectorInternal)>,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Vec<ScoredPoint>>> {
        let result = self.toc.query_batch(
            collection_name,
            requests,
            read_consistency,
            Access::full("query"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// 滚动浏览点
    /// 
    /// 使用 ScrollRequestInternal 进行滚动浏览
    pub async fn scroll_points(
        &self,
        collection_name: &str,
        request: ScrollRequestInternal,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<ScrollResult> {
        let result = self.toc.scroll(
            collection_name,
            request,
            read_consistency,
            timeout,
            ShardSelectorInternal::All,
            Access::full("scroll"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// 设置 payload
    pub async fn set_payload(
        &self,
        collection_name: String,
        payload: SetPayload,
    ) -> Result<()> {
        let payload_op = SetPayloadOp {
            payload: payload.payload,
            points: payload.points,
            filter: payload.filter,
            key: payload.key,
        };
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::PayloadOperation(PayloadOps::SetPayload(payload_op))),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("set_payload"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 删除 payload
    pub async fn delete_payload(
        &self,
        collection_name: String,
        payload: PayloadOps,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::PayloadOperation(payload)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("delete_payload"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 清除 payload
    pub async fn clear_payload(
        &self,
        collection_name: String,
        payload: PayloadOps,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::PayloadOperation(payload)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("clear_payload"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 更新向量
    pub async fn update_vectors(
        &self,
        collection_name: String,
        vectors: VectorOperations,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::VectorOperation(vectors)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("update_vectors"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }

    /// 删除向量
    pub async fn delete_vectors(
        &self,
        collection_name: String,
        vectors: VectorOperations,
    ) -> Result<()> {
        self.toc.update(
            &collection_name,
            OperationWithClockTag::from(CollectionUpdateOperations::VectorOperation(vectors)),
            true,
            WriteOrdering::Strong,
            ShardSelectorInternal::All,
            Access::full("delete_vectors"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(())
    }
}
