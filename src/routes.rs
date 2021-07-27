/*
engine.POST("/api/dtmsvr/prepare", common.WrapHandler(prepare))
    engine.POST("/api/dtmsvr/submit", common.WrapHandler(submit))
    engine.POST("/api/dtmsvr/registerXaBranch", common.WrapHandler(registerXaBranch))
    engine.POST("/api/dtmsvr/registerTccBranch", common.WrapHandler(registerTccBranch))
    engine.POST("/api/dtmsvr/abort", common.WrapHandler(abort))
    engine.GET("/api/dtmsvr/query", common.WrapHandler(query))
    engine.GET("/api/dtmsvr/newGid", common.WrapHandler(newGid))
*/
use rocket::Request;
use rocket_versioning::Versioning;

use rocket::request::{self, Request, FromRequest};

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Transaction {
    type Error = MyError;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
    }
}

impl FromRequest for Transaction {
    //
    data := M{}
	b, err := c.GetRawData()
	e2p(err)
	common.MustUnmarshal(b, &data)
	logrus.Printf("creating trans in prepare")
	if data["steps"] != nil {
		data["data"] = common.MustMarshalString(data["steps"])
	}
	m := TransGlobal{}
	common.MustRemarshal(data, &m)
	return &m
}

#[get(prepare)]
fn prepare(_v: Versioning<1, 0>, req: &'r Request) -> &str {
    t := TransFromContext(c);
	t.Status = "prepared"
	t.saveNew(dbGet())
	M{"dtm_result": "SUCCESS", "gid": t.Gid}, nil
}
