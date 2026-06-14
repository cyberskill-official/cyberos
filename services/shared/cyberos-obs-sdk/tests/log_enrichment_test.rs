use cyberos_obs_sdk::logging::{request_span, ObsContextLayer};
use cyberos_obs_sdk::red::{self, LOG_ENRICHMENT_TOTAL};
use tracing_subscriber::prelude::*;

#[test]
fn obs_context_layer_counts_enriched_log_events() {
    red::reset_for_tests();
    let subscriber = tracing_subscriber::registry().with(ObsContextLayer::new("auth-service"));

    tracing::subscriber::with_default(subscriber, || {
        let span = request_span(
            "auth-service",
            "/v1/auth/token",
            "tenant-a",
            "4bf92f3577b34da6a3ce929d0e0e4736",
            "00f067aa0ba902b7",
        );
        let _entered = span.enter();
        tracing::info!("issued token");
    });

    assert_eq!(
        red::snapshot().counter_value(LOG_ENRICHMENT_TOTAL, &[("service", "auth-service")]),
        1
    );
}
