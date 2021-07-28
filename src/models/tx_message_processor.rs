use std::collections::HashMap;

use quaint::pooled::PooledConnection;
use rocket::serde::uuid::Uuid;
use serde::Serialize;

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
            Box::new(|tx: &Transaction| -> Box<dyn Processor> {
                Box::new(TxMessageProcessor { tx })
            }),
        )
    }
}

#[async_trait]
impl<'tx> Processor for TxMessageProcessor<'tx> {
    fn branches(&self) -> Vec<TransactionBranch> {
        let steps: Vec<HashMap<String, String>> = serde_json::from_str(self.tx.payload()).unwrap();
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
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".try_into().unwrap(),
        );
        headers.insert(
            reqwest::header::ACCEPT,
            "application/json".try_into().unwrap(),
        );
        let resp = cli
            .post(branch.url())
            .body(branch.payload().to_string())
            .headers(headers)
            .send()
            .await?
            .json::<HashMap<String, serde_json::Value>>()
            .await?;
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
    async fn maybe_query_prepared(&self, db: &Conn) -> Result<bool, errors::Error> {
        match self.tx.state() {
            State::Prepared => {
                // Only accept this.
            }
            _ => {
                return Ok(false);
            }
        }
        let cli = reqwest::Client::new();
        #[derive(Debug, Serialize)]
        struct Q {
            gid: Uuid,
        }
        let resp = cli
            .get(self.tx.query_prepared())
            .query(&Q { gid: self.tx.gid() })
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
