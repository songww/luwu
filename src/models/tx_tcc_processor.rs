use std::collections::HashMap;

use quaint::pooled::PooledConnection;

use crate::errors;

use super::transaction::{add_processor_creator, Processor, State, Transaction, TransactionBranch};

type Conn = PooledConnection;

struct TxTCCProcessor<'tx> {
    tx: &'tx Transaction,
}

impl<'tx> TxTCCProcessor<'tx> {
    pub fn regist() {
        add_processor_creator(
            "tcc".to_string(),
            Box::new(|tx: &Transaction| -> Box<dyn Processor> { Box::new(TxTCCProcessor { tx }) }),
        );
    }
}

#[async_trait]
impl<'tx> Processor for TxTCCProcessor<'tx> {
    fn branches(&self) -> Vec<TransactionBranch> {
        Vec::new()
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
            .body(branch.payload().to_string())
            .headers(headers)
            .send()
            .await?
            .json::<HashMap<String, serde_json::Value>>();
        // resp, err := common.RestyClient.R().SetBody(branch.Data).SetHeader("Content-type", "application/json").SetQueryParams(t.getBranchParams(branch)).Post(branch.URL)
        /*
        body := resp.String()
        if strings.Contains(body, "SUCCESS") {
            t.touch(db, config.TransCronInterval)
            branch.changeStatus(db, "succeed")
        } else if branch.BranchType == "try" && strings.Contains(body, "FAILURE") {
            t.touch(db, config.TransCronInterval)
            branch.changeStatus(db, "failed")
        } else {
            panic(fmt.Errorf("unknown response: %s, will be retried", body))
        }
        */
        Ok(())
    }

    async fn once(&self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
        let r#type = match self.tx.state() {
            State::Succeed | State::Failed => {
                return Ok(());
            }
            State::Submitted => "confirm",
            _ => "cancel",
        };
        for branch in branches.iter().rev() {
            if branch.r#type() == r#type {
                self.exec(db, branch).await;
            }
        }
        let state = match self.tx.state() {
            State::Submitted => State::Succeed,
            _ => State::Failed,
        };
        // 已全部处理完
        self.tx.update_state(db, state);
        Ok(())
    }
}
