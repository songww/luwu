use rocket::tokio;
use rocket::serde::{json::Json, uuid::Uuid};
use rocket_versioning::Versioning;
use serde::{Deserialize, Serialize};

use crate::processors::ProcessorType;
use crate::config::Config;
use crate::database::DB;
use crate::errors::{self, ErrorResponse};
use crate::models::transaction::{Gid, State, Transaction, TransactionBranch, TransactionCreation};
use crate::responder::DynResponse;

#[get("/transactions/<gid>")]
async fn fetch_transaction(
    _v: Versioning<1, 0>,
    db: DB,
    gid: Uuid,
) -> Result<DynResponse<Tx>, errors::ErrorResponse> {
    let tx = Transaction::load(gid, db.as_ref()).await?;
    let branches = tx.branches(db.as_ref()).await?;
    Ok(DynResponse::new(Tx { tx, branches }))
}

#[post("/transactions", data = "<tx>")]
async fn create_transaction(
    _v: Versioning<1, 0>,
    db: DB,
    tx: Json<TransactionCreation>,
) -> Result<String, errors::ErrorResponse> {
    let mut tx = Transaction::from(tx.0);
    tx.save(db.as_ref()).await?;
    Ok(tx.gid().to_string())
}

#[get("/gid")]
fn gid(_v: Versioning<1, 0>) -> String {
    Gid::new_v4().to_string()
}

#[put("/transactions/<gid>/submitting")]
async fn submit(_v: Versioning<1, 0>, db: DB, gid: Uuid) -> Result<String, errors::ErrorResponse> {
    let mut tx = Transaction::load(gid, db.as_ref()).await?;
    match tx.state() {
        State::Prepared | State::Submitted => {
            //
        }
        _ => {
            let err = errors::Error::CannotSubmitTransaction(gid, tx.state().tag().to_string());
            return Err(ErrorResponse::new(err, 5010));
        }
    }
    tx.submitted();
    tx.save(db.as_ref()).await?;
    let db = db.into();
    tokio::task::spawn(async move { tx.process(&db).await.unwrap(); });
    Ok("SUCCESS".to_string())
}

#[put("/transactions/<gid>/aborting")]
async fn abort(_v: Versioning<1, 0>, db: DB, gid: Uuid) -> Result<String, errors::ErrorResponse> {
    let mut tx = Transaction::load(gid, db.as_ref()).await?;
    match tx.r#type() {
        &ProcessorType::Xa(_) | &ProcessorType::TCC(_) => {
            //
        },
        _ => {
            let err = errors::Error::UnexpectedType(tx.r#type().tag().to_string(), tx.state().tag().to_string(), "it is not abortable!".to_string());
            return Err(ErrorResponse::new(err, 5020));
        }
    }
    match tx.state() {
        State::Prepared | State::Aborting => {},
        _ => {
            let err = errors::Error::UnexpectedType(tx.r#type().tag().to_string(), tx.state().tag().to_string(), "it is not abortable!".to_string());
            return Err(ErrorResponse::new(err, 5020));
        }
    }
    tokio::task::spawn(async move { tx.process(db.as_ref()).await.unwrap() });
    Ok("SUCCESS".to_string())
}

#[post("/transactions/<gid>/branches", data = "<tb>")]
async fn create_xa_branches(
    _v: Versioning<1, 0>,
    db: DB,
    config: &rocket::State<Config>,
    gid: Uuid,
    tb: Json<TransactionBranch>,
) -> Result<String, errors::ErrorResponse> {
    let mut tx = Transaction::load(gid, db.as_ref()).await?;
    match tx.state() {
        State::Prepared => {
            //
        }
        _ => {
            let err = errors::Error::UnexpectedType(tx.r#type().tag().to_string(), tx.state().tag().to_string(), "is denied for branch registering.".to_string());
            return Err(ErrorResponse::new(err, 5030));
        }
    }
    let mut branches = vec![tb.clone(), tb.clone()];
    branches[0].with_type("rollback".to_string());
    branches[1].with_type("commit".to_string());
    /*
    db.Must().Clauses(clause.OnConflict{
        DoNothing: true,
    }).Create(branches)
    */
    tx.touch(db.as_ref(), config.delay).await?;
    Ok(gid.to_string())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TCCBranchCreation {
    state: State,
    payload: String,
    cancel_url: String,
    confirm_url: String,
    r#try_url: String,
}

#[post("/transactions/<gid>/branches/<branch_id>/tcc", data = "<branch>")]
async fn create_tcc_branches(
    _v: Versioning<1, 0>,
    db: DB,
    gid: Uuid,
    branch_id: Uuid,
    branch: Json<TCCBranchCreation>,
    config: &rocket::State<Config>,
) -> Result<String, errors::ErrorResponse> {
    let mut tx = Transaction::load(gid, db.as_ref()).await?;
    match tx.state() {
        State::Prepared => {
            //
        }
        _ => {
            let err = errors::Error::UnexpectedType(tx.r#type().tag().to_string(), tx.state().tag().to_string(), "is denied for branch registering.".to_string());
            return Err(ErrorResponse::new(err, 5030));
        }
    }

    let TCCBranchCreation {
        cancel_url,
        confirm_url,
        try_url,
        state,
        payload,
    } = branch.0;

    let mut branches = Vec::with_capacity(3);
    branches.push(TransactionBranch::new(
        gid,
        branch_id,
        "cancel".to_string(),
        state,
        cancel_url,
        payload.to_string(),
    ));
    branches.push(TransactionBranch::new(
        gid,
        branch_id,
        "confirm".to_string(),
        state,
        confirm_url,
        payload.to_string(),
    ));
    branches.push(TransactionBranch::new(
        gid,
        branch_id,
        "try".to_string(),
        state,
        try_url,
        payload.to_string(),
    ));

    /*
    db.Must().Clauses(clause.OnConflict{
        DoNothing: true,
    }).Create(branches)
    */
    tx.touch(db.as_ref(), config.delay).await?;
    Ok("SUCCESS".to_string())
}

#[derive(Debug, Deserialize, Serialize)]
struct Tx {
    #[serde(flatten)]
    tx: Transaction,
    branches: Vec<TransactionBranch>,
}

pub fn routes() -> Vec<rocket::Route> {
    routes![
        gid,
        fetch_transaction,
        create_transaction,
        create_tcc_branches,
        create_xa_branches,
        submit,
        abort
    ]
}
