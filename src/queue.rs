#[cfg(feature = "tracing-consumer")]
pub async fn consume<F, Fut, T>(handler: F, message: T) -> anyhow::Result<()>
where
    F: Fn(T) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<()>> + Send + 'static,
{
    use crate::TRACER;
    use opentelemetry::Context;
    use opentelemetry::trace::{TraceContextExt, Tracer};
    use tracing::{Instrument, error_span};
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    // todo parse trace info form message header
    let span = TRACER.start("queue consumer");
    let context = Context::current_with_span(span);
    let span = error_span!("scheduler");
    span.set_parent(context.clone());
    handler(message).instrument(span).await
}
