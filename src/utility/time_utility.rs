use std::time::{SystemTime};

pub enum TimeUnit{
    SECONDS,
    MILLI
}

pub fn current_time(timeunit: TimeUnit) -> Option<u128>{
    let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH){
        Ok(value) => match timeunit {
                TimeUnit::SECONDS => Some(value.as_secs() as u128),
                TimeUnit::MILLI => Some(value.as_millis())
            },
        Err(_) => None
    };

    timestamp
}
