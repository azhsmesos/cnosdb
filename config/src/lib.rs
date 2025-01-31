use std::{fs::File, io::prelude::Read};

use serde::{Deserialize, Serialize};
use trace::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub query: QueryConfig,
    pub storage: StorageConfig,
    pub wal: WalConfig,
    pub cache: CacheConfig,
    pub log: LogConfig,
    pub security: SecurityConfig,
    pub reporting_disabled: Option<bool>,
}

impl Config {
    pub fn override_by_env(&mut self) {
        self.storage.override_by_env();
        self.wal.override_by_env();
        self.cache.override_by_env();
        self.query.override_by_env();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    pub max_server_connections: u32,
    pub query_sql_limit: u64,
    pub write_sql_limit: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub path: String,
    pub max_summary_size: u64,
    pub max_level: u32,
    pub base_file_size: u64,
    pub compact_trigger: u32,
    pub max_compact_size: u64,
    pub strict_write: bool,
}

impl StorageConfig {
    pub fn override_by_env(&mut self) {
        if let Ok(path) = std::env::var("CNOSDB_APPLICATION_PATH") {
            self.path = path;
        }
        if let Ok(size) = std::env::var("CNOSDB_SUMMARY_MAX_SUMMARY_SIZE") {
            self.max_summary_size = size.parse::<u64>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_STORAGE_MAX_LEVEL") {
            self.max_level = size.parse::<u32>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_STORAGE_BASE_FILE_SIZE") {
            self.base_file_size = size.parse::<u64>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_STORAGE_COMPACT_TRIGGER") {
            self.compact_trigger = size.parse::<u32>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_STORAGE_MAX_COMPACT_SIZE") {
            self.max_compact_size = size.parse::<u64>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_STORAGE_STRICT_WRITE") {
            self.strict_write = size.parse::<bool>().unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalConfig {
    pub enabled: bool,
    pub path: String,
    pub sync: bool,
}

impl WalConfig {
    pub fn override_by_env(&mut self) {
        if let Ok(enabled) = std::env::var("CNOSDB_WAL_ENABLED") {
            self.enabled = enabled.as_str() == "true";
        }
        if let Ok(path) = std::env::var("CNOSDB_WAL_PATH") {
            self.path = path;
        }
        if let Ok(sync) = std::env::var("CNOSDB_WAL_SYNC") {
            self.sync = sync.as_str() == sync;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_buffer_size: u64,
    pub max_immutable_number: u16,
}

impl CacheConfig {
    pub fn override_by_env(&mut self) {
        if let Ok(size) = std::env::var("CNOSDB_CACHE_MAX_BUFFER_SIZE") {
            self.max_buffer_size = size.parse::<u64>().unwrap();
        }
        if let Ok(size) = std::env::var("CNOSDB_CACHE_MAX_IMMUTABLE_NUMBER") {
            self.max_immutable_number = size.parse::<u16>().unwrap();
        }
    }
}

impl QueryConfig {
    pub fn override_by_env(&mut self) {
        if let Ok(size) = std::env::var("MAX_SERVER_CONNECTIONS") {
            self.max_server_connections = size.parse::<u32>().unwrap();
        }
        if let Ok(size) = std::env::var("QUERY_SQL_LIMIT") {
            self.query_sql_limit = size.parse::<u64>().unwrap();
        }
        if let Ok(size) = std::env::var("WRITE_SQL_LIMIT") {
            self.write_sql_limit = size.parse::<u64>().unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub path: String,
}

impl LogConfig {
    pub fn override_by_env(&mut self) {
        if let Ok(level) = std::env::var("CNOSDB_LOG_LEVEL") {
            self.level = level;
        }
        if let Ok(path) = std::env::var("CNOSDB_LOG_PATH") {
            self.path = path;
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub tls_config: Option<TLSConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TLSConfig {
    pub certificate: String,
    pub private_key: String,
}

pub fn get_config(path: &str) -> Config {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(err) => panic!("Failed to open configurtion file '{}': {}", path, err),
    };
    let mut content = String::new();
    if let Err(err) = file.read_to_string(&mut content) {
        panic!("Failed to read configurtion file '{}': {}", path, err);
    }
    let config: Config = match toml::from_str(&content) {
        Ok(config) => config,
        Err(err) => panic!("Failed to parse configurtion file '{}': {}", path, err),
    };
    info!("Start with configuration: {:#?}", config);
    config
}

#[test]
fn test() {
    let config_str = r#"
[query]
max_server_connections = 10240
query_sql_limit = 16777216   # 16 * 1024 * 1024
write_sql_limit = 167772160   # 160 * 1024 * 1024
[storage]
path = 'data/db'
max_summary_size = 134217728 # 128 * 1024 * 1024
max_level = 4
base_file_size = 16777216 # 16 * 1024 * 1024
compact_trigger = 4
max_compact_size = 2147483648 # 2 * 1024 * 1024 * 1024
strict_write = true

[wal]
enabled = true
path = 'data/wal'
sync = true

[cache]
max_buffer_size = 1048576 # 134217728 # 128 * 1024 * 1024
max_immutable_number = 4

[log]
level = 'info'
path = 'data/log'

[security]

"#;

    let config: Config = toml::from_str(config_str).unwrap();
    dbg!(config);
}
