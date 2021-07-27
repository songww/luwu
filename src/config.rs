pub struct Config {
    cron_interval: i64, // 单位秒 当事务等待这个时间之后，还没有变化，则进行一轮处理，包括prepared中的任务和committed的任务
    database_url: String,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            cron_interval: 10,
            database_url: String::new(),
        }
    }
}
