use std::fmt;

// 文件大小格式化器
#[derive(Debug, Clone)]
pub struct FileSize(pub u64);

impl fmt::Display for FileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const UNITS: [&str; 6] = ["B", "KB", "MB", "GB", "TB", "PB"];
        let mut size = self.0 as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            write!(f, "{} {}", size, UNITS[unit_index])
        } else {
            write!(f, "{:.2} {}", size, UNITS[unit_index])
        }
    }
}

impl From<u64> for FileSize {
    fn from(size: u64) -> Self {
        FileSize(size)
    }
}
