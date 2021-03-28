use tracing::subscriber::DefaultGuard;

/// Utility function to initialize logging in the test environment.
/// Note that you have to keep the `_guard` in scope after calling in test:
///
/// ```rust
/// let _guard = init_tracing();
/// ```
pub fn init_tracing() -> DefaultGuard {
    let global_filter = tracing::Level::WARN;
    let test_filter = tracing::Level::DEBUG;
    let monero_harness_filter = tracing::Level::DEBUG;
    let monero_rpc_filter = tracing::Level::DEBUG;

    use tracing_subscriber::util::SubscriberInitExt as _;
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "{},test={},monero_harness={},monero_rpc={}",
            global_filter, test_filter, monero_harness_filter, monero_rpc_filter,
        ))
        .set_default()
}
