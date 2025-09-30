use std::cell::RefCell;
use std::collections::BTreeMap;

// 日志级别
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Success,
    Debug,
}

// 日志配置结构
#[derive(Debug)]
pub struct LogConfig {
    pub enabled: bool,
    pub colorize: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            enabled: std::env::var("CRUD_MACRO_DEBUG").is_ok(),
            colorize: true,
        }
    }
}

// 日志记录器
#[derive(Debug)]
pub struct MacroLogger {
    pub config: LogConfig,
    pub module_logs: BTreeMap<String, Vec<(LogLevel, String)>>,
}

impl MacroLogger {
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
            module_logs: BTreeMap::new(),
        }
    }

    pub fn log(&mut self, module: &str, level: LogLevel, message: String) {
        if !self.config.enabled {
            return;
        }

        self.module_logs
            .entry(module.to_string())
            .or_insert_with(Vec::new)
            .push((level, message));
    }

    pub fn flush(&self) {
        if !self.config.enabled || self.module_logs.is_empty() {
            return;
        }

        println!("\n{}", self.colorize("🚀 CRUD宏生成日志", LogLevel::Info));
        println!("{}", self.colorize(&"=".repeat(50), LogLevel::Info));

        for (module, logs) in &self.module_logs {
            println!(
                "\n{}",
                self.colorize(&format!("📦 模块: {}", module), LogLevel::Info)
            );
            println!("{}", self.colorize(&"-".repeat(40), LogLevel::Info));

            for (level, message) in logs {
                let prefix = match level {
                    LogLevel::Info => "ℹ️ ",
                    LogLevel::Success => "✅",
                    LogLevel::Debug => "🐛",
                };
                println!("{} {}", prefix, self.colorize(message, *level));
            }
        }

        println!("\n{}", self.colorize(&"=".repeat(50), LogLevel::Success));
        println!(
            "{}",
            self.colorize(
                &format!("✨ 共生成 {} 个模块", self.module_logs.len()),
                LogLevel::Success
            )
        );
    }

    fn colorize(&self, text: &str, level: LogLevel) -> String {
        if !self.config.colorize {
            return text.to_string();
        }

        let color_code = match level {
            LogLevel::Info => "36",    // 青色
            LogLevel::Success => "32", // 绿色
            LogLevel::Debug => "35",   // 紫色
        };

        format!("\x1b[{}m{}\x1b[0m", color_code, text)
    }
}

// 全局日志记录器
thread_local! {
    pub static LOGGER: RefCell<MacroLogger> = RefCell::new(MacroLogger::new());
}

/// 初始化日志记录器
// pub fn init_route_logger(config: Option<LogConfig>) {
//     LOGGER.with(|logger| {
//         if let Some(cfg) = config {
//             logger.borrow_mut().config = cfg;
//         }
//     });
// }

pub fn flush_crud_logs(_input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    LOGGER.with(|logger| {
        logger.borrow().flush();
    });

    quote::quote! {}.into()
}
