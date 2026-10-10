//! 子命令实现与统一分发。

pub mod generate;
pub mod info;
pub mod list;
pub mod update;

use std::fs;
use std::io::{self, Write};
use std::path::Path;

use anyhow::{bail, Context, Result};
use clap::CommandFactory;

use crate::cli::{Cli, Command, GenerateOpts};
use crate::core::LicError;
use crate::ui::prompts;

/// 按「子命令 > 等价标志 > 位置参数」的优先级分发。
pub fn run(cli: Cli) -> Result<()> {
    if let Some(command) = cli.command.as_ref() {
        return match command {
            Command::List => list::run(&cli.opts),
            Command::Info { license } => info::run(license, &cli.opts),
            Command::Update => update::run(&cli.opts),
        };
    }

    if cli.list {
        return list::run(&cli.opts);
    }
    if let Some(name) = cli.info.as_ref() {
        return info::run(name, &cli.opts);
    }
    if cli.update {
        return update::run(&cli.opts);
    }

    match cli.license.as_ref() {
        Some(name) => generate::run(name, &cli.opts),
        None => {
            print_quick_start();
            Ok(())
        }
    }
}

/// `lic` 不带任何参数时：打印帮助与快速上手。
fn print_quick_start() {
    let mut command = Cli::command();
    let _ = command.print_help();
    println!("\n{}", Cli::quick_start());
}

/// 统一的输出出口：没有 `-o` 就写 stdout，有就写文件（含覆盖策略）。
pub fn emit(content: &str, opts: &GenerateOpts) -> Result<()> {
    match opts.output.as_deref() {
        None => {
            let stdout = io::stdout();
            let mut lock = stdout.lock();
            lock.write_all(content.as_bytes())?;
            lock.flush()?;
            Ok(())
        }
        Some(path) => write_file(path, content, opts),
    }
}

/// 写入文件，并按 `--force` / `--yes` / TTY 决定覆盖策略。
fn write_file(path: &Path, content: &str, opts: &GenerateOpts) -> Result<()> {
    if path.is_dir() {
        bail!("{} is a directory", path.display());
    }

    if path.exists() && !opts.force {
        if opts.yes {
            // 非交互模式不提问，必须显式 --force
            return Err(LicError::FileExists {
                path: path.to_path_buf(),
            }
            .into());
        }
        match prompts::ask_overwrite(path)? {
            Some(true) => {}
            Some(false) => return Err(LicError::Cancelled.into()),
            // 非 TTY：按非交互处理
            None => {
                return Err(LicError::FileExists {
                    path: path.to_path_buf(),
                }
                .into())
            }
        }
    }

    fs::write(path, content).with_context(|| format!("failed to write {}", path.display()))?;
    eprintln!("wrote {} ({} bytes)", path.display(), content.len());
    Ok(())
}
