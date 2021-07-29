use quaint::pooled::PooledConnection;
use serde::Serialize;

use crate::errors;
use crate::models::transaction::{State, Transaction, TransactionBranch};

use super::Processor;

type Conn = PooledConnection;

#[derive(Debug)]
pub struct TxXaProcessor<'tx> {
    tx: &'tx mut Transaction,
}

#[async_trait]
impl<'tx> Processor<'tx> for TxXaProcessor<'tx> {
    fn with_transaction(tx: &'tx mut Transaction) -> Box<dyn Processor<'tx> + Send + 'tx>
    where Self: Sized {
        Box::new(TxXaProcessor { tx })
    }

    fn branches(&self) -> Vec<TransactionBranch> {
        Vec::new()
    }

    async fn exec(&mut self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
        #[derive(Debug, Serialize)]
        struct Payload {
            gid: uuid::Uuid,
            branch_id: uuid::Uuid,
            action: String,
        }
        let mut paylaod = Payload {
            branch_id: branch.branch_id(),
            gid: self.tx.gid(),
            action: match self.tx.state() {
                State::Prepared => "rollback".to_string(),
                _ => "commit".to_string(),
            },
        };
        let cli = reqwest::Client::new();
        let resp = cli.post(branch.url()).json(&paylaod).send().await?;
        /*
        body := resp.String()
        if strings.Contains(body, "SUCCESS") {
            t.touch(db, config.TransCronInterval)
            branch.changeStatus(db, "succeed")
        } else {
            panic(fmt.Errorf("bad response: %s", body))
        }
        */
        Ok(())
    }

    async fn once(&mut self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
        let r#type = match self.tx.state() {
            State::Succeed => {
                return Ok(());
            }
            State::Submitted => "commit",
            _ => "rollback",
        };
        for branch in branches {
            match (branch.r#type() == r#type, branch.state()) {
                (true, State::Succeed) => {
                    self.exec(db, branch).await;
                }
                _ => {}
            }
        }
        let state = match self.tx.state() {
            State::Submitted => State::Succeed,
            _ => State::Failed,
        };
        self.tx.update_state(db, state).await?;
        Ok(())
    }
}
