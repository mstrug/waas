use std::{collections::hash_map::*, thread::panicking};
use rand::Rng;

#[derive(Clone, Debug)]
pub enum DbError {
    UserNotFound,
    UserAlreadyExists,
    WrongPassword,
    KeyNotFound
}

pub type UserId = u64;

pub trait DbInterface {
    async fn validate_user_password(&self, user: &str, password_hash: &str) -> Result<UserId, DbError>;

    async fn get_user_key(&self, user_id: UserId) -> Result<String, DbError>;

    async fn add_user(&mut self, user: &str, password_hash: &str) -> Result<UserId, DbError>;
}


pub struct MemDb {
    users: HashMap<String, (UserId, String)>,
    keys: HashMap<UserId, String>,
}

impl MemDb {
    pub fn new() -> Self {
        Self {
            users: HashMap::from([
                ("user1".to_string(), (1, "$2y$05$gifLHpZdNAixJzy36HyOc.1PsRNbn5Je9vlWalKyg3sGqSAW.8rFG".to_string())),
                ("user2".to_string(), (2, "$2y$05$gifLHpZdNAixJzy36HyOc.ge.9FMFAI.6NwvXHqIpLQpCF3hepE9e".to_string())),
            ]),
// 123456 $2y$05$gifLHpZdNAixJzy36HyOc.1PsRNbn5Je9vlWalKyg3sGqSAW.8rFG
// Alex5 $2y$05$gifLHpZdNAixJzy36HyOc.ge.9FMFAI.6NwvXHqIpLQpCF3hepE9e
// trumpet7 $2y$05$gifLHpZdNAixJzy36HyOc.wUk55Iu1fE9xt4ji7wq/WLxS8S4zDV2
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

    async fn add_user(&mut self, user: &str, password_hash: &str) -> Result<UserId, DbError> {
        if self.users.contains_key(user) {
            Err(DbError::UserAlreadyExists)
        } else {
            let mut rng = rand::thread_rng();
            let user_id: UserId = rng.gen();
            self.users.insert(user.to_string(), (user_id, password_hash.to_string()));
            Ok(user_id)
        }
    }
}