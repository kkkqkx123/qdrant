use std::time::Duration;

use storage::content_manager::collection_meta_ops::{
    CreateCollection, CreateCollectionOperation, DeleteCollectionOperation,
    UpdateCollection, UpdateCollectionOperation, CollectionMetaOperations,
    ChangeAliasesOperation,
};
use storage::content_manager::snapshots::{
    do_delete_collection_snapshot,
};
use storage::content_manager::snapshots::recover::do_recover_from_snapshot;
use storage::rbac::Access;
use collection::operations::shard_selector_internal::ShardSelectorInternal;
use common::counter::hardware_accumulator::HwMeasurementAcc;
use segment::types::QuantizationConfig;

use crate::client::QdrantEmbedded;
use crate::error::Result;
use collection::operations::point_ops::WriteOrdering;
use collection::operations::types::{
    VectorsConfig, ScrollRequestInternal, ScrollResult, PointRequestInternal,
    CountRequestInternal, CountResult, RecommendRequestInternal, DiscoverRequestInternal,
    CollectionInfo, AliasDescription,
};
use collection::operations::snapshot_ops::{SnapshotDescription, SnapshotRecover};
use collection::operations::consistency_params::ReadConsistency;
use collection::operations::universal_query::collection_query::CollectionQueryRequest;
use collection::operations::{OperationWithClockTag, CollectionUpdateOperations};
use segment::types::ScoredPoint;
use segment::data_types::facets::{FacetParams, FacetResponse};
use shard::search::CoreSearchRequestBatch;
use shard::operations::point_ops::PointOperations;
use shard::operations::payload_ops::{PayloadOps, SetPayload, SetPayloadOp};
use shard::operations::vector_ops::VectorOperations;
use shard::retrieve::record_internal::RecordInternal;
use collection::collection::distance_matrix::{
    CollectionSearchMatrixRequest, CollectionSearchMatrixResponse,
};
use collection::grouping::group_by::GroupRequest;
use collection::operations::types::GroupsResult;

impl QdrantEmbedded {
    /// List all collections
    pub async fn list_collections(&self) -> Result<Vec<String>> {
        let access = Access::full("list_collections");
        let response = self.toc.all_collections(&access).await;
        Ok(response.into_iter().map(|c| c.name().to_string()).collect())
    }

    /// Creating Collections
    ///
    /// # Arguments
    /// * `collection_name` - Name of the collection to create
    /// * `vectors_config` - Vector configuration (size, distance, etc.)
    /// * `quantization_config` - Optional quantization configuration for vector compression
    ///
    /// # Quantization Options
    /// - `None`: No quantization (default)
    /// - `Scalar`: 8-bit scalar quantization, good balance of speed and accuracy
    /// - `Product`: High compression ratio, ideal for large-scale search
    /// - `Binary`: Maximum compression, best for approximate search on huge datasets
    pub async fn create_collection(
        &self,
        collection_name: String,
        vectors_config: VectorsConfig,
        quantization_config: Option<QuantizationConfig>,
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
            quantization_config,
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

    /// Creating Collections with full configuration
    ///
    /// This method provides full control over all collection creation parameters.
    pub async fn create_collection_full(
        &self,
        collection_name: String,
        operation: CreateCollection,
    ) -> Result<()> {
        let create_collection_op = CreateCollectionOperation::new(collection_name, operation)?;
        self.dispatcher
            .submit_collection_meta_op(
                storage::content_manager::collection_meta_ops::CollectionMetaOperations::CreateCollection(
                    create_collection_op,
                ),
                Access::full("create_collection_full"),
                None,
            )
            .await?;
        Ok(())
    }

    /// Deleting Collections
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

    /// Insert or update points
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

    /// deletion point
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

    /// Search points (bulk)
    /// 
    /// Bulk Searching with CoreSearchRequestBatch
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

    /// Inquiry points (batch)
    /// 
    /// Querying with CollectionQueryRequest
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

    /// Scroll through the points
    /// 
    /// Scrolling with ScrollRequestInternal
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

    /// Setting the payload
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

    /// Update Vector
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

    /// Deletion Vector
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

    /// Access Points (by ID)
    ///
    /// Retrieve the specified point using PointRequestInternal
    pub async fn get_points(
        &self,
        collection_name: &str,
        request: PointRequestInternal,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<Vec<RecordInternal>> {
        let result = self.toc.retrieve(
            collection_name,
            request,
            read_consistency,
            timeout,
            ShardSelectorInternal::All,
            Access::full("get_points"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// measuring point
    ///
    /// Using CountRequestInternal to count points
    pub async fn count_points(
        &self,
        collection_name: &str,
        request: CountRequestInternal,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<CountResult> {
        let result = self.toc.count(
            collection_name,
            request,
            read_consistency,
            timeout,
            ShardSelectorInternal::All,
            Access::full("count_points"),
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// recommended point
    ///
    /// Recommendation Queries with RecommendRequestInternal
    pub async fn recommend_points(
        &self,
        collection_name: &str,
        request: RecommendRequestInternal,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ScoredPoint>> {
        let result = self.toc.recommend(
            collection_name,
            request,
            read_consistency,
            ShardSelectorInternal::All,
            Access::full("recommend_points"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// point of discovery
    ///
    /// Discovery Queries with DiscoverRequestInternal
    pub async fn discover_points(
        &self,
        collection_name: &str,
        request: DiscoverRequestInternal,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<Vec<ScoredPoint>> {
        let result = self.toc.discover(
            collection_name,
            request,
            read_consistency,
            ShardSelectorInternal::All,
            Access::full("discover_points"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// Facet aggregation
    ///
    /// Aggregate Queries with FacetParams
    pub async fn facet(
        &self,
        collection_name: &str,
        request: FacetParams,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<FacetResponse> {
        let result = self.toc.facet(
            collection_name,
            request,
            ShardSelectorInternal::All,
            read_consistency,
            Access::full("facet"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// Search Point Distance Matrix
    ///
    /// Use CollectionSearchMatrixRequest to compute the distance matrix between points
    pub async fn search_points_matrix(
        &self,
        collection_name: &str,
        request: CollectionSearchMatrixRequest,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<CollectionSearchMatrixResponse> {
        let result = self.toc.search_points_matrix(
            collection_name,
            request,
            read_consistency,
            ShardSelectorInternal::All,
            Access::full("search_points_matrix"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// grouping inquiry
    ///
    /// Grouping Queries with GroupRequest
    pub async fn group_points(
        &self,
        collection_name: &str,
        request: GroupRequest,
        read_consistency: Option<ReadConsistency>,
        timeout: Option<Duration>,
    ) -> Result<GroupsResult> {
        let result = self.toc.group(
            collection_name,
            request,
            read_consistency,
            ShardSelectorInternal::All,
            Access::full("group_points"),
            timeout,
            HwMeasurementAcc::disposable(),
        ).await?;
        Ok(result)
    }

    /// Getting Collection Information
    ///
    /// Get detailed configuration and status information about the collection
    pub async fn get_collection_info(
        &self,
        collection_name: &str,
    ) -> Result<CollectionInfo> {
        let access = Access::full("get_collection_info");
        let collection_pass = access.check_collection_access(collection_name, storage::rbac::AccessRequirements::new())?;
        let collection = self.toc.get_collection(&collection_pass).await?;
        let info = collection.info(&ShardSelectorInternal::All).await?;
        Ok(info)
    }

    /// Updating the collection configuration
    ///
    /// Parameters for updating a collection with UpdateCollection
    pub async fn update_collection(
        &self,
        collection_name: String,
        update: UpdateCollection,
    ) -> Result<()> {
        let update_op = UpdateCollectionOperation::new(collection_name, update);
        self.dispatcher
            .submit_collection_meta_op(
                CollectionMetaOperations::UpdateCollection(update_op),
                Access::full("update_collection"),
                None,
            )
            .await?;
        Ok(())
    }

    /// Check if the collection exists
    ///
    /// Returns whether the collection exists
    pub async fn collection_exists(
        &self,
        collection_name: &str,
    ) -> Result<bool> {
        let access = Access::full("collection_exists");
        let collections = self.toc.all_collections(&access).await;
        let exists = collections.iter().any(|c| c.name() == collection_name);
        Ok(exists)
    }

    /// List all aliases
    ///
    /// Returns a list of aliases for all collections
    pub async fn list_aliases(&self) -> Result<Vec<AliasDescription>> {
        let access = Access::full("list_aliases");
        let aliases = self.toc.list_aliases(&access).await?;
        Ok(aliases)
    }

    /// List the aliases of a collection
    ///
    /// Returns all aliases of the specified collection
    pub async fn list_collection_aliases(
        &self,
        collection_name: &str,
    ) -> Result<Vec<String>> {
        let access = Access::full("list_collection_aliases");
        let collection_pass = access.check_collection_access(
            collection_name,
            storage::rbac::AccessRequirements::new()
        )?;
        let aliases = self.toc.collection_aliases(&collection_pass, &access).await?;
        Ok(aliases)
    }

    /// Updating aliases
    ///
    /// Creating, deleting or renaming aliases
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

    /// Creating a Collection Snapshot
    ///
    /// Creates a snapshot of a specified collection
    pub async fn create_snapshot(
        &self,
        collection_name: &str,
    ) -> Result<SnapshotDescription> {
        let access = Access::full("create_snapshot");
        let collection_pass = access.check_collection_access(
            collection_name,
            storage::rbac::AccessRequirements::new().manage()
        )?;
        
        let snapshot = self.toc.create_snapshot(&collection_pass).await?;
        Ok(snapshot)
    }

    /// List collection snapshots
    ///
    /// Returns a list of all snapshots in the specified collection
    pub async fn list_snapshots(
        &self,
        collection_name: &str,
    ) -> Result<Vec<SnapshotDescription>> {
        let access = Access::full("list_snapshots");
        let collection_pass = access.check_collection_access(
            collection_name,
            storage::rbac::AccessRequirements::new()
        )?;
        
        let collection = self.toc.get_collection(&collection_pass).await?;
        let snapshots = collection.list_snapshots().await?;
        Ok(snapshots)
    }

    /// Deleting a Collection Snapshot
    ///
    /// Deletes the specified snapshot file
    pub async fn delete_snapshot(
        &self,
        collection_name: &str,
        snapshot_name: &str,
    ) -> Result<bool> {
        let result = do_delete_collection_snapshot(
            &self.dispatcher,
            Access::full("delete_snapshot"),
            collection_name,
            snapshot_name,
        ).await?;
        Ok(result)
    }

    /// Restoring a collection from a snapshot
    ///
    /// Recovering Collection Data from Snapshot Files
    pub async fn recover_from_snapshot(
        &self,
        collection_name: &str,
        source: SnapshotRecover,
    ) -> Result<bool> {
        let client = reqwest::Client::new();
        let result = do_recover_from_snapshot(
            &self.dispatcher,
            collection_name,
            source,
            Access::full("recover_from_snapshot"),
            client,
        ).await?;
        Ok(result)
    }
}
