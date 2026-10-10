//! 终端交互：目前只有「是否覆盖已存在文件」这一个确认。

use std::io::{BufRead, IsTerminal, Write};
use std::path::Path;

/// 询问用户是否覆盖 `path`。
///
/// 返回值：
/// - `Some(true)`：用户同意覆盖
/// - `Some(false)`：用户拒绝
/// - `None`：当前 stdin 不是终端（或读到 EOF），调用方应按非交互处理
pub fn ask_overwrite(path: &Path) -> std::io::Result<Option<bool>> {
    let stdin = std::io::stdin();
    if !stdin.is_terminal() {
        return Ok(None);
    }

    eprint!("file already exists: {}. Overwrite? [y/N] ", path.display());
    std::io::stderr().flush()?;

    let mut answer = String::new();
    let read = stdin.lock().read_line(&mut answer)?;
    if read == 0 {
        return Ok(None);
    }

    let answer = answer.trim().to_ascii_lowercase();
    Ok(Some(answer == "y" || answer == "yes"))
}
