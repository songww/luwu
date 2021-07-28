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
            |tx: &Transaction| -> Box<dyn Processor> {
                return &TxTCCProcessor { tx };
            }
            .into(),
        );
    }
}

#[async_trait]
impl<'tx> Processor for TxTCCProcessor<'tx> {
    fn branches() -> Vec<TransactionBranch> {
        Vec::new()
    }

    async fn exec(&self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
        let cli = reqwest::Client::new();
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
        let r#type = match self.tx.state {
            State::succeed | State::Failed => {
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
        let state = match self.tx.state {
            State::Submitted => State::Succeed,
            _ => State::Failed,
        };
        // 已全部处理完
        self.tx.update_state(db, state);
    }
}
