//! `lic update` / `lic --update`：刷新模板缓存。
//!
//! 模板全部 `include_str!` 进二进制，没有磁盘缓存，因此这里没有可刷新的内容。
//! 本命令给出明确说明并成功退出（0），不假装做了什么。

use anyhow::Result;

use crate::cli::GenerateOpts;
use crate::commands::emit;
use crate::core::catalog;

/// 执行更新命令。
pub fn run(opts: &GenerateOpts) -> Result<()> {
    let message = format!(
        "license-forge {version} compiles all {count} templates into the binary,\n\
         so there is no cache to refresh.\n",
        version = env!("CARGO_PKG_VERSION"),
        count = catalog::all().len(),
    );
    emit(&message, opts)
}
