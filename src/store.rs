use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expiry {
    instant: Instant,
    pub duration: Duration,
}

impl Expiry {
    pub fn new(duration: Duration) -> Self {
        Expiry {
            instant: Instant::now(),
            duration,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Value {
    pub expiry: Option<Expiry>,
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct Store {
    db: HashMap<String, Value>,
}

impl Store {
    pub fn new() -> Self {
        Store { db: HashMap::new() }
    }

    pub fn clean_up(&mut self) {
        // This is super inefficient as it loops through all the keys in the store checking for expiry.
        // But this solution is sufficient to pass the codecrafters tests.
        // Perhaps a better solution would be to keep another data structure with keys sorted by
        // expiry time. This way, we can exit the loop early once we reach the non-expiring keys.
        for (key, value) in self.db.clone() {
            if let Some(expiry) = value.expiry {
                if expiry.instant.elapsed() > expiry.duration {
                    self.db.remove(&key);
                }
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.db.get(key)
    }

    pub fn set(&mut self, key: String, value: Value) {
        self.db.insert(key, value);
    }
}
