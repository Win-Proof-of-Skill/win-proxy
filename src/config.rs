//! Configuration for the WIN proxy server.
use std::{fs, path::Path};

#[derive(Debug, serde::Deserialize, Clone)]
pub struct Config {
    /// The port on which the proxy server will listen
    pub http_port: u16,

    /// cryptographic seed
    pub crypto_seed: String,

    /// This is a system directory path to the archiver seed file
    /// Archiver seed file should contain a list of reputable and reliable node
    pub archiver_seed_path: String,

    /// The interval in seconds at which the node list will be refreshed
    pub nodelist_refresh_interval_sec: u64,

    /// This is currently not used anywhere
    pub debug: bool,

    /// The maximum time in milliseconds that the rpc will cancel a request when communication with
    /// consensus node
    pub max_http_timeout_ms: u128,

    /// TCP keep alive time in seconds
    pub tcp_keepalive_time_sec: u32,

    /// Standalone network configuration
    pub standalone_network: StandaloneNetworkConfig,

    /// Node filtering configuration for join-ordered lists
    pub node_filtering: NodeFilteringConfig,

    pub tls: TLSConfig,

    pub shardus_monitor: ShardusMonitorProxyConfig,

    pub local_source: LocalSource,

    pub notifier: NotifierConfig,

    /// Base URLs of TSS observers (e.g. http://127.0.0.1:8101). Used to forward
    /// POST /notify-bridgeout from the bridge UI so observers can poll for BridgeOut events.
    #[serde(default)]
    pub observer_urls: Vec<String>,
}

#[derive(Debug, serde::Deserialize, Clone)]
/// Standalone network mean that consensus node and archivers reside within same server.
/// This mean archiver will returns node list with loopback ip address 0.0.0.0, 127.0.0.1, localhost.
/// When rpc is on a separate machine, loopback ips of nodes will not work.
/// Usually standalone network config are for testnets. Should not be used in productions
pub struct StandaloneNetworkConfig {
    pub replacement_ip: String,
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
/// Node filtering configuration for join-ordered node lists
/// This helps avoid unstable nodes that are joining or leaving the network
pub struct NodeFilteringConfig {
    /// Whether to enable node filtering based on join order
    pub enabled: bool,
    /// Number of nodes to remove from the top of the join-ordered list
    pub remove_top_nodes: usize,
    /// Number of nodes to remove from the bottom of the join-ordered list
    pub remove_bottom_nodes: usize,
    /// Minimum number of nodes required before filtering is applied
    pub min_nodes_for_filtering: usize,
}

#[derive(Debug, serde::Deserialize, Clone)]
/// TLS configuration
pub struct TLSConfig {
    pub enabled: bool,
    pub cert_path: String,
    pub key_path: String,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct ShardusMonitorProxyConfig {
    pub enabled: bool,
    pub upstream_ip: String,
    pub upstream_port: u16,
    pub https: bool,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct LocalSource {
    pub collector_api_ip: String,
    pub collector_api_port: u16,
    pub collector_event_server_ip: String,
    pub collector_event_server_port: u16,
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct NotifierConfig {
    pub ip: String,
    pub port: u16,
}

impl Config {
    /// Select a deployment without modifying the checked-in configuration.
    pub fn load() -> Result<Self, String> {
        let path = std::env::var_os("WIN_PROXY_CONFIG").unwrap_or_else(|| "src/config.json".into());
        Self::load_from(Path::new(&path))
    }

    /// File paths are relative to this config file, or may be absolute.
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref();
        let config_data = fs::read_to_string(path)
            .map_err(|err| format!("Failed to read config {}: {}", path.display(), err))?;
        let mut config: Self = serde_json::from_str(&config_data)
            .map_err(|err| format!("Failed to parse config {}: {}", path.display(), err))?;
        config.validate()?;
        let directory = path.parent().unwrap_or_else(|| Path::new("."));
        for value in [
            &mut config.archiver_seed_path,
            &mut config.tls.cert_path,
            &mut config.tls.key_path,
        ] {
            if !value.is_empty() && Path::new(value).is_relative() {
                *value = directory.join(&*value).to_string_lossy().into_owned();
            }
        }
        Ok(config)
    }

    fn validate(&self) -> Result<(), String> {
        if self.http_port == 0 || self.http_port == u16::MAX {
            return Err(
                "http_port must be between 1 and 65534 (WebSocket uses the next port)".into(),
            );
        }
        if self.nodelist_refresh_interval_sec == 0
            || self.nodelist_refresh_interval_sec > u64::MAX / 2
        {
            return Err(
                "nodelist_refresh_interval_sec must be positive and fit doubled in u64".into(),
            );
        }
        if self.max_http_timeout_ms == 0
            || self.max_http_timeout_ms > (u64::MAX / 2) as u128
            || self.tcp_keepalive_time_sec == 0
        {
            return Err("Request and keep-alive timeouts must be positive and fit in u64".into());
        }
        if self.crypto_seed.len() != 64 || !self.crypto_seed.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(
                "crypto_seed must be the WIN archiver's 64-character hexadecimal hash key".into(),
            );
        }
        if self.archiver_seed_path.trim().is_empty() {
            return Err("archiver_seed_path must identify the WIN archiver seed file".into());
        }
        if self.standalone_network.enabled
            && self.standalone_network.replacement_ip.trim().is_empty()
        {
            return Err("standalone_network.replacement_ip is required when enabled".into());
        }
        if self.tls.enabled && (self.tls.cert_path.is_empty() || self.tls.key_path.is_empty()) {
            return Err("TLS requires cert_path and key_path".into());
        }
        Ok(())
    }

    pub fn load_archiver_seed(&self) -> Result<Vec<crate::archivers::Archiver>, String> {
        let data = fs::read_to_string(&self.archiver_seed_path).map_err(|err| {
            format!(
                "Failed to read WIN archiver seeds {}: {}",
                self.archiver_seed_path, err
            )
        })?;
        let seeds: Vec<crate::archivers::Archiver> = serde_json::from_str(&data)
            .map_err(|err| format!("Failed to parse WIN archiver seeds: {}", err))?;
        if seeds.is_empty() {
            return Err("No WIN archiver seeds configured. Add your WIN archiver's IP, port and publicKey to archiver_seed_path.".into());
        }
        for seed in &seeds {
            if seed.ip.trim().is_empty()
                || seed.port == 0
                || seed.publicKey.len() != 64
                || !seed.publicKey.bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("Each WIN archiver seed requires an IP/hostname, a nonzero port and a 64-character hexadecimal publicKey".into());
            }
        }
        Ok(seeds)
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::load().expect("Default config should load from src/config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_reads_default_config() {
        let cfg = Config::load_from("src/config.json").expect("config should parse");
        assert_eq!(cfg.http_port, 3030);
        assert!(cfg.debug);
        assert!(!cfg.tls.enabled);
        assert_eq!(cfg.node_filtering.remove_top_nodes, 3);
        assert!(!cfg.standalone_network.enabled);
        let seeds = cfg.load_archiver_seed().unwrap();
        assert_eq!(seeds[0].ip, "127.0.0.1");
        assert_eq!(seeds[0].port, 4000);
    }

    #[test]
    fn custom_config_resolves_paths_beside_the_file() {
        let directory =
            std::env::temp_dir().join(format!("win-proxy-config-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        let mut data: serde_json::Value =
            serde_json::from_str(include_str!("config.json")).unwrap();
        data["archiver_seed_path"] = "seed.json".into();
        data["tls"]["cert_path"] = "certs/win.crt".into();
        let path = directory.join("custom.json");
        fs::write(&path, data.to_string()).unwrap();
        fs::write(
            directory.join("seed.json"),
            include_str!("archiver_seed.json"),
        )
        .unwrap();
        let cfg = Config::load_from(&path).unwrap();
        assert_eq!(
            Path::new(&cfg.tls.cert_path),
            directory.join("certs/win.crt")
        );
        assert_eq!(cfg.load_archiver_seed().unwrap().len(), 1);

        fs::write(directory.join("seed.json"), "[]").unwrap();
        assert!(cfg
            .load_archiver_seed()
            .err()
            .unwrap()
            .contains("No WIN archiver"));
        fs::write(
            directory.join("seed.json"),
            r#"[{"ip":"127.0.0.1","port":4000,"publicKey":"invalid"}]"#,
        )
        .unwrap();
        assert!(cfg.load_archiver_seed().is_err());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn rejects_config_that_cannot_start() {
        let base: serde_json::Value = serde_json::from_str(include_str!("config.json")).unwrap();
        for (field, value) in [
            ("http_port", serde_json::json!(65535)),
            ("http_port", serde_json::json!(0)),
            ("crypto_seed", serde_json::json!("not-a-hash-key")),
            ("nodelist_refresh_interval_sec", serde_json::json!(0)),
            ("max_http_timeout_ms", serde_json::json!(0)),
        ] {
            let mut data = base.clone();
            data[field] = value;
            let config: Config = serde_json::from_value(data).unwrap();
            assert!(config.validate().is_err(), "{} should be rejected", field);
        }
    }
}
