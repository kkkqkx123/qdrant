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
