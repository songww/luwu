use std::io::Cursor;

#[cfg(feature = "msgpack")]
use rmps;
use rocket::http::{ContentType, MediaType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder, Response};
use rocket::serde::Serialize;
#[cfg(feature = "json")]
use serde_json;

pub struct DynResponse<C>
where
    C: Serialize,
{
    content: C,
    media_type: Option<MediaType>,
    status: Status,
}

impl<C> DynResponse<C>
where
    C: Serialize,
{
    pub fn new(content: C) -> DynResponse<C> {
        DynResponse {
            content,
            media_type: None,
            status: Status::Ok,
        }
    }

    /*
    #[cfg(feature = "json")]
    fn json(content: C) -> DynResponse<C> {
        DynResponse {
            content,
            media_type: Some(MediaType::JSON),
            status: Status::Ok,
        }
    }

    #[cfg(feature = "msgpack")]
    fn msgpack(content: C) -> DynResponse<C> {
        DynResponse {
            content,
            media_type: Some(MediaType::MsgPack),
            status: Status::Ok,
        }
    }

    #[cfg(feature = "bincode")]
    fn bincode(content: C) -> DynResponse<C> {
        DynResponse {
            content,
            media_type: Some(MediaType::Binary),
            status: Status::Ok,
        }
    }

    #[cfg(feature = "xml")]
    fn xml(content: C) -> DynResponse<C> {
        DynResponse {
            content,
            media_type: Some(MediaType::XML),
            status: Status::Ok,
        }
    }
    */

    #[cfg(feature = "json")]
    pub fn json(&mut self) {
        self.media_type.replace(MediaType::JSON);
    }

    #[cfg(feature = "msgpack")]
    pub fn msgpack(&mut self) {
        self.media_type.replace(MediaType::MsgPack);
    }

    #[cfg(feature = "bincode")]
    pub fn bincode(&mut self) {
        self.media_type.replace(MediaType::Binary);
    }

    pub fn status(&mut self, status: Status) {
        self.status = status
    }
}

#[rocket::async_trait]
impl<'r, C> Responder<'r, 'static> for DynResponse<C>
where
    C: Serialize,
{
    fn respond_to(self, request: &'r Request<'_>) -> response::Result<'static> {
        let default_media_type = MediaType::JSON;
        let media_type = self
            .media_type
            .as_ref()
            .or(request
                .accept()
                .and_then(|accept| accept.media_types().next()))
            .unwrap_or(&default_media_type);
        error!("request accept {:?}", request.accept());
        println!("request accept {:?}", request.accept().unwrap().preferred());
        let body = if media_type.is_json() {
            serde_json::to_vec(&self.content).unwrap()
        } else if media_type.is_msgpack() {
            rmps::to_vec(&self.content).unwrap()
        } else {
            panic!("Unsupported content type: {:?}", &media_type);
        };
        Response::build()
            .header(ContentType(media_type.clone()))
            .status(self.status)
            .sized_body(body.len(), Cursor::new(body))
            .ok()
    }
}

#[derive(Debug, Serialize)]
struct TransactionPreparationResponse {
    code: Code,
    gid: usize,
}

#[derive(Debug, Serialize)]
#[repr(u16)]
pub enum Code {
    Ok = 20000,
}
