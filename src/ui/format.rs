//! 输出格式：纯文本 / Markdown / JSON。
//!
//! 设计文档 §2.6：
//! - 纯文本：默认，写到 stdout
//! - Markdown：包含许可证名称、SPDX 标识符和文本
//! - JSON：结构化输出，字段包含 `spdx_id`、`name`、`text`、`author`、`year`、
//!   `project`、`permissions`、`conditions`、`limitations`

use std::fmt;

use clap::ValueEnum;
use serde::Serialize;

use crate::core::license::{Category, License};
use crate::core::template::{self, Vars};

/// `--format` 的取值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Format {
    /// Plain text (default)
    Text,
    /// Markdown with a metadata header
    Markdown,
    /// Structured JSON
    Json,
}

impl Format {
    /// 与 `ValueEnum` 一致的字符串形式。
    pub fn slug(self) -> &'static str {
        match self {
            Format::Text => "text",
            Format::Markdown => "markdown",
            Format::Json => "json",
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.slug())
    }
}

/// `lic <LICENSE>` 的 JSON 结构。
#[derive(Debug, Serialize)]
struct GeneratedJson<'a> {
    key: &'a str,
    spdx_id: &'a str,
    name: &'a str,
    name_zh: &'a str,
    category: Category,
    text: &'a str,
    author: &'a str,
    year: &'a str,
    project: &'a str,
    permissions: &'a [&'a str],
    conditions: &'a [&'a str],
    limitations: &'a [&'a str],
}

/// `lic list` 的 JSON 条目。
#[derive(Debug, Serialize)]
struct ListItemJson<'a> {
    key: &'a str,
    spdx_id: &'a str,
    name: &'a str,
    name_zh: &'a str,
    category: Category,
    description: &'a str,
}

/// `lic info <LICENSE>` 的 JSON 结构。
#[derive(Debug, Serialize)]
struct InfoJson<'a> {
    key: &'a str,
    spdx_id: &'a str,
    name: &'a str,
    name_zh: &'a str,
    category: Category,
    description: &'a str,
    aliases: &'a [&'a str],
    placeholders: Vec<&'static str>,
    template_bytes: usize,
    permissions: &'a [&'a str],
    conditions: &'a [&'a str],
    limitations: &'a [&'a str],
}

/// 把 JSON 值序列化成带换行的字符串。
fn to_pretty_json<T: Serialize>(value: &T) -> String {
    let mut out = serde_json::to_string_pretty(value).expect("serializing to JSON cannot fail");
    out.push('\n');
    out
}

/// 渲染 `lic <LICENSE>` 的输出。
pub fn render_license(license: &License, body: &str, vars: &Vars<'_>, format: Format) -> String {
    match format {
        Format::Text => body.to_string(),
        Format::Markdown => {
            let mut out = String::new();
            out.push_str(&format!("# {}\n\n", license.name));
            out.push_str(&format!("- **SPDX ID**: `{}`\n", license.spdx_id));
            out.push_str(&format!("- **Author**: {}\n", vars.author));
            out.push_str(&format!("- **Year**: {}\n", vars.year));
            out.push_str(&format!("- **Project**: {}\n", vars.project));
            out.push_str("\n---\n\n");
            out.push_str(body);
            if !out.ends_with('\n') {
                out.push('\n');
            }
            out
        }
        Format::Json => to_pretty_json(&GeneratedJson {
            key: license.key,
            spdx_id: license.spdx_id,
            name: license.name,
            name_zh: license.name_zh,
            category: license.category,
            text: body,
            author: vars.author,
            year: vars.year,
            project: vars.project,
            permissions: license.permissions,
            conditions: license.conditions,
            limitations: license.limitations,
        }),
    }
}

/// 渲染 `lic list` 的输出。
pub fn render_list(licenses: &'static [License], format: Format) -> String {
    match format {
        Format::Text => {
            let mut out = format!(
                "license-forge {} · {} built-in licenses\n",
                env!("CARGO_PKG_VERSION"),
                licenses.len()
            );
            for category in Category::ALL {
                let group: Vec<&License> = licenses
                    .iter()
                    .filter(|lic| lic.category == category)
                    .collect();
                if group.is_empty() {
                    continue;
                }
                out.push_str(&format!("\n{}\n", category.label()));
                for lic in group {
                    out.push_str(&format!("  {:<14}{}\n", lic.key, lic.name));
                }
            }
            out.push_str(
                "\nUse `lic <LICENSE>` to generate one, `lic info <LICENSE>` for details.\n",
            );
            out.push_str(
                "SPDX identifiers (e.g. `lic MIT`) and aliases (e.g. `lic apache`) work too.\n",
            );
            out
        }
        Format::Markdown => {
            let mut out =
                String::from("| Category | Key | SPDX ID | Name |\n| --- | --- | --- | --- |\n");
            for lic in licenses {
                out.push_str(&format!(
                    "| {} | `{}` | `{}` | {} |\n",
                    lic.category.label(),
                    lic.key,
                    lic.spdx_id,
                    lic.name
                ));
            }
            out
        }
        Format::Json => {
            let items: Vec<ListItemJson<'_>> = licenses
                .iter()
                .map(|lic| ListItemJson {
                    key: lic.key,
                    spdx_id: lic.spdx_id,
                    name: lic.name,
                    name_zh: lic.name_zh,
                    category: lic.category,
                    description: lic.description,
                })
                .collect();
            to_pretty_json(&items)
        }
    }
}

/// 渲染 `lic info <LICENSE>` 的输出。
pub fn render_info(license: &License, format: Format) -> String {
    let placeholders = template::used_placeholders(license.template);

    match format {
        Format::Text => {
            let mut out = format!("{}\n", license.name);
            if license.name_zh != license.name {
                out.push_str(&format!("  {}\n", license.name_zh));
            }
            out.push('\n');
            out.push_str(&format!("  {:<13}{}\n", "SPDX ID", license.spdx_id));
            out.push_str(&format!("  {:<13}{}\n", "Key", license.key));
            out.push_str(&format!(
                "  {:<13}{}\n",
                "Category",
                license.category.label()
            ));
            out.push_str(&format!(
                "  {:<13}{}\n",
                "Aliases",
                if license.aliases.is_empty() {
                    "-".to_string()
                } else {
                    license.aliases.join(", ")
                }
            ));
            out.push_str(&format!(
                "  {:<13}{}\n",
                "Placeholders",
                if placeholders.is_empty() {
                    "-".to_string()
                } else {
                    placeholders.join(", ")
                }
            ));
            out.push_str(&format!(
                "  {:<13}{} bytes\n",
                "Template",
                license.template.len()
            ));
            out.push('\n');
            out.push_str(&wrap(license.description, 74, "  "));
            out.push_str(&section("Permissions", license.permissions));
            out.push_str(&section("Conditions", license.conditions));
            out.push_str(&section("Limitations", license.limitations));
            out.push_str(&format!("\nGenerate it with: lic {}\n", license.key));
            out
        }
        Format::Markdown => {
            let mut out = format!("# {}\n\n", license.name);
            out.push_str(&format!("{}\n\n", license.description));
            out.push_str("| Field | Value |\n| --- | --- |\n");
            out.push_str(&format!("| SPDX ID | `{}` |\n", license.spdx_id));
            out.push_str(&format!("| Key | `{}` |\n", license.key));
            out.push_str(&format!("| Category | {} |\n", license.category.label()));
            out.push_str(&format!(
                "| Permissions | {} |\n",
                join_or_dash(license.permissions)
            ));
            out.push_str(&format!(
                "| Conditions | {} |\n",
                join_or_dash(license.conditions)
            ));
            out.push_str(&format!(
                "| Limitations | {} |\n",
                join_or_dash(license.limitations)
            ));
            out
        }
        Format::Json => to_pretty_json(&InfoJson {
            key: license.key,
            spdx_id: license.spdx_id,
            name: license.name,
            name_zh: license.name_zh,
            category: license.category,
            description: license.description,
            aliases: license.aliases,
            placeholders,
            template_bytes: license.template.len(),
            permissions: license.permissions,
            conditions: license.conditions,
            limitations: license.limitations,
        }),
    }
}

/// 把 `-` 或逗号分隔的列表渲染成一行。
fn join_or_dash(items: &[&str]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

/// 渲染 `Permissions` / `Conditions` / `Limitations` 小节。
fn section(title: &str, items: &[&str]) -> String {
    format!("\n{}\n  {}\n", title, join_or_dash(items))
}

/// 简单的按词换行，保持 `lic info` 的输出不被终端折行打乱。
fn wrap(text: &str, width: usize, indent: &str) -> String {
    let mut out = String::new();
    let mut line_len = 0usize;
    for word in text.split_whitespace() {
        if line_len == 0 {
            out.push_str(indent);
            out.push_str(word);
            line_len = word.len();
        } else if line_len + 1 + word.len() > width {
            out.push('\n');
            out.push_str(indent);
            out.push_str(word);
            line_len = word.len();
        } else {
            out.push(' ');
            out.push_str(word);
            line_len += 1 + word.len();
        }
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::catalog;

    fn mit() -> &'static License {
        catalog::all()
            .iter()
            .find(|lic| lic.key == "mit")
            .expect("mit must exist")
    }

    #[test]
    fn text_format_is_just_the_body() {
        let vars = Vars {
            author: "Alice",
            year: "2025",
            project: "myapp",
        };
        let body = template::render(mit().template, &vars);
        assert_eq!(render_license(mit(), &body, &vars, Format::Text), body);
    }

    #[test]
    fn markdown_format_has_header_and_body() {
        let vars = Vars {
            author: "Alice",
            year: "2025",
            project: "myapp",
        };
        let body = template::render(mit().template, &vars);
        let out = render_license(mit(), &body, &vars, Format::Markdown);
        assert!(out.starts_with("# MIT License\n"));
        assert!(out.contains("- **SPDX ID**: `MIT`"));
        assert!(out.contains("- **Author**: Alice"));
        assert!(out.ends_with(&body));
    }

    #[test]
    fn json_format_has_the_documented_fields() {
        let vars = Vars {
            author: "Alice",
            year: "2025",
            project: "myapp",
        };
        let body = template::render(mit().template, &vars);
        let out = render_license(mit(), &body, &vars, Format::Json);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value["spdx_id"], "MIT");
        assert_eq!(value["name"], "MIT License");
        assert_eq!(value["author"], "Alice");
        assert_eq!(value["year"], "2025");
        assert_eq!(value["project"], "myapp");
        assert_eq!(value["category"], "permissive");
        assert_eq!(value["permissions"][0], "commercial-use");
        assert!(value["text"].as_str().unwrap().contains("Alice"));
    }

    #[test]
    fn list_text_groups_by_category() {
        let out = render_list(catalog::all(), Format::Text);
        for category in Category::ALL {
            assert!(
                out.contains(category.label()),
                "missing {}",
                category.label()
            );
        }
        assert!(out.contains("mulanpsl-2.0"));
    }

    #[test]
    fn list_json_covers_every_license() {
        let out = render_list(catalog::all(), Format::Json);
        let value: serde_json::Value = serde_json::from_str(&out).expect("valid JSON");
        assert_eq!(value.as_array().unwrap().len(), catalog::all().len());
    }

    #[test]
    fn info_text_reports_placeholders() {
        let out = render_info(mit(), Format::Text);
        assert!(out.contains("{{author}}"));
        assert!(out.contains("{{year}}"));
        assert!(out.contains("commercial-use"));
    }
}
