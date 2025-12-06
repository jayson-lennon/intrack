use clap_verbosity_flag::{Verbosity, WarnLevel};
use std::env;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

pub fn init(verbosity: Verbosity<WarnLevel>) {
    let filter = match env::var("RUST_LOG") {
        // Use RUST_LOG if found
        Ok(filter_str) => filter_str,
        // Otherwise use this fallback
        Err(_) => format!("intrack={verbosity}"),
    };

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_filter(EnvFilter::new(filter)))
        .init();
}
