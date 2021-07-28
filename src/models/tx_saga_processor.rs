use std::collections::HashMap;

use quaint::pooled::PooledConnection;

use crate::errors;

use super::transaction::{
    add_processor_creator, Gid, Processor, State, Transaction, TransactionBranch,
};

type Conn = PooledConnection;

pub struct TxSagaProcessor<'tx> {
    tx: &'tx Transaction,
}

impl<'tx> TxSagaProcessor<'tx> {
    pub fn regist() {
        add_processor_creator(
            "saga".to_string(),
            Box::new(|tx: &Transaction| -> Box<dyn Processor> { Box::new(TxSagaProcessor { tx }) }),
        )
    }
}

#[async_trait]
impl<'tx> Processor for TxSagaProcessor<'tx> {
    fn branches(&self) -> Vec<TransactionBranch> {
        let steps: Vec<HashMap<String, String>> = serde_json::from_str(self.tx.payload()).unwrap();
        let mut branches = Vec::with_capacity(steps.len());
        for (i, step) in steps.iter().enumerate() {
            // let branch_id = format!("{:02}", i + 1);
            let branch_id = Gid::new_v4();
            for r#type in ["compensate", "action"] {
                branches.push(TransactionBranch::new(
                    self.tx.gid(),
                    branch_id,
                    r#type.to_string(),
                    State::Prepared,
                    step.get(r#type).unwrap().to_string(),
                    step.get("payload").unwrap().to_string(),
                ));
            }
        }
        branches
    }

    async fn exec(&self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
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
            .query(&self.tx.branch_params(branch))
            .body(branch.payload().to_string())
            .headers(headers)
            .send()
            .await?;
        /*
        body := resp.String()
        if strings.Contains(body, "SUCCESS") {
            t.touch(db, config.TransCronInterval)
            branch.changeStatus(db, "succeed")
        } else if branch.BranchType == "action" && strings.Contains(body, "FAILURE") {
            t.touch(db, config.TransCronInterval)
            branch.changeStatus(db, "failed")
        } else {
            panic(fmt.Errorf("unknown response: %s, will be retried", body))
        }
        */
        Ok(())
    }

    async fn once(&self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
        match self.tx.state() {
            State::Submitted => {}
            _ => {
                return Ok(());
            }
        }
        let mut taken = 0;
        let mut ok = true;
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
                    taken += 1;
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok {
            self.tx.update_state(db, State::Succeed).await?;
            return Ok(());
        }
        match self.tx.state() {
            State::Aborting | State::Failed => {
                //
            }
            _ => {
                self.tx.update_state(db, State::Aborting);
            }
        };
        for branch in branches.iter().take(taken) {
            match (branch.r#type(), branch.state()) {
                ("compensate", State::Prepared) => {
                    self.exec(db, branch).await;
                }
                _ => {
                    continue;
                }
            }
        }
        self.tx.update_state(db, State::Failed).await?;
        Ok(())
    }
}
