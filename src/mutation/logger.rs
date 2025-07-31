use std::time::SystemTime;

const COLOR_INFO: &str = "\x1b[38;2;90;160;100m";
const COLOR_WARN: &str = "\x1b[38;2;242;165;0m";
const COLOR_DEBUG: &str = "\x1b[38;2;242;165;0m";
const COLOR_TRACE: &str = "\x1b[38;2;242;165;0m";
const COLOR_ERROR: &str = "\x1b[38;2;215;80;110m";
const COLOR_FILENAME: &str = "\x1b[38;2;118;101;149m";
const COLOR_RESET: &str = "\x1b[0m";

pub struct MutationLogger;

#[allow(dead_code)]
impl MutationLogger {
    pub fn info(msg: &str) {
        println!(
            "{}  {}INFO{}  {}{}{}",
            Self::timestamp(),
            COLOR_INFO,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn info_file(filename: &str, msg: &str) {
        println!(
            "{}  {}INFO{}  {}{}{} {}{}{}",
            Self::timestamp(),
            COLOR_INFO,
            COLOR_RESET,
            COLOR_FILENAME,
            filename,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn step(msg: &str) {
        println!(
            "{}  {}TRACE{}  {}{}{}",
            Self::timestamp(),
            COLOR_TRACE,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn debug(msg: &str) {
        println!(
            "{}  {}DEBUG{}  {}{}{}",
            Self::timestamp(),
            COLOR_DEBUG,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn trace(msg: &str) {
        println!(
            "{}  {}TRACE{}  {}{}{}",
            Self::timestamp(),
            COLOR_TRACE,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn warn(msg: &str) {
        println!(
            "{}  {}WARN {}  {}{}{}",
            Self::timestamp(),
            COLOR_WARN,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn warn_file(filename: &str, msg: &str) {
        println!(
            "{}  {}WARN {}  {}{}{} {}{}{}",
            Self::timestamp(),
            COLOR_WARN,
            COLOR_RESET,
            COLOR_FILENAME,
            filename,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn error(msg: &str) {
        println!(
            "{}  {}ERROR{}  {}{}{}",
            Self::timestamp(),
            COLOR_ERROR,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn error_file(filename: &str, msg: &str) {
        println!(
            "{}  {}ERROR{}  {}{}{} {}{}{}",
            Self::timestamp(),
            COLOR_ERROR,
            COLOR_RESET,
            COLOR_FILENAME,
            filename,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    pub fn fix(msg: &str) {
        println!(
            "{}  {}WARN {}  {}{}{}",
            Self::timestamp(),
            COLOR_WARN,
            COLOR_RESET,
            msg,
            COLOR_RESET,
            ""
        );
    }
    fn timestamp() -> String {
        let now = SystemTime::now();
        let datetime: chrono::DateTime<chrono::Local> = now.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
