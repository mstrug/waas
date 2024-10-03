use std::{collections::hash_map::*, thread::panicking};


pub enum DbError {
    UserNotFound,
    WrongPassword,
    KeyNotFound
}

pub type UserId = u64;

pub trait DbInterface {
    async fn validate_user_password(&self, user: &str, password_hash: &str) -> Result<UserId, DbError>;

    async fn get_user_key(&self, user_id: UserId) -> Result<String, DbError>;
}


pub struct MemDb {
    users: HashMap<String, (UserId, String)>,
    keys: HashMap<UserId, String>,
}

impl MemDb {
    pub fn new() -> Self {
        Self {
            users: HashMap::from([
                ("user1".to_string(), (1, "bb".to_string())),
                ("user2".to_string(), (2, "cc".to_string())),
            ]),
            keys: HashMap::from([
                (1, "k1".to_string()),
                //(2, "k2".to_string()),
            ]),
        }
    }
}

impl DbInterface for MemDb {
    async fn validate_user_password(&self, user: &str, password_hash: &str) -> Result<UserId, DbError> {
        if let Some(v) = self.users.get(user) {
            if v.1 == password_hash {
                Ok(v.0)
            } else {
                Err(DbError::WrongPassword)
            }
        } else {
            Err(DbError::UserNotFound)
        }
    }

    async fn get_user_key(&self, user_id: UserId) -> Result<String, DbError> {
        self.keys.get(&user_id).ok_or(DbError::KeyNotFound).cloned()
    }
}