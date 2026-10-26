use arc_swap::ArcSwap;
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::RwLock,
};
use win_proxy::{config::Config, crypto::ShardusCrypto, http, subscription, win};

/// Exercise the complete HTTP dispatcher, including route selection and body framing.
async fn forward_request(request: Vec<u8>, response_body: &'static str) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let expected_request = request.clone();
    let upstream = async move {
        let (mut stream, _) = listener.accept().await.unwrap();
        let mut received = Vec::new();
        http::collect_http(&mut stream, &mut received)
            .await
            .unwrap();
        assert_eq!(
            received, expected_request,
            "WIN requests must remain unchanged"
        );
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream.write_all(response.as_bytes()).await.unwrap();
    };

    let config = Arc::new(Config::load_from("src/config.json").unwrap());
    let network = Arc::new(win::WinCoin::new(
        Arc::new(ShardusCrypto::new(&config.crypto_seed)),
        Arc::new(ArcSwap::from_pointee(Vec::new())),
        (*config).clone(),
    ));
    network.active_nodelist.store(Arc::new(vec![win::Consensor {
        id: "local-win-validator".into(),
        ip: "127.0.0.1".into(),
        port,
        publicKey: "a".repeat(64),
        foundationNode: None,
        rng_bias: None,
    }]));
    let manager = Arc::new(subscription::Manager::new(
        Arc::new(RwLock::new(HashMap::new())),
        network.clone(),
    ));
    let (mut client, proxy) = tokio::io::duplex(8192);
    let handler = http::handle_stream(proxy, network, manager, config, "127.0.0.1:0".into());
    let client_request = async {
        client.write_all(&request).await.unwrap();
        // End the input stream while retaining the response half.
        client.shutdown().await.unwrap();
        let mut response = Vec::new();
        client.read_to_end(&mut response).await.unwrap();
        assert!(response.starts_with(b"HTTP/1.1 200 OK"));
        assert_eq!(http::split_head_body(&response).1, response_body.as_bytes());
    };
    tokio::time::timeout(Duration::from_secs(5), async {
        let (_, result, _) = tokio::join!(upstream, handler, client_request);
        result.unwrap();
    })
    .await
    .expect("WIN gateway request should complete");
}

#[tokio::test]
async fn win_routes_reach_the_validator() {
    for route in [
        "/network/parameters",
        "/account/abc/balance",
        "/profiles?limit=10",
        "/profiles/alice",
        "/contracts/contract-id",
        "/skills/rust",
        "/endorsements/endorsement-id",
        "/scores/alice",
        "/dao/proposals/summary",
        "/transaction/tx-id",
        "/bridge/status",
    ] {
        forward_request(
            format!("GET {} HTTP/1.1\r\nHost: wallet.example\r\n\r\n", route).into_bytes(),
            r#"{"network":"WIN","balance":"100000000000000000000"}"#,
        )
        .await;
    }
}

#[tokio::test]
async fn inject_preserves_win_transaction_network_id_signature_and_amount() {
    // A transport fixture, deliberately never sent to a real validator.
    let transaction = serde_json::json!({
        "type": "transfer",
        "networkId": "f4ad1838b012fe4af28cfbcc01c30044e3e225ef6be7e8461343d1de369949d9",
        "amount": "100000000000000000001",
        "sign": {"owner": "test-wallet", "sig": "test-signature"}
    });
    let body = serde_json::json!({"tx": transaction.to_string()}).to_string();
    forward_request(
        format!(
            "POST /inject HTTP/1.1\r\nHost: wallet.example\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(), body
        )
        .into_bytes(),
        r#"{"result":{"success":true}}"#,
    )
    .await;
}
