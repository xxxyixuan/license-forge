//! 命令行接口定义（clap derive）。
//!
//! 解析策略（设计文档 §2.2）：位置参数 `<LICENSE>` 与子命令共存。
//! clap 天然优先匹配子命令，因此 `lic info mit` 会走 `info` 子命令，
//! 而 `lic mit` 会把 `mit` 当作位置参数执行生成命令。
//! 同时提供 `--list` / `--info <LICENSE>` / `--update` 三个等价标志。

use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use crate::ui::format::Format;

const LONG_ABOUT: &str = "\
Forge your LICENSE in seconds.

Writes the license text to stdout by default; use `-o` to write a file.
Author, year and project all have smart defaults, so `lic mit` just works.

All 17 templates are compiled into the binary, so `lic` never touches the
network and needs no cache.";

/// `lic` 的顶层命令行参数。
#[derive(Debug, Parser)]
#[command(
    name = "lic",
    version,
    about = "Forge your LICENSE in seconds.",
    long_about = LONG_ABOUT,
    // 让 `lic list` 走子命令而不是把 "list" 当成许可证名
    subcommand_precedence_over_arg = true,
    // 不提供 `lic help` 子命令，统一用 `--help`
    disable_help_subcommand = true,
    max_term_width = 100
)]
pub struct Cli {
    /// License name: an SPDX identifier or an alias (e.g. mit, apache-2.0, mulanpsl-2.0)
    #[arg(value_name = "LICENSE")]
    pub license: Option<String>,

    #[command(flatten)]
    pub opts: GenerateOpts,

    /// List every built-in license (same as `lic list`)
    #[arg(long)]
    pub list: bool,

    /// Show details for one license (same as `lic info <LICENSE>`)
    #[arg(long, value_name = "LICENSE")]
    pub info: Option<String>,

    /// Refresh the template cache (same as `lic update`)
    #[arg(long)]
    pub update: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

/// 子命令。`ls` 是 `list` 的别名。
#[derive(Debug, Subcommand)]
pub enum Command {
    /// List every built-in license
    #[command(visible_alias = "ls")]
    List,

    /// Show details for one license
    Info {
        /// License name: an SPDX identifier or an alias
        #[arg(value_name = "LICENSE")]
        license: String,
    },

    /// Refresh the template cache
    Update,
}

/// 生成命令与其它命令共用的选项。
///
/// 全部标记为 `global`，因此 `lic info mit --format json` 这类写法同样有效。
#[derive(Debug, Args)]
pub struct GenerateOpts {
    /// Copyright holder. Defaults to LICENSE_FULL_NAME / LICENSE_AUTHOR, git config, hg config, then the system user
    #[arg(short = 'n', long, global = true, value_name = "AUTHOR")]
    pub author: Option<String>,

    /// Copyright year. Defaults to the current year
    #[arg(short = 'y', long, global = true, value_name = "YEAR")]
    pub year: Option<String>,

    /// Project name. Defaults to the name of the current directory
    #[arg(short = 'p', long, global = true, value_name = "PROJECT")]
    pub project: Option<String>,

    /// Write to FILE instead of stdout
    #[arg(short = 'o', long, global = true, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format
    #[arg(
        long,
        global = true,
        value_enum,
        value_name = "FORMAT",
        default_value_t = Format::Text
    )]
    pub format: Format,

    /// Non-interactive: never prompt. An existing file still needs `--force`
    #[arg(long, global = true)]
    pub yes: bool,

    /// Overwrite the output file if it already exists
    #[arg(long, global = true)]
    pub force: bool,

    /// Use only built-in templates and cache
    #[arg(long, global = true)]
    pub offline: bool,

    /// GitHub API credentials: `token` or `username:token` (accepted, not used yet)
    #[arg(
        long,
        global = true,
        value_name = "AUTH",
        env = "GITHUB_TOKEN",
        hide_env_values = true
    )]
    pub auth: Option<String>,

    /// Template cache TTL in days (accepted, not used yet)
    #[arg(long, global = true, value_name = "DAYS", default_value_t = 30)]
    pub cache_ttl: u32,
}

impl Cli {
    /// 供 `lic` 无参数时打印的快速上手说明。
    pub fn quick_start() -> String {
        "\
Quick start:
  lic mit                        # print the MIT license to stdout
  lic apache-2.0 -o LICENSE      # write it to a LICENSE file
  lic --author \"Alice\" -y 2025 mit
  lic mulanpsl-2.0               # Mulan PSL v2 (木兰宽松许可证 第2版)
  lic list                       # list every built-in license
  lic info mit                   # show details for one license

Run `lic --help` for all options."
            .to_string()
    }
}
