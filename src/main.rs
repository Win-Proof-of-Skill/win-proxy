//! WIN proxy: HTTP on the configured port and WebSocket on the next port.
//! See the repository README for local WIN network and deployment setup.
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_rustls::TlsAcceptor;

use win_proxy::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _configs = Config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let tls_config = if _configs.tls.enabled {
        match tls::configure_tls(&_configs.tls.cert_path, &_configs.tls.key_path) {
            Ok(c) => Some(c),
            Err(e) => {
                eprintln!("Failed to configure TLS: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        None
    };

    let tls_acceptor = match tls_config.is_some() {
        true => {
            let tls_config = tls_config.unwrap();
            let tls_acceptor = TlsAcceptor::from(Arc::new(tls_config));
            Some(tls_acceptor)
        }
        false => None,
    };

    let archiver_seed = _configs.load_archiver_seed()?;

    let crypto = Arc::new(crypto::ShardusCrypto::new(&_configs.crypto_seed));

    let arch_utils = Arc::new(archivers::ArchiverUtil::new(
        crypto.clone(),
        archiver_seed,
        _configs.clone(),
    ));

    let network = Arc::new(win::WinCoin::new(
        crypto.clone(),
        arch_utils.get_active_archivers(),
        _configs.clone(),
    ));

    let _archivers = Arc::clone(&arch_utils);
    let _win = Arc::clone(&network);

    // first time archiver and nodelist queries
    Arc::clone(&_archivers).discover().await;
    _win.update_active_nodelist().await;

    {
        // schedule a background task that refresh nodelist
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(
                _configs.nodelist_refresh_interval_sec,
            ));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                ticker.tick().await;
                _win.update_active_nodelist().await;
            }
        });

        // schedule a background task that refresh archiver list
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(
                _configs.nodelist_refresh_interval_sec * 2,
            ));
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                ticker.tick().await;
                Arc::clone(&_archivers).discover().await;
            }
        });
    }

    println!("Waiting for active nodelist...");
    loop {
        if network.active_nodelist.load().len() > 0 {
            break;
        }
        // Prevent busy-wait CPU spinning
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    let config = Arc::new(_configs);

    let server_stats = Arc::new(Stats {
        stream_count: Arc::new(AtomicUsize::new(0)),
    });

    let _stats = Arc::clone(&server_stats);

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(tokio::time::Duration::from_millis(500));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            ticker.tick().await;
            print!(
                "\rActive Connection Streams: {:<3}",
                _stats
                    .stream_count
                    .load(std::sync::atomic::Ordering::Relaxed)
            );
        }
    });

    let win_http = Arc::clone(&network);
    let config_http = Arc::clone(&config);
    let server_stats_http = Arc::clone(&server_stats);
    let tls_http = tls_acceptor.clone();

    let pid = std::process::id();
    println!("PID: {}", pid);

    let sock_map = Arc::new(RwLock::new(HashMap::new()));
    let subscription_manager = Arc::new(subscription::Manager::new(
        sock_map.clone(),
        win_http.clone(),
    ));

    let manager_for_http = Arc::clone(&subscription_manager);
    let http_listener_task = tokio::spawn(async move {
        http::listen(
            win_http,
            manager_for_http,
            config_http,
            server_stats_http,
            tls_http,
        )
        .await;
    });

    let ws_config = Arc::clone(&config);
    let ws_server_stats = Arc::clone(&server_stats);
    let ws_tls = tls_acceptor.clone();
    let ws_win = Arc::clone(&network);

    let websocket_listener_task = tokio::spawn(async move {
        ws::listen(
            ws_win,
            subscription_manager,
            sock_map,
            ws_config,
            ws_server_stats,
            ws_tls,
        )
        .await;
    });

    tokio::try_join!(http_listener_task, websocket_listener_task)?;

    Ok(())
}
