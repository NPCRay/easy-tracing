# easy-tracing

This crate aims to provide simple and easy-to-use distributed tracing functionality for Rust programs.

Previously, implementing distributed tracing in Rust programs required introducing external tracing libraries such as `opentelemetry` and `tracing-subscriber`, then configuring them according to documentation, which was very cumbersome.

This crate provides a simple way to quickly implement distributed tracing in Rust programs.

## Usage

```rust
easy_tracing::init(
    "app-name",
    "INFO",
    easy_tracing::LogFormat::Line,
    Some("127.0.0.1:4317"),
);
```


Where:
- `app-name` is the application name
- `INFO` is the log level
- `Line`/`Json` is the log format
- `127.0.0.1:4317` is the address of the OTLP gRPC tracing service

## Logging

Logging is implemented using the `tracing-subscriber` library. After initialization, you can directly output logs through macros like `tracing::info!()` and `tracing::error!()`. Both Line and Json log formats are supported.

## Tracing

Distributed tracing is implemented using the OpenTelemetry SDK. It only supports output via the OTLP gRPC protocol, and by default, trace information will be included in the logs.

## Metrics

Manual metric instrumentation is not supported by default. It is recommended to use [spanmetricsconnector](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/connector/spanmetricsconnector/README.md) to convert span data to metric data for observation rather than manual instrumentation.
 