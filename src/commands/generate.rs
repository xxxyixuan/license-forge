//! `lic <LICENSE>`：生成许可证文本。
//!
//! 流程：解析许可证名 -> 计算智能默认值 -> 渲染模板 -> 按 `--format` 格式化 ->
//! 写 stdout 或文件。

use anyhow::Result;

use crate::cli::GenerateOpts;
use crate::commands::emit;
use crate::core::license::License;
use crate::core::{catalog, defaults, resolve, template, Vars};
use crate::ui::format as fmt;

/// 执行生成命令。
pub fn run(name: &str, opts: &GenerateOpts) -> Result<()> {
    let license = resolve(catalog::all(), name)?;

    let author = defaults::detect_author(opts.author.as_deref());
    let year = defaults::detect_year(opts.year.as_deref());
    let project = defaults::detect_project(opts.project.as_deref());

    let vars = Vars {
        author: &author,
        year: &year,
        project: &project,
    };

    let body = template::render(license.template, &vars);
    warn_about_missing_author(license, &author);

    let rendered = fmt::render_license(license, &body, &vars, opts.format);
    emit(&rendered, opts)
}

/// 模板需要作者名、但所有来源都没能给出时，提醒用户（不阻断生成）。
fn warn_about_missing_author(license: &License, author: &str) {
    if license.needs_author() && author.is_empty() {
        eprintln!(
            "warning: could not detect an author name, so {{{{author}}}} renders as empty.\n\
             warning: pass `--author <AUTHOR>` or set LICENSE_FULL_NAME to fill it in."
        );
    }
}
