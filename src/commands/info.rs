//! `lic info <LICENSE>` / `lic --info <LICENSE>`：查看单个许可证详情。

use anyhow::Result;

use crate::cli::GenerateOpts;
use crate::commands::emit;
use crate::core::{catalog, resolve};
use crate::ui::format as fmt;

/// 执行详情命令。
pub fn run(name: &str, opts: &GenerateOpts) -> Result<()> {
    let license = resolve(catalog::all(), name)?;
    let rendered = fmt::render_info(license, opts.format);
    emit(&rendered, opts)
}
