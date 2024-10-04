use std::collections::hash_map::*;

#[derive(Clone, Debug)]
pub enum DbError {
    UserNotFound,
    WrongPassword,
    KeyNotFound
}

pub type UserId = u64;

pub struct MemDb {
    users: HashMap<String, (UserId, String)>,
    keys: HashMap<UserId, Vec<u8>>,
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
            keys: HashMap::new()
        }
    }

    pub fn validate_user_password(&self, user: &str, password_hash: &str) -> Result<UserId, DbError> {
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

    pub fn get_user_key(&self, user_id: UserId) -> Result<Vec<u8>, DbError> {
        self.keys.get(&user_id).ok_or(DbError::KeyNotFound).cloned()
    }

    pub fn get_user_name(&self, user_id: UserId) -> Option<String> {
        self.users.iter().find_map(|i| { if i.1.0 == user_id { Some(i.0) } else { None } }).cloned()
    }

    // pub fn _add_user(&mut self, user: &str, password_hash: &str) -> Result<UserId, DbError> {
    //     if self.users.contains_key(user) {
    //         Err(DbError::UserAlreadyExists)
    //     } else {
    //         let mut rng = rand::thread_rng();
    //         let user_id: UserId = rng.gen();
    //         self.users.insert(user.to_string(), (user_id, password_hash.to_string()));
    //         Ok(user_id)
    //     }
    // }

    pub fn add_user_key(&mut self, user_id: UserId, key: &[u8]) -> Result<(), DbError> {
        self.keys.insert(user_id, key.to_vec());
        Ok(())
    }

    pub fn discard_user_key(&mut self, user_id: UserId) -> Result<(), DbError> {
        self.keys.remove(&user_id);
        Ok(())
    }
}