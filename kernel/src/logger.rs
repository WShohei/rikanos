#![allow(dead_code)]

use log::{LevelFilter, SetLoggerError};
use log::{Metadata, Record};
use crate::{print, println};

const LOGGER: Logger = Logger;

pub fn init(max_level: LevelFilter) -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(max_level))
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}\n", record.level(), record.args())
        }
    }

    fn flush(&self) {}
}
