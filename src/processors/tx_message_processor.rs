use std::collections::HashMap;

use quaint::pooled::PooledConnection;
use rocket::serde::uuid::Uuid;
use serde::Serialize;

use crate::config::CONFIG;
use crate::errors;
use crate::models::transaction::{State, Transaction, TransactionBranch};

use super::{Message, MessageStep, Processor};

type Conn = PooledConnection;

#[derive(Debug)]
pub struct TxMessageProcessor<'tx> {
    tx: &'tx mut Transaction,
}

#[async_trait]
impl<'tx> Processor<'tx> for TxMessageProcessor<'tx> {
    fn with_transaction(tx: &'tx mut Transaction) -> Box<dyn Processor<'tx> + Send + 'tx>
    where Self: Sized {
        Box::new(TxMessageProcessor { tx })
    }

    fn branches(&self) -> Vec<TransactionBranch> {
        let steps: Vec<MessageStep> = serde_json::from_str(self.tx.payload()).unwrap();
        let mut branches = Vec::with_capacity(3);
        for step in steps {
            branches.push(TransactionBranch::new(
                self.tx.gid(),
                uuid::Uuid::new_v4(),
                "action".to_string(),
                State::Prepared,
                step.callback.to_string(),
                step.payload.to_string(),
            ));
        }
        branches
    }

    async fn exec(&mut self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
        let cli = reqwest::Client::new();
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
        let config = CONFIG.get().unwrap();
        self.tx.touch(db, config.delay).await?;
        Ok(())
    }

    async fn once(&mut self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
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
                    // return Err("Message go pass all branch".to_string());
                }
            }
        }
        self.tx.update_state(db, State::Succeed).await?;
        Ok(())
    }
}

impl<'a> TxMessageProcessor<'a> {
    async fn maybe_query_prepared(&mut self, db: &Conn) -> Result<bool, errors::Error> {
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
        dbg!(&resp);
        // resp, err := common.RestyClient.R().SetQueryParam("gid", t.Gid).Get(t.QueryPrepared)
        // body := resp.String()
        // if strings.Contains(body, "SUCCESS") {
        // 	t.changeStatus(db, "submitted")
        // } else {
        // 	t.touch(db, t.NextCronInterval*2)
        // }
        let config = CONFIG.get().unwrap();
        self.tx.touch(db, config.delay).await?;
        Ok(true)
    }
}
