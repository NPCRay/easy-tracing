#[cfg(feature = "tracing-reqwest")]
pub mod reqwest {
    use http::Extensions;
    use opentelemetry::global;
    use opentelemetry_http::HeaderInjector;
    use reqwest_middleware::Middleware;
    use tracing::Span;
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    pub struct ReqwestTraceMiddleware();

    #[async_trait::async_trait]
    impl Middleware for ReqwestTraceMiddleware {
        async fn handle(
            &self,
            mut req: reqwest::Request,
            extensions: &mut Extensions,
            next: reqwest_middleware::Next<'_>,
        ) -> reqwest_middleware::Result<reqwest::Response> {
            let context = Span::current().context();
            global::get_text_map_propagator(|propagator| {
                propagator.inject_context(&context, &mut HeaderInjector(req.headers_mut()))
            });
            next.run(req, extensions).await
        }
    }
}

#[cfg(feature = "tracing-axum")]
pub mod axum {
    use crate::TRACER;
    use axum::extract::Request;
    use axum::middleware::Next;
    use axum::response::Response;
    use opentelemetry::trace::{TraceContextExt, Tracer};
    use opentelemetry::{Context, global};
    use opentelemetry_http::{HeaderExtractor, HeaderInjector};
    use tracing::{Instrument, error_span};
    use tracing_opentelemetry::OpenTelemetrySpanExt;

    pub async fn axum_tracing_middleware(request: Request, next: Next) -> Response {
        let context = global::get_text_map_propagator(|propagator| {
            propagator.extract(&HeaderExtractor(request.headers()))
        });

        let span = if context.has_active_span() {
            TRACER.start_with_context("http middleware", &context)
        } else {
            TRACER.start("http middleware")
        };

        let context = Context::current_with_span(span);

        let span = error_span!("http_request");
        span.set_parent(context.clone());

        let mut response = next.run(request).instrument(span).await;

        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&context, &mut HeaderInjector(response.headers_mut()))
        });
        response
    }
}
