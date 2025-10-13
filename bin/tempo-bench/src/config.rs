use alloy::primitives::Address;
use eyre::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Root configuration structure for tempo-bench
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchConfig {
    /// Path to parent config to inherit from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherits: Option<String>,

    /// Benchmark-specific settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub benchmark: Option<BenchmarkConfig>,
}

/// Benchmark configuration settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkConfig {
    /// Initial rate limit (TPS) to start with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initial_ratelimit: Option<u64>,

    /// Progressive rate limit thresholds: [[tx_count, new_tps], ...]
    /// When tx_count transactions have been sent, increase TPS to new_tps
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ratelimit_thresholds: Option<Vec<[u64; 2]>>,

    /// Test duration in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,

    /// Number of accounts for pre-generation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accounts: Option<u64>,

    /// Number of workers to send transactions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workers: Option<usize>,

    /// Mnemonic for generating accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mnemonic: Option<String>,

    /// Chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u64>,

    /// Token address used when creating TIP20 transfer calldata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<Address>,

    /// Target URLs for network connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_urls: Option<Vec<String>>,

    /// Total network connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_connections: Option<u64>,

    /// Disable binding worker threads to specific CPU cores
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_thread_pinning: Option<bool>,

    /// File descriptor limit to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fd_limit: Option<u64>,
}

impl BenchConfig {
    /// Load configuration from a TOML file with inheritance support
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        let mut config: BenchConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;

        // Handle inheritance
        if let Some(inherits) = &config.inherits {
            let parent_path = Self::resolve_parent_path(path, inherits)?;
            let parent_config = Self::from_file(&parent_path)?;
            config = config.merge(parent_config);
        }

        Ok(config)
    }

    /// Resolve the parent config path relative to the current config
    fn resolve_parent_path(current_path: &Path, inherits: &str) -> Result<PathBuf> {
        let parent_dir = current_path
            .parent()
            .ok_or_else(|| eyre::eyre!("Config file has no parent directory"))?;

        let parent_path = parent_dir.join(inherits);

        if !parent_path.exists() {
            eyre::bail!(
                "Parent config file not found: {} (referenced from {})",
                parent_path.display(),
                current_path.display()
            );
        }

        Ok(parent_path)
    }

    /// Merge this config with a parent config (self takes precedence)
    fn merge(self, parent: BenchConfig) -> Self {
        BenchConfig {
            inherits: self.inherits.or(parent.inherits),
            benchmark: match (self.benchmark, parent.benchmark) {
                (Some(child), Some(parent)) => Some(child.merge(parent)),
                (Some(child), None) => Some(child),
                (None, Some(parent)) => Some(parent),
                (None, None) => None,
            },
        }
    }
}

impl BenchmarkConfig {
    /// Merge this benchmark config with a parent (self takes precedence)
    fn merge(self, parent: BenchmarkConfig) -> Self {
        BenchmarkConfig {
            initial_ratelimit: self.initial_ratelimit.or(parent.initial_ratelimit),
            ratelimit_thresholds: self.ratelimit_thresholds.or(parent.ratelimit_thresholds),
            duration: self.duration.or(parent.duration),
            accounts: self.accounts.or(parent.accounts),
            workers: self.workers.or(parent.workers),
            mnemonic: self.mnemonic.or(parent.mnemonic),
            chain_id: self.chain_id.or(parent.chain_id),
            token_address: self.token_address.or(parent.token_address),
            target_urls: self.target_urls.or(parent.target_urls),
            total_connections: self.total_connections.or(parent.total_connections),
            disable_thread_pinning: self
                .disable_thread_pinning
                .or(parent.disable_thread_pinning),
            fd_limit: self.fd_limit.or(parent.fd_limit),
        }
    }
}
