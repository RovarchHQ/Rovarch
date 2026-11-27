use crate::i18n::tr_vars;
use std::time::{SystemTime, UNIX_EPOCH};

const MINUTE: i64 = 60;
const HOUR: i64 = 60 * MINUTE;
const DAY: i64 = 24 * HOUR;

pub fn time_since(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|since_epoch| i64::try_from(since_epoch.as_secs()).ok())
        .unwrap_or(timestamp);
    let seconds = (now - timestamp).max(0);

    if seconds < HOUR {
        tr_vars(
            "{minutes} min ago",
            &[("minutes", &((seconds / MINUTE).max(1)).to_string())],
        )
    } else if seconds < DAY {
        tr_vars("{hours} h ago", &[("hours", &(seconds / HOUR).to_string())])
    } else {
        match seconds / DAY {
            1 => tr_vars("1 day ago", &[]),
            days => tr_vars("{days} days ago", &[("days", &days.to_string())]),
        }
    }
}
