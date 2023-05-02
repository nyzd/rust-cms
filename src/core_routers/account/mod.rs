pub mod verify;
pub mod get_token;
pub mod send_verification;

use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub fn current_time_stamp() -> f64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Cant get timestamp");

    since_the_epoch.as_secs_f64()
}

pub fn generate_uuid() -> String {
    let id = Uuid::new_v4();

    id.to_string()
}
