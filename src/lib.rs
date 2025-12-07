//!
//!## easy Usage
//!
//!```rust
//! easy_tracing::init(
//!     "app-name",
//!     "INFO",
//!     easy_tracing::LogFormat::Line,
//!     Some("127.0.0.1:4317"),
//! );
//! ```
//! ## Logging
//!
//! Logging is implemented using the `tracing-subscriber` library. After initialization, you can directly output logs through macros like `tracing::info!()` and `tracing::error!()`. Both Line and Json log formats are supported.
//!
//! ## Tracing
//!
//! Distributed tracing is implemented using the OpenTelemetry SDK. It only supports output via the OTLP gRPC protocol, and by default, trace information will be included in the logs.
//!
//! ## Metrics
//!
//! Manual metric instrumentation is not supported by default. It is recommended to use [spanmetricsconnector](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/connector/spanmetricsconnector/README.md) to convert span data to metric data for observation rather than manual instrumentation.

mod http;
mod queue;
mod scheduler;

///```rust
/// let client = ClientBuilder::new(reqwest_client)
///    .with(LoggingMiddleware)
///    .build();
///
///  let resp = client.get("https://xxxxx.com").send().await.unwrap();
/// ```
#[cfg(feature = "tracing-reqwest")]
pub use http::reqwest::ReqwestTraceMiddleware;

///```rust
/// Router::new().route_layer((
///             middleware::from_fn(easy_tracing::axum_tracing_middleware),
///         ))
/// ```
#[cfg(feature = "tracing-axum")]
pub use http::axum::axum_tracing_middleware;

///```rust
/// easy_tracing::scheduler_tracing(|| {println!("Hello World")}).await;
/// ```
#[cfg(feature = "tracing-scheduler")]
pub use scheduler::scheduler_tracing;

///```rust
///  let result = easy_tracing::queue_consumer_tracing(|t| {println!(t)}, "Hello World").await;
/// ```
#[cfg(feature = "tracing-consumer")]
pub use queue::consume as queue_consumer_tracing;

use opentelemetry::global;
use opentelemetry::global::BoxedTracer;
use opentelemetry::trace::{TraceContextExt, TracerProvider};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{Sampler, SdkTracerProvider};
use serde_json::{json, Map, Number, Value};
use std::sync::{LazyLock, OnceLock};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::fmt::format::{JsonFields, Writer};
use tracing_subscriber::fmt::{FmtContext, FormatEvent, FormatFields};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

static APP_NAME: OnceLock<String> = OnceLock::new();

static TRACER: LazyLock<BoxedTracer> =
    LazyLock::new(|| global::tracer(APP_NAME.get().unwrap().as_str()));

// init tracing
//
pub fn init(app_name: &str, log_level: &str, log_format: LogFormat, otlp_endpoint: Option<&str>) {
    APP_NAME.set(app_name.to_string()).unwrap();
    global::set_text_map_propagator(TraceContextPropagator::new());
    let mut provider_builder = SdkTracerProvider::builder();
    match otlp_endpoint {
        None => {}
        Some(endpoint) => {
            let exporter = SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build();
            match exporter {
                Ok(exporter) => {
                    provider_builder = provider_builder
                        .with_batch_exporter(exporter)
                        .with_sampler(Sampler::AlwaysOn);
                }
                Err(err) => {
                    tracing::error!("Failed to create OTLP exporter: {:?}", err);
                }
            }
        }
    }
    let provider = provider_builder.build();
    let tracer = provider.tracer(app_name.to_string());
    global::set_tracer_provider(provider);

    let telemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_error_records_to_exceptions(true);
    let filter_layer = EnvFilter::builder()
        .with_default_directive(log_level.parse().unwrap())
        .from_env_lossy();
    let registry = tracing_subscriber::registry()
        .with(telemetry_layer)
        .with(filter_layer);

    match log_format {
        LogFormat::Json => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .event_format(JsonTraceIdFormatter)
                .fmt_fields(JsonFields::default());
            registry.with(fmt_layer).init();
        }
        LogFormat::Line => {
            registry.with(tracing_subscriber::fmt::layer()).init();
        }
    }
}

pub enum LogFormat {
    Json,
    Line,
}

struct JsonTraceIdFormatter;

impl<S, N> FormatEvent<S, N> for JsonTraceIdFormatter
where
    S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &tracing::Event<'_>,
    ) -> std::fmt::Result {
        // get trace id
        let context = Span::current().context();

        let trace_id = context.span().span_context().trace_id().to_string();
        let span_id = context.span().span_context().span_id().to_string();

        // collect fields as serde-json Value
        let mut map = Map::new();
        let mut visitor = SerdeMapVisitor { map: &mut map };

        // manual gather message (from event)
        event.record(&mut visitor);

        // insert trace id
        map.insert("trace_id".to_string(), json!(trace_id));
        // insert span id
        map.insert("span_id".to_string(), json!(span_id));
        // insert level
        let lvl = event.metadata().level().to_string();
        map.insert("level".to_string(), json!(lvl));
        // insert target
        map.insert(
            "target".to_string(),
            json!(event.metadata().target().to_string()),
        );
        // insert line number
        map.insert(
            "line_number".to_string(),
            json!(event.metadata().line().unwrap_or(0)),
        );

        // insert timestamp
        map.insert(
            "timestamp".to_string(),
            json!(chrono::Utc::now().to_rfc3339()),
        );

        // output as JSON
        let v: Value = Value::Object(map);
        writeln!(&mut writer, "{}", v)
    }
}

struct SerdeMapVisitor<'a> {
    map: &'a mut Map<String, Value>,
}

impl<'a> tracing::field::Visit for SerdeMapVisitor<'a> {
    fn record_f64(&mut self, field: &tracing::field::Field, value: f64) {
        let n = Number::from_f64(value).unwrap_or_else(|| Number::from(0));
        self.map.insert(field.name().to_string(), Value::Number(n));
    }
    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.map
            .insert(field.name().to_string(), Value::Number(Number::from(value)));
    }
    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.map
            .insert(field.name().to_string(), Value::Number(Number::from(value)));
    }
    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.map
            .insert(field.name().to_string(), Value::Bool(value));
    }
    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        self.map
            .insert(field.name().to_string(), Value::String(value.to_string()));
    }
    fn record_error(
        &mut self,
        field: &tracing::field::Field,
        value: &(dyn std::error::Error + 'static),
    ) {
        self.map
            .insert(field.name().to_string(), Value::String(value.to_string()));
    }
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.map.insert(
            field.name().to_string(),
            Value::String(format!("{:?}", value)),
        );
    }
}
