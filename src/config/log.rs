use log::Level;
use log4rs::{
    append::console::ConsoleAppender,
    append::rolling_file::RollingFileAppender,
    append::rolling_file::policy::compound::CompoundPolicy,
    append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller,
    append::rolling_file::policy::compound::trigger::size::SizeTrigger,
    config::{Appender, Config, Logger, Root},
    encode::pattern::PatternEncoder,
};

use log::LevelFilter;
use nu_ansi_term::Color;

struct ColorEncoder;
impl std::fmt::Debug for ColorEncoder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("ColorEncoder")
    }
}

impl log4rs::encode::Encode for ColorEncoder {
    fn encode(
        &self,
        buf: &mut dyn log4rs::encode::Write,
        record: &log::Record,
    ) -> Result<(), anyhow::Error> {
        let colored_message = match record.level() {
            Level::Info => Color::Green.paint(format!(
                "INFO - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.args()
            )),
            Level::Error => Color::Red.paint(format!(
                "ERROR - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.args()
            )),
            Level::Debug => Color::Blue.paint(format!(
                "DEBUG - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.args()
            )),
            Level::Warn => Color::Yellow.paint(format!(
                "WARN - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.args()
            )),
            Level::Trace => Color::Purple.paint(format!(
                "TRACE - {} - {} - {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.target(),
                record.args()
            )),
        };

        writeln!(buf, "{}", colored_message).map_err(anyhow::Error::new)
    }
}

pub fn init_logger() {
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{l} - {d(%Y-%m-%d %H:%M:%S)} - {m}{n}",
        )))
        .encoder(Box::new(ColorEncoder))
        .build();

    // Preserve the existing default; allow controlled diagnostics without recompiling.
    let roll_bytes = std::env::var("LOG_ROLL_BYTES")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|v| *v >= 1024)
        .unwrap_or(1024 * 10);
    let size_trigger = SizeTrigger::new(roll_bytes);
    let window_roller = FixedWindowRoller::builder()
        .base(1)
        .build("logs/test.{}.log", 30)
        .unwrap();

    let compound_policy = CompoundPolicy::new(Box::new(size_trigger), Box::new(window_roller));
    let file_appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{l} - {d(%Y-%m-%d %H:%M:%S)} - {m}{n}",
        )))
        .build("logs/log.log", Box::new(compound_policy))
        .unwrap();

    let enabled = |name: &str| std::env::var(name).map(|v| v != "false").unwrap_or(true);
    let config = Config::builder()
        .logger(Logger::builder().build(
            "http_access",
            if enabled("HTTP_ACCESS_LOG_ENABLED") {
                LevelFilter::Info
            } else {
                LevelFilter::Off
            },
        ))
        .logger(Logger::builder().build(
            "connection",
            if enabled("CONNECTION_LOG_ENABLED") {
                LevelFilter::Info
            } else {
                LevelFilter::Off
            },
        ))
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("file", Box::new(file_appender)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("file")
                .build(LevelFilter::Info),
        )
        .unwrap();

    let _handle = log4rs::init_config(config).unwrap();
}
