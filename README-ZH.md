# easy-tracing
这个crate的主要目的是为Rust程序提供简单易用的链路追踪功能。
之前在Rust程序中实现链路追踪，需要引入一个外部的链路追踪库，比如opentelemetry，tracing-subscriber等，然后按照文档进行配置，非常麻烦。
这个crate提供了一种简单的方式，通过在Rust程序快速的实现链路追踪。

## 使用方法
```rust
easy_tracing::init(
    "app-name",
    "INFO",
    easy_tracing::LogFormat::Line,
    Some("127.0.0.1:4317"),
);
```
其中，app-name是应用程序的名称，INFO是日志的级别，Line/Json是日志的格式，127.0.0.1:4317是链路追踪的OTLP gRPC 服务的地址。

## 日志(log)
日志是使用tracing-subscriber库进行实现。初始化后可以直接通过tracing::info!()、tracing::error!()等宏进行日志输出，支持日志格式为Line和Json。

## 追踪(trace)
链路追踪是使用OpenTelemetry SDK进行实现， 仅支持OTLP gRPC协议进行输出，并且默认日志中会输出链路追踪信息。对于需要特殊打点的地方请使用`#[tracing::instrument]`进行追踪。

## 度量(metric)
默认不支持metric手动打点，推荐使用 [spanmetricsconnector](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/connector/spanmetricsconnector/README.md) 将span数据转为metric数据进行观测而不是手动打点。