use std::sync::Arc;
use tokio::runtime::Runtime;

use storage::content_manager::toc::TableOfContent;
use storage::dispatcher::Dispatcher;
use storage::types::StorageConfig;
use collection::shards::channel_service::ChannelService;
use common::budget::ResourceBudget;

use crate::config::EmbeddedConfig;
use crate::error::Result;

/// Qdrant 嵌入库客户端
pub struct QdrantEmbedded {
    pub toc: Arc<TableOfContent>,
    pub dispatcher: Arc<Dispatcher>,
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
                max_search_threads: config.search_threads.unwrap_or(4),
                max_optimization_runtime_threads: config.optimizer_threads.unwrap_or(2),
                optimizer_cpu_budget: config.cpu_limit.map(|v| {
                    if v == 0 { 0 } else { v as isize }
                }).unwrap_or(0),
                optimizer_io_budget: config.io_limit.unwrap_or(0),
                update_rate_limit: config.update_rate_limit,
                search_timeout_sec: config.search_timeout_sec,
                incoming_shard_transfers_limit: None,
                outgoing_shard_transfers_limit: None,
                async_scorer: config.async_scorer,
            },
            snapshots_config: collection::common::snapshots_manager::SnapshotsConfig::default(),
            optimizers: config.optimizers_config.unwrap_or_else(|| collection::optimizers_builder::OptimizersConfig {
                deleted_threshold: 0.1,
                vacuum_min_vector_number: 1000,
                default_segment_number: 0,
                max_segment_size: None,
                #[expect(deprecated)]
                memmap_threshold: None,
                indexing_threshold: Some(100_000),
                flush_interval_sec: 60,
                max_optimization_threads: None,
            }),
            optimizers_overwrite: None,
            on_disk_payload: config.on_disk_payload.unwrap_or(false),
            wal: config.wal_config.unwrap_or_default(),
            hnsw_index: config.hnsw_config.unwrap_or_default(),
            hnsw_global_config: segment::types::HnswGlobalConfig::default(),
            mmap_advice: memory::madvise::Advice::Random,
            node_type: collection::operations::types::NodeType::Normal,
            update_queue_size: config.update_queue_size,
            handle_collection_load_errors: false,
            recovery_mode: None,
            update_concurrency: None,
            shard_transfer_method: None,
            collection: config.collection_defaults,
            max_collections: config.max_collections,
        };

        // 创建 TableOfContent
        let optimizer_resource_budget = ResourceBudget::new(
            storage_config.performance.optimizer_cpu_budget.max(0) as usize,
            storage_config.performance.optimizer_io_budget,
        );
        let channel_service = ChannelService::new(1000, None);

        // 直接创建 TableOfContent，不使用 block_on
        let toc = TableOfContent::new(
            &storage_config,
            search_runtime,
            update_runtime,
            general_runtime,
            optimizer_resource_budget,
            channel_service,
            0,
            None,
        );

        let toc = Arc::new(toc);
        let dispatcher = Arc::new(Dispatcher::new(toc.clone()));

        Ok(Self {
            toc,
            dispatcher,
        })
    }
}
