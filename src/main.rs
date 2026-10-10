//! license-forge —— *Forge your LICENSE in seconds.*
//!
//! 二进制名 `lic`。0.1.0（MVP）范围：
//! - clap 解析位置参数 `<LICENSE>` 与 `list` / `ls` / `info` / `update` 子命令
//! - 17 个许可证模板用 `include_str!` 内置，完全离线
//! - 生成到 stdout 或文件，支持 text / markdown / json
//! - 作者名、年份、项目名的智能默认值
//!
//! 退出码：`0` 成功，`1` 用户取消，`2` 错误（设计文档 §3.4）。

mod cli;
mod commands;
mod core;
mod ui;

use std::process::ExitCode;

use clap::Parser;

use crate::cli::Cli;
use crate::core::LicError;

fn main() -> ExitCode {
    let cli = Cli::parse();

    match commands::run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // 用户取消不是「出错」，单独给一行更平和的提示，仍然返回 1。
            if matches!(err.downcast_ref::<LicError>(), Some(LicError::Cancelled)) {
                eprintln!("aborted.");
                return ExitCode::from(1);
            }

            let code = err
                .downcast_ref::<LicError>()
                .map(LicError::exit_code)
                .unwrap_or(2);

            eprintln!("error: {err}");
            if let Some(lic_err) = err.downcast_ref::<LicError>() {
                if let Some(hint) = lic_err.hint() {
                    eprintln!("hint: {hint}");
                }
            }
            // anyhow 的 context 链，便于定位 IO 失败的具体原因。
            for cause in err.chain().skip(1) {
                eprintln!("cause: {cause}");
            }

            ExitCode::from(code)
        }
    }
}
