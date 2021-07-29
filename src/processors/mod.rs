use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::models::transaction::{Transaction, TransactionBranch};
use crate::database::Conn;
use crate::errors;

mod tx_tcc_processor;
mod tx_saga_processor;
mod tx_xa_processor;
mod tx_message_processor;

pub use tx_xa_processor::TxXaProcessor;
pub use tx_tcc_processor::TxTCCProcessor;
pub use tx_saga_processor::TxSagaProcessor;
pub use tx_message_processor::TxMessageProcessor;

#[async_trait]
pub trait Processor<'tx>: Debug + Send {
    fn with_transaction(tx: &'tx mut Transaction) -> Box<dyn Processor<'tx> + Send + 'tx> where Self: Sized;
    fn branches(&self) -> Vec<TransactionBranch>;
    async fn once(&mut self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error>;
    async fn exec(&mut self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Xa {
    //
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TCC {
    //
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Saga {
    steps: Vec<SagaStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SagaStep {
    payload: String,
    on_reverting: String,
    on_committing: String,
}

impl SagaStep {
    pub fn new(on_committing: String, on_reverting: String, payload: String) -> SagaStep {
        SagaStep {
            payload,
            on_reverting,
            on_committing
        }
    }

    pub fn on_reverting(&self) -> &str {
        &self.on_reverting
    }

    pub fn on_committing(&self) -> &str {
        &self.on_committing
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    steps: Vec<MessageStep>,
    query_prepared: String,
}

impl Message {
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageStep {
    payload: String,
    callback: String,
}

impl MessageStep {
    pub fn new(payload: String, callback: String) -> MessageStep {
        MessageStep {
            payload,
            callback
        }
    }

    pub fn payload(&self) -> &str {
        &self.payload
    }

    pub fn callback(&self) -> &str {
        &self.callback
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "lowercase")]
pub enum ProcessorType {
    Xa(Xa),
    TCC(TCC),
    Saga(Saga),
    Message(Message),
}

impl ProcessorType {
    pub fn take_processor<'tx>(&self, tx: &'tx mut Transaction) -> Box<dyn Processor<'tx> + 'tx> {
        match self {
            ProcessorType::Xa(_) => TxXaProcessor::with_transaction(tx),
            ProcessorType::TCC(_) => TxTCCProcessor::with_transaction(tx),
            ProcessorType::Saga(_) => TxSagaProcessor::with_transaction(tx),
            ProcessorType::Message(_) => TxMessageProcessor::with_transaction(tx)
        }
    }
}

/*
impl FromStr for ProcessorType {
    type Err = errors::Error;
    fn from_str(ty: &str) -> Result<ProcessorType, Self::Err> {
        match ty.to_lowercase().as_str() {
            "xa" => Ok(ProcessorType::Xa),
            "tcc" => Ok(ProcessorType::TCC),
            "saga" => Ok(ProcessorType::Saga),
            "message" => Ok(ProcessorType::Message),
            _ => Err(errors::Error::InvalidProcessorType(ty.to_string()))
        }
    }
}
*/
