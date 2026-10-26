//! HTTP and WebSocket gateway for Win Proof of Skill (WIN).
//! Validator requests preserve the WIN transaction body and signing network ID.

use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

#[derive(Clone)]
pub struct Stats {
    pub stream_count: Arc<AtomicUsize>,
}

pub mod archivers;
pub mod collector;
pub mod config;
pub mod crypto;
pub mod http;
pub mod notifier;
pub mod observer_gateway;
pub mod rpc;
pub mod shardus_monitor;
pub mod subscription;
pub mod tls;
pub mod win;
pub mod ws;

pub use config::Config;
