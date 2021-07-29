use std::fmt::Debug;
use std::str::FromStr;

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

pub enum ProcessorType {
    Xa,
    TCC,
    Saga,
    Message,
}

impl ProcessorType {
    pub fn take_processor<'tx>(&self, tx: &'tx mut Transaction) -> Box<dyn Processor<'tx> + 'tx> {
        match self {
            ProcessorType::Xa => TxXaProcessor::with_transaction(tx),
            ProcessorType::TCC => TxTCCProcessor::with_transaction(tx),
            ProcessorType::Saga => TxSagaProcessor::with_transaction(tx),
            ProcessorType::Message => TxMessageProcessor::with_transaction(tx)
        }
    }
}

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
