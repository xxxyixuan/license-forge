//! 智能默认值：作者名、年份、项目名。
//!
//! 作者名检测优先级（见设计文档 §2.3）：
//!
//! 1. `--author` / `-n` 命令行标志
//! 2. 环境变量 `LICENSE_FULL_NAME` 或 `LICENSE_AUTHOR`
//! 3. `git config user.name`
//! 4. hg config 用户名（`hg config ui.username`）
//! 5. 系统用户名
//! 6. 空字符串

use std::env;
use std::path::Path;
use std::process::Command;

/// 运行外部命令并取其标准输出的首行（已 trim）；失败返回 `None`。
fn command_stdout(program: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(program).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let trimmed = text.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// 读取第一个非空的环境变量。
fn first_env(keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| {
        env::var(key)
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// 系统用户名。
pub fn system_username() -> Option<String> {
    // Windows 用 USERNAME，Unix 用 USER；再退到 LOGNAME。
    if let Some(name) = first_env(&["USER", "USERNAME", "LOGNAME"]) {
        return Some(name);
    }
    // 兜底：调用 whoami。
    command_stdout("whoami", &[])
}

/// 按优先级检测作者名；全部来源都拿不到时返回空字符串。
pub fn detect_author(explicit: Option<&str>) -> String {
    if let Some(value) = explicit {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }

    if let Some(value) = first_env(&["LICENSE_FULL_NAME", "LICENSE_AUTHOR"]) {
        return value;
    }

    if let Some(value) = command_stdout("git", &["config", "user.name"]) {
        return value;
    }

    if let Some(value) = command_stdout("hg", &["config", "ui.username"]) {
        return value;
    }

    system_username().unwrap_or_default()
}

/// 年份：显式指定优先，否则取本地时区当前年份。
///
/// `time` 的 `local-offset` 在极少数环境下拿不到本地时区，此时回退到 UTC 年份。
pub fn detect_year(explicit: Option<&str>) -> String {
    if let Some(value) = explicit {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }

    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    now.year().to_string()
}

/// 项目名：显式指定优先，否则取当前目录的目录名。
pub fn detect_project(explicit: Option<&str>) -> String {
    if let Some(value) = explicit {
        let value = value.trim();
        if !value.is_empty() {
            return value.to_string();
        }
    }
    project_from_dir(env::current_dir().ok().as_deref())
}

/// 从目录路径推断项目名（独立成函数便于测试）。
fn project_from_dir(dir: Option<&Path>) -> String {
    dir.and_then(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_author_wins() {
        assert_eq!(detect_author(Some("Alice")), "Alice");
    }

    #[test]
    fn blank_explicit_author_falls_through_to_other_sources() {
        // 环境里没有 LICENSE_FULL_NAME / LICENSE_AUTHOR 时才会走到 git / 系统用户名；
        // 无论走到哪一步，都不应该是那段空白。
        let name = detect_author(Some("   "));
        assert!(!name.contains("   "));
        assert_ne!(name, "   ");
    }

    #[test]
    fn explicit_year_wins() {
        assert_eq!(detect_year(Some("1999")), "1999");
    }

    #[test]
    fn default_year_is_four_digits() {
        let year = detect_year(None);
        assert_eq!(year.len(), 4, "unexpected year: {year}");
        assert!(year.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn explicit_project_wins() {
        assert_eq!(detect_project(Some("myapp")), "myapp");
    }

    #[test]
    fn project_name_comes_from_directory() {
        assert_eq!(
            project_from_dir(Some(Path::new("/home/alice/myapp"))),
            "myapp"
        );
        assert_eq!(project_from_dir(None), "");
    }
}
