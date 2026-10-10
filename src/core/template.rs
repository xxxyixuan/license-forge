//! 模板渲染：把 `{{author}}` / `{{year}}` / `{{project}}` 替换成实际值。
//!
//! 按设计文档 §3.4 的要求，使用 `str::replace` 实现，不引入模板引擎。

/// 渲染时使用的变量集合。
#[derive(Debug, Clone, Copy)]
pub struct Vars<'a> {
    pub author: &'a str,
    pub year: &'a str,
    pub project: &'a str,
}

/// 模板中支持的占位符。
pub const PLACEHOLDERS: [&str; 3] = ["{{author}}", "{{year}}", "{{project}}"];

/// 渲染模板。未被模板使用的变量会被忽略。
pub fn render(template: &str, vars: &Vars<'_>) -> String {
    template
        .replace("{{author}}", vars.author)
        .replace("{{year}}", vars.year)
        .replace("{{project}}", vars.project)
}

/// 列出模板中实际出现的占位符（按固定顺序），供 `lic info` 展示。
pub fn used_placeholders(template: &str) -> Vec<&'static str> {
    PLACEHOLDERS
        .iter()
        .copied()
        .filter(|placeholder| template.contains(placeholder))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars() -> Vars<'static> {
        Vars {
            author: "Alice",
            year: "2025",
            project: "myapp",
        }
    }

    #[test]
    fn replaces_all_placeholders() {
        let out = render("{{project}} (c) {{year}} {{author}}", &vars());
        assert_eq!(out, "myapp (c) 2025 Alice");
    }

    #[test]
    fn replaces_repeated_placeholders() {
        let out = render("{{author}}/{{author}}", &vars());
        assert_eq!(out, "Alice/Alice");
    }

    #[test]
    fn leaves_unrelated_text_untouched() {
        let out = render("see <https://fsf.org/> and {{year}}", &vars());
        assert_eq!(out, "see <https://fsf.org/> and 2025");
    }

    #[test]
    fn empty_values_render_to_empty_string() {
        let out = render(
            "(c) {{year}} {{author}}",
            &Vars {
                author: "",
                year: "2025",
                project: "",
            },
        );
        assert_eq!(out, "(c) 2025 ");
    }

    #[test]
    fn detects_used_placeholders() {
        assert_eq!(
            used_placeholders("{{year}} then {{author}}"),
            vec!["{{author}}", "{{year}}"]
        );
        assert!(used_placeholders("no placeholders here").is_empty());
    }
}
