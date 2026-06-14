use cyberos_obs_sdk::red;
use cyberos_obs_sdk::red_instrument;

#[red_instrument(service = "macro-test", route = "/macro")]
async fn macro_handler(value: u16) -> Result<u16, &'static str> {
    if value == 0 {
        return Err("bad");
    }
    Ok(value + 1)
}

#[tokio::test]
async fn macro_preserves_handler_signature() {
    red::reset_for_tests();
    let value: Result<u16, &'static str> = macro_handler(41).await;
    assert_eq!(value, Ok(42));
    assert_eq!(
        red::snapshot().counter_value(
            red::REQUESTS_TOTAL,
            &[
                ("service", "macro-test"),
                ("route", "/macro"),
                ("tenant_id", "unknown"),
                ("status_class", "2xx"),
            ],
        ),
        1
    );
}
