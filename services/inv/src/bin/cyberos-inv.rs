//! cyberos-inv — Wise webhook HTTP host (TASK-INV-012).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use cyberos_inv::wise::{
    router, CachedPublicKeys, StaticPemSource, WiseHostState, WiseProcessor, WiseWalQueue,
};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pem = std::env::var("WISE_PUBLIC_KEY_PEM").unwrap_or_else(|_| {
        eprintln!("cyberos-inv: WISE_PUBLIC_KEY_PEM unset — using empty PEM (verify will 401)");
        String::new()
    });
    let listen = std::env::var("INV_LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:7710".into());
    let addr: SocketAddr = listen
        .parse()
        .unwrap_or_else(|e| panic!("invalid INV_LISTEN_ADDR {listen}: {e}"));

    let keys = Arc::new(CachedPublicKeys::new(StaticPemSource::new(pem)));
    let wal = Arc::new(Mutex::new(WiseWalQueue::default()));
    let processor = Arc::new(WiseProcessor::new(Arc::clone(&wal)));
    let state = Arc::new(WiseHostState {
        keys,
        wal: Arc::clone(&wal),
        processor: Arc::clone(&processor),
    });

    let proc = Arc::clone(&processor);
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(std::time::Duration::from_millis(50));
        loop {
            tick.tick().await;
            proc.drain_all();
        }
    });

    let app = router(state);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("bind {addr}: {e}"));
    eprintln!("cyberos-inv listening on {addr}");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| panic!("serve: {e}"));
}
