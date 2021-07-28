use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Config {
    pub delay: i64, // 单位秒 当事务等待这个时间之后，还没有变化，则进行一轮处理，包括prepared中的任务和committed的任务
    pub database_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            delay: 10,
            database_url: String::new(),
        }
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();
