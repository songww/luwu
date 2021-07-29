use once_cell::sync::OnceCell;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use crate::config::Config;
use crate::errors;

static POOL: OnceCell<quaint::pooled::Quaint> = OnceCell::new();

pub struct DatabaseManager;

#[rocket::async_trait]
impl Fairing for DatabaseManager {
    fn info(&self) -> Info {
        Info {
            name: "Database connection manager.",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &rocket::Rocket<rocket::Orbit>) {
        let config = rocket.state::<Config>().unwrap();
        let mut builder = quaint::pooled::Quaint::builder(config.database_url()).expect("Can not connect to database.");
        builder.test_on_check_out(true);
        POOL.set(builder.build()).ok();
    }
}

pub type Conn = quaint::pooled::PooledConnection;

#[derive(Deref, DerefMut, AsRef, AsMut, Into)]
#[into(owned, ref, ref_mut)]
pub struct DB(Conn);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for DB {
    type Error = errors::Error;
    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let outcome = request.guard::<&rocket::State<DatabaseManager>>().await;
        match outcome {
            request::Outcome::Success(_) => match POOL.get().unwrap().check_out().await {
                Ok(conn) => request::Outcome::Success(DB(conn)),
                Err(err) => request::Outcome::Failure((Status::ServiceUnavailable, err.into())),
            },
            request::Outcome::Forward(forward) => request::Outcome::Forward(forward),
            request::Outcome::Failure((status, _)) => {
                request::Outcome::Failure((status, errors::Error::DBNotAvailable))
            }
        }
    }
}
