//! 许可证的数据模型。

use serde::Serialize;

use crate::core::error::LicError;

/// 许可证分类。用于 `lic list` 分组与 `lic info` 展示。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Category {
    /// 宽松许可证：允许闭源再分发，仅要求保留声明。
    Permissive,
    /// 弱著佐权：按文件/库粒度要求开源，允许更大作品采用其它条款。
    WeakCopyleft,
    /// 强著佐权：衍生作品必须同许可开源。
    StrongCopyleft,
    /// 公共领域奉献：放弃著作权，几乎无附加条件。
    PublicDomain,
}

impl Category {
    /// 按 `lic list` 的展示顺序排列的全部分类。
    pub const ALL: [Category; 4] = [
        Category::Permissive,
        Category::WeakCopyleft,
        Category::StrongCopyleft,
        Category::PublicDomain,
    ];

    /// 英文标签。
    pub fn label(self) -> &'static str {
        match self {
            Category::Permissive => "Permissive",
            Category::WeakCopyleft => "Weak Copyleft",
            Category::StrongCopyleft => "Strong Copyleft",
            Category::PublicDomain => "Public Domain",
        }
    }
}

/// 一个内置许可证模板及其元数据。
#[derive(Debug)]
pub struct License {
    /// CLI 主键，同时是模板文件名（`src/templates/<key>.txt`）。
    pub key: &'static str,
    /// 额外的可接受写法（不含 key 与 SPDX 标识符本身）。
    pub aliases: &'static [&'static str],
    /// SPDX 标识符。
    pub spdx_id: &'static str,
    /// 英文显示名。
    pub name: &'static str,
    /// 原生/中文名（木兰系列以中文为正式名称）。
    pub name_zh: &'static str,
    /// 分类。
    pub category: Category,
    /// 一句话说明。
    pub description: &'static str,
    /// 授予的权利。
    pub permissions: &'static [&'static str],
    /// 使用条件。
    pub conditions: &'static [&'static str],
    /// 免责与限制。
    pub limitations: &'static [&'static str],
    /// 许可证全文模板（`include_str!` 打包进二进制）。
    pub template: &'static str,
}

impl License {
    /// 该模板是否使用 `{{author}}` 占位符。
    ///
    /// 用来判断「检测不到作者名」时要不要提醒用户。
    pub fn needs_author(&self) -> bool {
        self.template.contains("{{author}}")
    }
}

/// 把用户输入归一化成便于比较的形式：去首尾空白、转小写。
fn normalize(input: &str) -> String {
    input.trim().to_lowercase()
}

/// 按 key / SPDX 标识符 / 别名精确匹配。
pub fn find_exact(catalog: &'static [License], input: &str) -> Option<&'static License> {
    let needle = normalize(input);
    if needle.is_empty() {
        return None;
    }
    catalog.iter().find(|lic| {
        lic.key == needle
            || lic.spdx_id.to_lowercase() == needle
            || lic.aliases.iter().any(|alias| *alias == needle)
    })
}

/// 用 `strsim` 给出相似名称建议（按相似度降序，最多 `limit` 条）。
pub fn suggest(catalog: &'static [License], input: &str, limit: usize) -> Vec<String> {
    let needle = normalize(input);
    if needle.is_empty() {
        return Vec::new();
    }

    // 相似度阈值：低于该值视为“完全不像”，不给出建议。
    const THRESHOLD: f64 = 0.75;

    let mut scored: Vec<(f64, &'static str)> = Vec::new();
    for lic in catalog {
        let mut best = strsim::jaro_winkler(&needle, lic.key);
        for candidate in std::iter::once(lic.spdx_id).chain(lic.aliases.iter().copied()) {
            let score = strsim::jaro_winkler(&needle, &candidate.to_lowercase());
            if score > best {
                best = score;
            }
        }
        if best >= THRESHOLD {
            scored.push((best, lic.key));
        }
    }

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(limit);
    scored.into_iter().map(|(_, key)| key.to_string()).collect()
}

/// 解析用户输入的许可证名；失败时附带相似名称建议。
pub fn resolve(catalog: &'static [License], input: &str) -> Result<&'static License, LicError> {
    find_exact(catalog, input).ok_or_else(|| LicError::UnknownLicense {
        input: input.trim().to_string(),
        suggestions: suggest(catalog, input, 3),
    })
}
