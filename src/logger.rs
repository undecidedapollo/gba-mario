use core::cell::RefCell;
use core::fmt::Write;

use gba::mgba::{MgbaBufferedLogger, MgbaMessageLevel};

pub struct Logger {
    error_buf: RefCell<Option<MgbaBufferedLogger>>,
    warning_buf: RefCell<Option<MgbaBufferedLogger>>,
    info_buf: RefCell<Option<MgbaBufferedLogger>>,
    debug_buf: RefCell<Option<MgbaBufferedLogger>>,
}

impl Logger {
    pub const fn new() -> Self {
        Logger {
            error_buf: RefCell::new(None),
            warning_buf: RefCell::new(None),
            info_buf: RefCell::new(None),
            debug_buf: RefCell::new(None),
        }
    }

    pub fn init(&self) {
        *self.error_buf.borrow_mut() = MgbaBufferedLogger::try_new(MgbaMessageLevel::Error).ok();
        *self.warning_buf.borrow_mut() =
            MgbaBufferedLogger::try_new(MgbaMessageLevel::Warning).ok();
        *self.info_buf.borrow_mut() = MgbaBufferedLogger::try_new(MgbaMessageLevel::Info).ok();
        *self.debug_buf.borrow_mut() = MgbaBufferedLogger::try_new(MgbaMessageLevel::Debug).ok();
    }

    pub fn error(&self, args: core::fmt::Arguments) {
        let mut buf_opt = self.error_buf.borrow_mut();
        if let Some(ref mut buf) = *buf_opt {
            let _ = writeln!(buf, "{}", args);
        }
    }

    pub fn warning(&self, args: core::fmt::Arguments) {
        let mut buf_opt = self.warning_buf.borrow_mut();
        if let Some(ref mut buf) = *buf_opt {
            let _ = writeln!(buf, "{}", args);
        }
    }

    pub fn info(&self, args: core::fmt::Arguments) {
        let mut buf_opt = self.info_buf.borrow_mut();
        if let Some(ref mut buf) = *buf_opt {
            let _ = writeln!(buf, "{}", args);
        }
    }

    pub fn debug(&self, args: core::fmt::Arguments) {
        let mut buf_opt = self.debug_buf.borrow_mut();
        if let Some(ref mut buf) = *buf_opt {
            let _ = writeln!(buf, "{}", args);
        }
    }
}

pub struct SingleThreadLogger(pub Logger);
unsafe impl Sync for SingleThreadLogger {}

pub static LOGGER: SingleThreadLogger = SingleThreadLogger(Logger::new());

pub fn init_logger() {
    LOGGER.0.init();
}

#[macro_export]
macro_rules! gba_error {
    ($($arg:tt)*) => {
        $crate::logger::LOGGER.0.error(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! gba_warning {
    ($($arg:tt)*) => {
        $crate::logger::LOGGER.0.warning(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! gba_info {
    ($($arg:tt)*) => {
        $crate::logger::LOGGER.0.info(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! gba_debug {
    ($($arg:tt)*) => {
        $crate::logger::LOGGER.0.debug(format_args!($($arg)*))
    };
}
