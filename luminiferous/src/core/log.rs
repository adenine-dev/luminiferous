macro_rules! infoln {
    ($($arg:tt)*) => {
        println!("\x1B[0;34m[INFO] {}:{}\x1B[0;39m  {}", file!(), line!(), format!($($arg)*))
    };
}

macro_rules! warnln {
    ($($arg:tt)*) => {
        println!("\x1B[0;33m[WARN] {}:{}\x1B[0;39m  {}", file!(), line!(), format!($($arg)*))
    };
}

macro_rules! errorln {
    ($($arg:tt)*) => {
        println!("\x1B[0;31m[ERROR] {}:{}\x1B[0;39m  {}", file!(), line!(), format!($($arg)*))
    };
}

pub(crate) use errorln;
pub(crate) use infoln;
pub(crate) use warnln;
