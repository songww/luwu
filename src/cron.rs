use rand::prelude::*;

use std::thread;
use std::time::Duration;

/// Cron expired trans. use expire_in as expire time
fn once(expire_after: Duration) -> bool {
    if let Ok(trans) = lock(expire_after) {
        // defer WaitTransProcessed(trans.Gid);
        trans.process(dbGet());
        return true;
    }
    return false;
}

// Cron expired trans, num == -1 indicate for ever
fn expired(mut num: isize) {
    loop {
        if num == 0 {
            break;
        }
        if once(Duration::new(0, 0)) {
            sleep();
        }
    }
}

struct Database;

fn database() -> Database {
    Database
}

fn lock(expire_after: Duration) -> Transaction {
    let mut trans = Transaction::new();
    let owner = trans.gid;
    let db = database();
    // 这里next_cron_time需要限定范围，否则数据量累计之后，会导致查询变慢
    // 限定update_time < now - 3，否则会出现刚被这个应用取出，又被另一个取出
    // dbr := db.Must().Model(&trans).
    // Where("next_cron_time < date_add(now(), interval ? second) and next_cron_time > date_add(now(), interval -3600 second) and update_time < date_add(now(), interval ? second) and status in ('prepared', 'aborting', 'submitted')", int(expireIn/time.Second), -3+int(expireIn/time.Second)).
    // Limit(1).Update("owner", owner)
    // if dbr.RowsAffected == 0 {
    // 	return nil
    // }
    // dbr = db.Must().Where("owner=?", owner).Find(&trans)
    // updates := trans.setNextCron(trans.NextCronInterval * 2) // 下次被cron的间隔加倍
    // db.Must().Model(&trans).Select(updates).Updates(&trans)
    trans
}

/*
func handlePanic() {
    if err := recover(); err != nil {
        logrus.Errorf("\x1b[31m\n----panic %s handlered\x1b[0m\n%s", err.(error).Error(), string(debug.Stack()))
    }
}
*/

fn sleep() {
    let delta = 3f32.min(config.cron_interval as f32);
    let interval =
        Duration::from_secs_f32(config.cron_interval as f32 - rand::random::<f32>() * delta);
    thread.sleep(interval);
}
