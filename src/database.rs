use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use crate::config::Config;
use crate::errors;

#[derive(Deref, DerefMut)]
pub struct DatabaseManager(quaint::pooled::Quaint);

#[rocket::async_trait]
impl Fairing for DatabaseManager {
    fn info(&self) -> Info {
        Info {
            name: "Database connection manager.",
            kind: Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &rocket::Rocket<rocket::Orbit>) {
        println!("-------> config: {:?}", rocket.state::<Config>());
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
            request::Outcome::Success(manager) => match manager.check_out().await {
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
