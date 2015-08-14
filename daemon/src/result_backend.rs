extern crate mysql;

use self::mysql::conn::MyOpts;
use self::mysql::conn::pool::MyPool;
use std::default::Default;
use worker::Item;

pub trait ResultBackend {
  fn store (&self, worker: &Item) -> Result<(), ResultBackendError>;
}

pub enum ResultBackendError {
  CannotStoreResult
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct MysqlConfig {
  address: String,
  username: String,
  password: String,
  database: String
}

pub struct MysqlBackend {
  pool: MyPool
}

impl MysqlBackend {

  pub fn new (config: &MysqlConfig) -> MysqlBackend {
    let opts = MyOpts {
      tcp_addr: Some(config.address.clone()),
      user: Some(config.username.clone()),
      pass: Some(config.password.clone()),
      db_name: Some(config.database.to_string()),
      ..Default::default()
    };

    MysqlBackend { pool: MyPool::new(opts).unwrap() }
  }

}

impl ResultBackend for MysqlBackend {

  fn store (&self, worker: &Item) -> Result<(), ResultBackendError> {
    let request = &worker.request.clone();
    let response = &worker.response.clone();

    // insert uuid's the optimized way https://www.percona.com/blog/2014/12/19/store-uuid-optimized-way/
    let mut ordered_uuid = request.id[14..18].to_string();
    ordered_uuid.push_str(&request.id[9..13]);
    ordered_uuid.push_str(&request.id[0..8]);
    ordered_uuid.push_str(&request.id[19..23]);
    ordered_uuid.push_str(&request.id[24..]);

    let query = r"INSERT INTO results (id, command, cwd, status, stderr, stdout) VALUES (UNHEX(?), ?, ?, ?, ?, ?)";
    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(ResultBackendError::CannotStoreResult)
    };

    let result = match stmt.execute(
      (&ordered_uuid, &request.command, &request.cwd, &response.status, &response.stderr, &response.stdout)
    ) {
      Ok(_) => Ok(()),
      Err(_) => return Err(ResultBackendError::CannotStoreResult)
    };

    result
  }

}