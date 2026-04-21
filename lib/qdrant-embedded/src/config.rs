use std::path::PathBuf;
use collection::optimizers_builder::OptimizersConfig;
use collection::config::WalConfig;
use segment::types::HnswConfig;
use segment::data_types::collection_defaults::CollectionConfigDefaults;

/// 嵌入库配置
#[derive(Debug, Clone)]
pub struct EmbeddedConfig {
    /// 存储路径
    pub storage_path: PathBuf,
    /// 快照路径（可选）
    pub snapshots_path: Option<PathBuf>,
    /// 临时文件路径（可选）
    pub temp_path: Option<PathBuf>,
    
    // 性能配置
    /// 搜索线程数
    pub search_threads: Option<usize>,
    /// 优化线程数
    pub optimizer_threads: Option<usize>,
    /// IO 限制（可选）
    pub io_limit: Option<usize>,
    /// CPU 限制（可选）
    pub cpu_limit: Option<usize>,
    /// 更新速率限制（可选）
    pub update_rate_limit: Option<usize>,
    /// 搜索超时秒数（可选）
    pub search_timeout_sec: Option<usize>,
    /// 是否使用异步评分器（可选）
    pub async_scorer: Option<bool>,
    
    // 存储配置
    /// 是否将 payload 存储在磁盘上
    pub on_disk_payload: Option<bool>,
    
    // 高级配置
    /// 优化器配置（可选）
    pub optimizers_config: Option<OptimizersConfig>,
    /// WAL 配置（可选）
    pub wal_config: Option<WalConfig>,
    /// HNSW 索引配置（可选）
    pub hnsw_config: Option<HnswConfig>,
    /// 集合默认配置（可选）
    pub collection_defaults: Option<CollectionConfigDefaults>,
    
    // 限制配置
    /// 最大集合数（可选）
    pub max_collections: Option<usize>,
    /// 更新队列大小（可选）
    pub update_queue_size: Option<usize>,
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
            update_rate_limit: None,
            search_timeout_sec: None,
            async_scorer: None,
            on_disk_payload: None,
            optimizers_config: None,
            wal_config: None,
            hnsw_config: None,
            collection_defaults: None,
            max_collections: None,
            update_queue_size: None,
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

    pub fn update_rate_limit(mut self, limit: usize) -> Self {
        self.config.update_rate_limit = Some(limit);
        self
    }

    pub fn search_timeout_sec(mut self, timeout: usize) -> Self {
        self.config.search_timeout_sec = Some(timeout);
        self
    }

    pub fn async_scorer(mut self, enabled: bool) -> Self {
        self.config.async_scorer = Some(enabled);
        self
    }

    pub fn on_disk_payload(mut self, enabled: bool) -> Self {
        self.config.on_disk_payload = Some(enabled);
        self
    }

    pub fn optimizers_config(mut self, config: OptimizersConfig) -> Self {
        self.config.optimizers_config = Some(config);
        self
    }

    pub fn wal_config(mut self, config: WalConfig) -> Self {
        self.config.wal_config = Some(config);
        self
    }

    pub fn hnsw_config(mut self, config: HnswConfig) -> Self {
        self.config.hnsw_config = Some(config);
        self
    }

    pub fn collection_defaults(mut self, defaults: CollectionConfigDefaults) -> Self {
        self.config.collection_defaults = Some(defaults);
        self
    }

    pub fn max_collections(mut self, max: usize) -> Self {
        self.config.max_collections = Some(max);
        self
    }

    pub fn update_queue_size(mut self, size: usize) -> Self {
        self.config.update_queue_size = Some(size);
        self
    }

    pub fn build(self) -> EmbeddedConfig {
        self.config
    }
}
