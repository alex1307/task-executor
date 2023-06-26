pub mod utils;
pub mod files;
pub mod error;

use log::warn;


pub const MAX_32_MB: usize = 1000000;

pub fn configure_log4rs() {
    log4rs::init_file("config/logs/log4rs.yml", Default::default()).unwrap();
    warn!("SUCCESS: Loggers are configured with dir: _log/*");
}
