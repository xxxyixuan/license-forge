//! `lic list` / `lic ls` / `lic --list`：列出全部内置许可证。

use anyhow::Result;

use crate::cli::GenerateOpts;
use crate::commands::emit;
use crate::core::catalog;
use crate::ui::format as fmt;

/// 执行列出命令。
pub fn run(opts: &GenerateOpts) -> Result<()> {
    let rendered = fmt::render_list(catalog::all(), opts.format);
    emit(&rendered, opts)
}
