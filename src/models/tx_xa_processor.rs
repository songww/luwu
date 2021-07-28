use std::collections::HashMap;

use quaint::pooled::PooledConnection;
use serde::Deserialize;

use crate::errors;

use super::transaction::{add_processor_creator, Processor, State, Transaction, TransactionBranch};

type Conn = PooledConnection;

struct TxXaProcessor<'tx> {
    tx: &'tx Transaction,
}

impl<'tx> TxXaProcessor<'tx> {
    pub fn regist() {
        add_processor_creator(
            "xa".to_string(),
            |tx: &Transaction| -> Box<dyn Processor> {
                return &TxMessageProcessor { tx };
            }
            .into(),
        );
    }

    fn branches() -> Vec<TransactionBranch> {
        Vec::new()
    }

    async fn exec(&self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error> {
        #[derive(Debug, Deserialize)]
        struct J {
            gid: uuid::Uuid,
            branch_id: uuid::Uuid,
            action: String,
        }
        let mut j = J {
            branch_id: branch.branch_id(),
            gid: self.tx.gid(),
            action: match self.tx.state() {
                State::Prepared => "rollback".to_string(),
                _ => "commit".to_string(),
            },
        };
        let cli = reqwest::Client::new();
        let resp = cli.post(branch.url()).json(j).send().await?;
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

    async fn once(&self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error> {
        let r#type = match self.tx.state() {
            State::Succeed => {
                return Ok(());
            }
            State::Submitted => "commit",
            _ => "rollback",
        };
        for branch in branches {
            match (branch.r#type(), branch.state()) {
                (r#type, State::Succeed) => {
                    self.exec(db, branch).await;
                }
            }
        }
        let state = match self.tx.state {
            State::Submitted => "succeed",
            _ => "failed",
        };
        self.update_state(db, state).await?;
        Ok(())
    }
}
