#![feature(const_generics_defaults)]

#[macro_use]
extern crate rocket;

use std::io::Cursor;

#[cfg(feature = "msgpack")]
use rmps;
use rocket::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::{ContentType, MediaType, Status};
use rocket::request::{self, FromRequest, Request};
#[cfg(feature = "msgpack")]
use rocket::response::content::MsgPack;
#[cfg(feature = "json")]
use rocket::response::content::Json;
use rocket::response::{self, Responder, Response};
use rocket::serde::{Deserialize, Serialize};
use rocket::tokio::time::Instant;
#[cfg(feature = "json")]
use serde_json;
use tracing::{error, span};
// use tracing_futures::WithSubscriber;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;

use luwu::responder::DynResponse;

#[cfg(feature = "telemetry")]
fn telemetry() -> tracing_opentelemetry::OpenTelemetryLayer<
    tracing_subscriber::Registry,
    opentelemetry::sdk::trace::Tracer,
> {
    use opentelemetry::sdk::export::trace::stdout;

    let tracer = stdout::new_pipeline().install_simple();

    // Create a tracing layer with the configured tracer
    tracing_opentelemetry::layer().with_tracer(tracer)
}

fn enable_tracing() {
    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = tracing_subscriber::registry();
    #[cfg(feature = "telemetry")]
    let subscriber = subscriber.with(telemetry());
    #[cfg(feature = "env-filter")]
    let subscriber = subscriber.with(tracing_subscriber::EnvFilter::new("INFO"));

    let file_appender = tracing_appender::rolling::daily("logs", "tracing.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    let subscriber = subscriber.with(tracing_subscriber::fmt::layer().with_writer(non_blocking));

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[derive(Debug, Serialize)]
struct Pong {
    message: &'static str,
}

#[get("/ping")]
fn index<'r>() -> DynResponse<Pong> {
    DynResponse::new(Pong { message: "Pong" })
}

#[derive(Debug)]
struct RequestTimer;
#[derive(Copy, Clone, Debug)]
struct RequestAt(Option<Instant>);

#[rocket::async_trait]
impl Fairing for RequestTimer {
    // This is a request and response fairing named "GET/POST Counter".
    fn info(&self) -> Info {
        Info {
            name: "Request timer.",
            kind: Kind::Request | Kind::Response
        }
    }

    // Increment the counter for `GET` and `POST` requests.
    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        // Store a `TimerStart` instead of directly storing a `Instant`
        // to ensure that this usage doesn't conflict with anything else
        // that might store a `SystemTime` in request-local cache.
        request.local_cache(|| RequestAt(Some(Instant::now())));
        let uri = request.uri();
        info!("begin {} {} query: {} body: ", request.method(), uri.path(), uri.query().map(|q|q.as_str()).unwrap_or(""))
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let request_at = req.local_cache(|| RequestAt(None));
        if let Some(duration) = request_at.0.map(|at| at.elapsed()) {
            let ms = duration.as_millis();
            res.set_raw_header("X-Response-Time", format!("{} ms", ms));
            let uri = req.uri();
            info!("used {} ms {} {} query: {:?} body: ", ms, req.method(), uri.path(), uri.query());
        }
    }
}

#[launch]
fn rocket() -> _ {
    enable_tracing();
    rocket::build().mount("/api/", routes![index]).attach(RequestTimer)
}
