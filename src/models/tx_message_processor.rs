use std::collections::HashMap;

use quaint::pooled::PooledConnection;

use crate::errors;

use super::transaction::{add_processor_creator, Processor, State, Transaction, TransactionBranch};

type Conn = PooledConnection;

struct TxMessageProcessor<'tx> {
    tx: &'tx Transaction,
}

impl<'tx> TxMessageProcessor<'tx> {
    pub fn regist() {
        add_processor_creator(
            "message".to_string(),
            |tx: &Transaction| -> Box<dyn Processor> {
                return &TxMessageProcessor { tx };
            }
            .into(),
        )
    }
}

#[async_trait]
impl<'tx> Processor for TxMessageProcessor<'tx> {
    fn branches(&self) -> Vec<TransactionBranch> {
        let steps: Vec<HashMap<String, String>> = serde_json::from_str(self.tx.message()).unwrap();
        let mut branches = Vec::with_capacity(steps.len());
        for step in steps {
            branches.push(TransactionBranch::new(
                self.tx.gid(),
                uuid::Uuid::new_v4(),
                "action".to_string(),
                State::Prepared,
                step.get("action").unwrap().to_string(),
                step.get("message").unwrap().to_string(),
            ));
        }
        branches
    }

    async fn exec(&self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
        let mut cli = reqwest::Client::new();
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert("content-type", "application/json");
        headers.insert("accept", "application/json");
        let resp = cli
            .post(branch.url())
            .body(branch.message())
            .headers(headers)
            .send()
            .await?
            .json::<HashMap<String, serde_json::Value>>();
        dbg!(&resp);
        /*
        body := resp.String()
        if strings.Contains(body, "SUCCESS") {
            branch.changeStatus(db, "succeed")
            t.touch(db, config.TransCronInterval)
        } else {
            panic(fmt.Errorf("unknown response: %s, will be retried", body))
        }
        */
        Ok(())
    }

    async fn once(&self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
        self.maybe_query_prepared(db).await?;
        match self.tx.state() {
            State::Submitted => {
                // Only accept this.
            }
            _ => {
                return Ok(());
            }
        }
        for branch in branches {
            // if branch.BranchType != "action" || branch.Status != "prepared" {
            match (branch.r#type(), branch.state()) {
                ("action", State::Prepared) => {
                    //
                }
                _ => {
                    continue;
                }
            }
            self.exec(db, branch).await?;
            match branch.state() {
                State::Succeed => {
                    // break;
                }
                _ => {
                    return Err("Message go pass all branch".to_string());
                }
            }
        }
        self.tx.update_state(db, State::Succeed).await?;
        Ok(())
    }
}

impl<'a> TxMessageProcessor<'a> {
    async fn maybe_query_prepared(&self, db: &Conn) -> Result<bool, ()> {
        match self.tx.state() {
            State::Prepared => {
                // Only accept this.
            }
            _ => {
                return Ok(false);
            }
        }
        let cli = reqwest::Client::new();
        let resp = cli
            .get(self.tx.query_prepared)
            .query(&[("gid", self.gid)])
            .send()
            .await?;
        // resp, err := common.RestyClient.R().SetQueryParam("gid", t.Gid).Get(t.QueryPrepared)
        // body := resp.String()
        // if strings.Contains(body, "SUCCESS") {
        // 	t.changeStatus(db, "submitted")
        // } else {
        // 	t.touch(db, t.NextCronInterval*2)
        // }
        Ok(true)
    }
}
