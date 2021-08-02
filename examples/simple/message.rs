// use luwu_cli as lw;

/*
async fn fire_request() -> Result<(), String> {
    debug!("transaction begin.");
    req := &TransReq{Amount: 30}
    msg := dtmcli.NewMsg(DtmServer, dtmcli.MustGenGid(DtmServer)).
        Add(Busi+"/TransOut", req).
        Add(Busi+"/TransIn", req)
    err := msg.Prepare(Busi + "/TransQuery")
    e2p(err)
    logrus.Printf("busi trans submit")
    err = msg.Submit()
    e2p(err)
    return msg.Gid
}
*/

fn main() {
    println!("Hello, world!");
}
