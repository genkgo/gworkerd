extern crate mysql;

use self::mysql::conn::MyOpts;
use self::mysql::conn::pool::MyPool;
use std::clone::Clone;
use std::default::Default;
use worker::Record;

pub trait ResultBackend {
  fn store (&self, record: Record) -> Result<(), ResultBackendError>;
}

pub enum ResultBackendError {
  CannotStoreRecord
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct MysqlConfig {
  address: String,
  username: String,
  password: String,
  database: String
}

impl MysqlConfig {

  pub fn to_connection (&self) -> MysqlBackend {
    let opts = MyOpts {
    tcp_addr: Some(self.address.clone()),
    user: Some(self.username.clone()),
    pass: Some(self.password.clone()),
    db_name: Some(self.database.to_string()),
    ..Default::default()
    };
	MysqlBackend::new(MyPool::new(opts).unwrap())
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

#[derive(Clone, Debug)]
pub struct MysqlOptimizedUuid {
  uuid: String
}

impl MysqlOptimizedUuid {
  pub fn from_uuid (uuid: String) -> MysqlOptimizedUuid {
     // insert uuid's the optimized way https://www.percona.com/blog/2014/12/19/store-uuid-optimized-way/
     let mut ordered_uuid = uuid[14..18].to_string();
     ordered_uuid.push_str(&uuid[9..13]);
     ordered_uuid.push_str(&uuid[0..8]);
     ordered_uuid.push_str(&uuid[19..23]);
     ordered_uuid.push_str(&uuid[24..]);
	 MysqlOptimizedUuid { uuid: ordered_uuid }
  }
}

impl ResultBackend for MysqlBackend {

  fn store (&self, record: Record) -> Result<(), ResultBackendError> {
    let uuid_optimized = MysqlOptimizedUuid::from_uuid(record.id.clone());
    let query = r"INSERT INTO results (id, command, cwd, status, stderr, stdout) VALUES (UNHEX(?), ?, ?, ?, ?, ?)";
    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(ResultBackendError::CannotStoreRecord)
    };

    let result = match stmt.execute(
      (uuid_optimized.uuid, record.command, record.cwd, record.status, record.stderr, record.stdout)
    ) {
      Ok(_) => Ok(()),
      Err(_) => return Err(ResultBackendError::CannotStoreRecord)
    };

    result
  }

}