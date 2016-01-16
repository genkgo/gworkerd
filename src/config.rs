use consumer::StompConfig;
use record_backend::MysqlConfig;
use monitor::Config as MonitorConfig;

pub static VERSION: &'static str = "0.5.0";

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct LogConfig {
  pub directory: String,
  pub levels: String
}

#[derive(Debug, Clone, RustcDecodable, RustcEncodable)]
pub struct Config {
  pub log: LogConfig,
  pub threads: u32,
  pub stomp: StompConfig,
  pub mysql: MysqlConfig,
  pub monitor: MonitorConfig
}