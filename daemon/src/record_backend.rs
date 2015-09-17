extern crate chrono;
extern crate mysql;

use self::chrono::UTC;
use self::chrono::offset::TimeZone;
use self::mysql::conn::MyOpts;
use self::mysql::conn::pool::MyPool;
use self::mysql::error::MyResult;
use self::mysql::value::from_row;
use self::mysql::value::Value;
use std::clone::Clone;
use std::default::Default;
use worker::Record;

pub trait RecordRepository {
  fn store (&self, record: Record) -> Result<(), RecordRepositoryError>;
  fn fetch_record (&self, id: String) -> Result<(Record), RecordRepositoryError>;
  fn fetch_limit (&self, size: u32, offset: u32) -> Result<(Vec<Record>), RecordRepositoryError>;
}

#[derive(Debug)]
pub enum RecordRepositoryError {
  CannotStoreRecord,
  CannotFetchRecord,
  CannotDenormalizeRecord,
  RecordNotFound
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct MysqlConfig {
  address: String,
  username: String,
  password: String,
  database: String
}

impl MysqlConfig {

  pub fn to_connection (&self) -> MysqlRepository {
    let opts = MyOpts {
      tcp_addr: Some(self.address.clone()),
      user: Some(self.username.clone()),
      pass: Some(self.password.clone()),
      db_name: Some(self.database.to_string()),
      ..Default::default()
    };
    MysqlRepository::new(MyPool::new(opts).unwrap())
  }
}


#[derive(Clone, Debug)]
pub struct MysqlRepository {
  pool: MyPool
}

impl MysqlRepository {

  pub fn new (pool: MyPool) -> MysqlRepository {
    MysqlRepository { pool: pool }
  }

  fn row_to_record (&self, row: MyResult<Vec<Value>>) -> Record {
    let (id, command, cwd, status, stderr, stdout, started_at_col, finished_at_col) = from_row::<(String, String, String, i32, String, String, String, String)>(row.unwrap());

    let started_at = UTC.datetime_from_str(&started_at_col, "%Y-%m-%d %H:%M:%S").unwrap();
    let finished_at = UTC.datetime_from_str(&finished_at_col, "%Y-%m-%d %H:%M:%S").unwrap();

    let optimized_uuid = MysqlOptimizedUuid { uuid: id.to_string() };

    Record {
      id: optimized_uuid.to_uuid(),
      command: command,
      cwd: cwd,
      status: status,
      stderr: stderr,
      stdout: stdout,
      started_at: started_at,
      finished_at: finished_at
    }
  }

}

#[derive(Clone, Debug)]
pub struct MysqlOptimizedUuid {
  uuid: String
}

impl MysqlOptimizedUuid {

  pub fn from_uuid (uuid: String) -> MysqlOptimizedUuid {
     // the optimized way https://www.percona.com/blog/2014/12/19/store-uuid-optimized-way/
    let mut ordered_uuid = uuid[14..18].to_string();
    ordered_uuid.push_str(&uuid[9..13]);
    ordered_uuid.push_str(&uuid[0..8]);
    ordered_uuid.push_str(&uuid[19..23]);
    ordered_uuid.push_str(&uuid[24..]);
    MysqlOptimizedUuid { uuid: ordered_uuid }
  }

  pub fn to_uuid (&self) -> String {
    let mut uuid = self.uuid[8..16].to_string();
    uuid.push_str("-");
    uuid.push_str(&self.uuid[4..8]);
    uuid.push_str("-");
    uuid.push_str(&self.uuid[0..4]);
    uuid.push_str("-");
    uuid.push_str(&self.uuid[16..20]);
    uuid.push_str("-");
    uuid.push_str(&self.uuid[20..]);
    uuid
  }
}

impl RecordRepository for MysqlRepository {

  fn store (&self, record: Record) -> Result<(), RecordRepositoryError> {
    let uuid_optimized = MysqlOptimizedUuid::from_uuid(record.id.clone());

    let query = r"INSERT INTO results (id, command, cwd, status, stderr, stdout, started_at, finished_at) VALUES (UNHEX(?), ?, ?, ?, ?, ?, ?, ?)";

    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(RecordRepositoryError::CannotStoreRecord)
    };

    let result = match stmt.execute(
      (uuid_optimized.uuid, record.command, record.cwd, record.status, record.stderr,
        record.stdout, record.started_at.format("%Y-%m-%d %H:%M:%S").to_string(), record.finished_at.format("%Y-%m-%d %H:%M:%S").to_string()
      )
    ) {
      Ok(_) => Ok(()),
      Err(_) => return Err(RecordRepositoryError::CannotStoreRecord)
    };

    result
  }

  fn fetch_limit (&self, size: u32, limit: u32) -> Result<(Vec<Record>), RecordRepositoryError> {
    let query = r"SELECT HEX(id) AS id, command, cwd, status, stderr, stdout, CAST(started_at AS char) AS started_at, CAST(finished_at AS char) AS finished_at FROM results LIMIT ? OFFSET ?";

    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(RecordRepositoryError::CannotFetchRecord)
    };

    let results: Result<(Vec<Record>), RecordRepositoryError> = match stmt
      .execute((size, limit))
      .map(|result| {
        result.map(|row| {
          self.row_to_record(row)
        }).collect()
      })
    {
      Ok(records) => Ok(records),
      Err(_) => return Err(RecordRepositoryError::CannotDenormalizeRecord)
    };

    results
  }

  fn fetch_record (&self, id: String) -> Result<(Record), RecordRepositoryError> {
    let query = r"SELECT HEX(id) AS id, command, cwd, status, stderr, stdout, started_at, finished_at FROM results WHERE id = ?";

    let mut stmt = match self.pool.prepare(query) {
      Ok(s) => s,
      Err(_) => return Err(RecordRepositoryError::CannotFetchRecord)
    };

    let results: Result<(Vec<Record>), RecordRepositoryError> = match stmt
      .execute((id, ))
      .map(|result| {
        result.map(|row| {
          self.row_to_record(row)
        }).collect()
      })
    {
      Ok(records) => Ok(records),
      Err(_) => return Err(RecordRepositoryError::CannotDenormalizeRecord)
    };

    let records: Vec<Record> = results.unwrap();
    let result: Result<(Record), RecordRepositoryError> = match records.len() {
      1 => {
        Ok(records[0].clone())
      },
      _ => return Err(RecordRepositoryError::RecordNotFound)
    };

    result
  }
}

#[cfg(test)]
mod tests {
  use super::MysqlOptimizedUuid;

  #[test]
  fn optimized_uuid() {
    let uuid = String::from("58e0a7d7-eebc-11d8-9669-0800200c9a66");
    let optimized_uuid = MysqlOptimizedUuid::from_uuid(uuid);
    assert_eq!("11d8eebc58e0a7d796690800200c9a66", optimized_uuid.uuid);
    assert_eq!("58e0a7d7-eebc-11d8-9669-0800200c9a66", optimized_uuid.to_uuid());
  }
}