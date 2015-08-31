extern crate mysql;

use self::mysql::conn::MyOpts;
use self::mysql::conn::pool::MyPool;
use std::clone::Clone;
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

impl MysqlConfig {

  pub fn new_pool (&self) -> MyPool {
    let opts = MyOpts {
    tcp_addr: Some(self.address.clone()),
    user: Some(self.username.clone()),
    pass: Some(self.password.clone()),
    db_name: Some(self.database.to_string()),
    ..Default::default()
    };
    MyPool::new(opts).unwrap()
  }
}


#[derive(Clone, Debug)]
pub struct MysqlBackend {
  pool: MyPool
}

impl MysqlBackend {

  pub fn new (pool: MyPool) -> MysqlBackend {
    MysqlBackend { pool: pool }
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

    let mut command = request.program.clone();
    command.push_str(" ");
    for arg in &request.args {
      command.push_str(" ");
      command.push_str(arg);
    }

    let query = r"INSERT INTO results (id, command, cwd, status, stderr, stdout) VALUES (UNHEX(?), ?, ?, ?, ?, ?)";
    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(ResultBackendError::CannotStoreResult)
    };

    let result = match stmt.execute(
      (&ordered_uuid, command, &request.cwd, &response.status, &response.stderr, &response.stdout)
    ) {
      Ok(_) => Ok(()),
      Err(_) => return Err(ResultBackendError::CannotStoreResult)
    };

    result
  }

}