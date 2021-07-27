use std::collections::HashMap;
use std::sync::Mutex;

use chrono::Duration;
use chrono::prelude::*;
use quaint::pooled::PooledConnection;
use quaint::prelude::*;
use serde::Deserialize;
use tracing::{debug, event, Level};

use crate::errors;

pub type Gid = uuid::Uuid;

type Conn = PooledConnection;

pub static CREATORS: Mutex<HashMap<String, Box<dyn Fn(&Transaction) -> Box<dyn Processor>>>> = Mutex::new(HashMap::new());

#[derive(Debug, Deserialize)]
pub struct Transaction {
    gid: Gid,
    state: State,
    r#type: String,
    message: String,
    query_prepared: String,
    committed_at: Option<DateTime<Local>>,
    finished_at: Option<DateTime<Local>>,
    rollbacked_at: Option<DateTime<Local>>,
    next_cron_interval: i64,
    next_cron_time: Option<DateTime<Local>>,
    created_at: DateTime<Local>,
    last_modified: DateTime<Local>,
}

impl Transaction {
    pub fn new() -> Transaction {
        Transaction {
            gid: Gid::new_v4(),
            r#type: "".to_string(),
            message: "".to_string(),
            state: State::Prepared,
            query_prepared: "".to_string(),
            committed_at: None,
            finished_at: None,
            rollbacked_at: None,
            next_cron_interval: -1,
            next_cron_time: None,
            created_at: Local::now(),
            last_modified: Local::now(),
        }
    }

    pub fn tablename() -> &'static str {
        "tx_transactions"
    }

    pub fn gid(&self) -> uuid::Uuid {
        self.gid
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn state(&self) -> State {
        self.state
    }

    fn set_next_cron(&mut self, expire_in: i64) {
        self.next_cron_interval = expire_in;
        let next_cron_time = Local::now() + Duration::seconds(config.cron_interval);
        self.next_cron_time.replace(next_cron_time);
    }

    async fn touch(&mut self, db: &Conn, interval: i64) -> Result<(), errors::Error> {
        event!(Level::TRACE, action = "touch transaction", gid = ?self.gid, state = "", branch = "");
        self.set_next_cron(interval);
        let next_cron_time: DateTime<Utc> = self.next_cron_time.as_ref().unwrap().with_timezone(&Utc);
        let x = Update::table(Self::tablename())
            .set("next_cron_time", next_cron_time)
            .set("next_cron_interval", self.next_cron_interval)
            .so_that("gid".equals(self.gid));
        db.update(x).await?;
        Ok(())
        /*
        db.execute(
            sqlx::query("UPDATE ? SET next_cron_time = ?, next_cron_interval = ? WHERE gid = ?")
                .bind(self.next_cron_time)
                .bind(self.next_cron_interval)
                .bind(self.gid),
        )?;
        */
        // db.Model(&TransGlobal{}).Where("gid=?", t.Gid).Select(updates).Updates(t)
    }

    pub async fn update_state(&mut self, db: &Conn, state: State) -> Result<(), errors::Error> {
        event!(Level::TRACE, gid = ?self.gid, action= "change state", state= ?state, branch= "");
        let old = self.state;
        self.set_next_cron(config.cron_interval);
        let next_cron_time: DateTime<Utc> = self.next_cron_time.as_ref().unwrap().with_timezone(&Utc);
        let mut x = Update::table(Self::tablename())
            .set("next_cron_time", next_cron_time)
            .set("next_cron_interval", self.next_cron_interval)
            .set("state", state)
            .so_that("gid".equals(self.gid));
        let now = Local::now();
        match state {
            State::Succeed => {
                self.finished_at = Some(now);
                let finished_at: DateTime<Utc> = self.finished_at.as_ref().unwrap().with_timezone(&Utc);
                x = x.set("finished_at", finished_at);
            }
            State::Failed => {
                self.rollbacked_at = Some(now);
                let rollbacked_at: DateTime<Utc> = self.rollbacked_at.as_ref().unwrap().with_timezone(&Utc);
                x = x.set("rollbacked_at", rollbacked_at);
            }
            _ => {}
        }
        db.update(x).await?;
        self.state = state;
        Ok(())
    }

    fn processor(&self) -> Box<dyn Processor> {
	    return CREATORS.lock().unwrap().get_mut(&self.r#type).unwrap()(&self);
    }
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq)]
#[repr(i64)]
pub enum State {
    Succeed = 1,
    Submitted,
    Failed,
    Prepared, // "prepared"
    Aborting, // "aborting"
}

impl From<State> for Value<'static> {
    fn from(state: State) -> Value<'static> {
        Value::Integer(Some(state as i64))
    }
}

// TransBranch branch transaction
#[derive(Debug, Deserialize)]
pub struct TransactionBranch {
    gid: Gid,
    url: String,
    message: String,
    branch_id: Gid,
    r#type: String,
    state: State,
    finished_at: Option<DateTime<Local>>,
    rollbacked_at: Option<DateTime<Local>>,
    created_at: DateTime<Local>,
    last_modified: DateTime<Local>,
}

impl TransactionBranch {
    fn tablename() -> &'static str {
        "tx_transaction_branches"
    }

    pub fn new(gid: Gid, branch_id: uuid::Uuid, r#type: String, state: State, url: String, message: String, ) -> TransactionBranch {
        TransactionBranch { gid, branch_id, state, r#type, url, message, finished_at: None, rollbacked_at: None, created_at: Local::now(), last_modified: Local::now() }
    }

    pub fn r#type(&self) -> &str {
        &self.r#type
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    async fn update_state(&mut self, db: &Conn, state: State) -> Result<(), errors::Error> {
        event!(Level::DEBUG, gid= ?self.gid, action= "branch change state", state= ?state, branch_id= ?self.branch_id);
        let now = Local::now();
        let finished_at: DateTime<Utc> = now.with_timezone(&Utc);
        db.update(
            Update::table(TransactionBranch::tablename())
            .set("state", state)
            .set("finished_at", finished_at)
            .so_that("gid".equals(self.gid).and("branch_id".equals(self.branch_id)))
        ).await?;
        self.finished_at = Some(now);
        self.state = state;
        Ok(())
    }
}

#[async_trait]
pub trait Processor {
	fn branches(&self) -> Vec<TransactionBranch>;
	async fn once(&self, db: &Conn, branches: &[TransactionBranch]) -> Result<(), errors::Error>;
	async fn exec(&self, db: &Conn, branch: &TransactionBranch) -> Result<(), errors::Error>;
}

// type ProcessorCreator = ;

// var processorFac = map[string]processorCreator{}
pub fn add_processor_creator(r#type: String, creator: Box<dyn Fn(&Transaction) -> Box<dyn Processor>>) {
	CREATORS.lock().unwrap().insert(r#type, creator);
}

impl Transaction {
    // Process process global transaction once
    async fn process(&mut self, db: &Conn) -> Result<(), errors::Error> {
    	debug!("processing: {} state: {:?}", self.gid, self.state);
        let _defer = Defer::new(Box::new({
            let gid = self.gid.clone();
            move||{
    		// if TransProcessedTestChan != nil {
    			debug!("processed: {}", gid);
    		//   TransProcessedTestChan <- t.Gid
    		// }
        }}));
        match (self.state, self.r#type.as_str()) {
            (State::Prepared, "msg") => {
                self.update_state(db, State::Aborting).await?;
            }
            _ => {}
        };
        let branches: Vec<TransactionBranch> = quaint::serde::from_rows(
            db.select(Select::from_table(TransactionBranch::tablename()).so_that("gid".equals(self.gid)).order_by("id".ascend())).await?)?;
    	self.processor().once(db, &branches);
        Ok(())
    }

    fn get_branch_params(&self, branch: &TransactionBranch) -> HashMap<String, uuid::Uuid> {
        let mut params = HashMap::with_capacity(4);
        params.insert("gid".to_string(),         self.gid);
        // params.insert("trans_type",  self.r#type);
        params.insert(
    		"branch_id".to_string(),   branch.branch_id);
        // params.insert("branch_type", branch.r#type);
        params
    }

    async fn save(&mut self, db: Conn) -> Result<(), errors::Error> {
        let db = db.start_transaction().await?;
		    self.set_next_cron(config.cron_interval);
		    event!(Level::DEBUG, gid = ?self.gid, action = "create transaction", state = ?self.state, branch = "", data = self.data.into());
            // r#type: String,
            // data: String,
            // state: State,
            // query_prepared: String,
            // committed_at: Option<DateTime<Local>>,
            // finished_at: Option<DateTime<Local>>,
            // rollbacked_at: Option<DateTime<Local>>,
            // next_cron_interval: i64,
            // next_cron_time: Option<DateTime<Local>>,
            // created_at: DateTime<Local>,
            // last_modified: DateTime<Local>,
            let insertion = Insert::single_into(Transaction::tablename()).value("gid", self.gid).value("data", self.data).build().on_conflict(OnConflict::DoNothing);
            let set = db.insert(insertion).await?;
            if !set.is_empty() { // 如果这个是新事务，保存所有的分支
			    let branches = self.processor().branches();
			    if !branches.is_empty() {
				    event!(Level::DEBUG, gid = ?self.gid, action = "save branches", state = ?self.state, data = ?branches);
                    // gid: Gid,
                    // url: String,
                    // data: String,
                    // branch_id: Gid,
                    // r#type: String,
                    // state: State,
                    // finished_at: DateTime<Local>,
                    // rollbacked_at: DateTime<Local>,
                    // created_at: DateTime<Local>,
                    // last_modified: DateTime<Local>,
                    let insertion = Insert::multi_into(TransactionBranch::tablename(), vec!["gid", "data", "branch_id", "type", "state"]);
                    for branch in branches.iter() {
                        insertion = insertion.values((branch.gid, branch.data, branch.branch_id, branch.r#type, branch.state))
                    }
                    let insertion = insertion.build().on_conflict(OnConflict::DoNothing);
                    db.insert(insertion).await?;
			    }
		    } else if self.state == State::Submitted { // 如果数据库已经存放了prepared的事务，则修改状态
                let next_cron_time: DateTime<Utc> = self.next_cron_time.as_ref().unwrap().with_timezone(&Utc);
                let up = Update::table(Transaction::tablename()).set("next_cron_time", next_cron_time).set("next_cron_interval", self.next_cron_interval).set("state", self.state).so_that("gid".equals(self.gid));
                db.update(up).await?;
		    }
            db.commit().await?;
            Ok(())
	// e2p(err)
    }

    // TransFromDb construct trans from db
    async fn load(gid: Gid, db: Conn) -> Result<Transaction, errors::Error> {
        let q = Select::from_table(Transaction::tablename()).so_that("gid".equals(gid)).limit(1);
        db.select(q).await?.from_first()
        /*
	        if dbr.Error == gorm.ErrRecordNotFound {
	        	return nil
	        }
	        e2p(dbr.Error)
	        return &m
        */
    }
}

struct Defer {
    doit: Box<dyn FnOnce()>,
}

impl Drop for Defer {
        fn drop(&mut self) {
            (self.doit)()
        }
}

impl Defer {
    fn new(doit: Box<dyn FnOnce()>) -> Self {
        Defer { doit }
    }
}
