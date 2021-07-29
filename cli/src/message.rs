use serde::{Serialize,Desrialize};

pub use luwu::processors::{Message,MessageStep};

impl Message {
    fn new(server: String, gid: String) -> Message {
	}

    // Add a step
    fn add(&mut self, callback: String, payload: String) {
	    debug!("message {} add {} {:?}", self.gid, back, paylaod);
	    // step = MsgStep{
	    // 	Action: action,
	    // 	Data:   common.MustMarshalString(postData),
	    // }
	    // s.Steps = append(s.Steps, step)
	    // return s
    }

    // Submit submit the msg
    async fn submit(&mut self) -> Result<(), ()> {
	    debug!("committing {} body: {:?}", s.Gid, &s.MsgData);
        let cli = reqwest::Client::new();
        // cli.post();
	    // let resp = common.RestyClient.R().SetBody(&s.MsgData).Post(fmt.Sprintf("%s/submit", s.Server))
	    // return CheckDtmResponse(resp, err)
        Ok(())
    }

    // prepare the message
    async fn prepare(&mut self, query_prepared: string) -> Result<(), ()> {
	    // s.QueryPrepared = common.OrString(queryPrepared, s.QueryPrepared):
	    debug!("preparing {} body: {}", self.gid, &self.paylaod);
	    // resp, err := common.RestyClient.R().SetBody(&s.MsgData).Post(fmt.Sprintf("%s/prepare", s.Server))
	    // return CheckDtmResponse(resp, err)
        Ok(())
    }
}
