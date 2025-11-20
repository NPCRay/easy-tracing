#[cfg(feature = "tracing-scheduler")]
pub async fn scheduler_tracing<F, Fut>(action: F)
where
    F: Send + Sync + 'static + Fn() -> Fut + Clone,
    Fut: Future<Output = ()> + Send + 'static,
{
    use crate::TRACER;
    use opentelemetry::Context;
    use opentelemetry::trace::{TraceContextExt, Tracer};
    use tracing::{Instrument, error_span};
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    let span = TRACER.start("http middleware");
    let context = Context::current_with_span(span);

    let span = error_span!("scheduler",);
    span.set_parent(context.clone());
    action().instrument(span).await;
}
