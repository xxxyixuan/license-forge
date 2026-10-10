//! 领域错误类型。
//!
//! 退出码约定（见设计文档 §3.4）：
//! - `0` 成功
//! - `1` 用户取消
//! - `2` 其它错误
//!
//! `main.rs` 通过 `LicError::exit_code()` 把错误映射成进程退出码，
//! 其它非 `LicError` 的 `anyhow` 错误一律按 `2` 处理。
//!
//! UI 文案统一使用英文。

use std::path::PathBuf;

/// license-forge 的领域错误。
#[derive(Debug, thiserror::Error)]
pub enum LicError {
    /// 找不到用户输入的许可证。
    #[error("unknown license `{input}`")]
    UnknownLicense {
        input: String,
        /// 基于 `strsim` 的相似名称建议，最多 3 条。
        suggestions: Vec<String>,
    },

    /// 目标文件已存在且不允许覆盖。
    #[error("file already exists: {}", path.display())]
    FileExists { path: PathBuf },

    /// 用户主动取消（退出码 1）。
    #[error("aborted by user")]
    Cancelled,
}

impl LicError {
    /// 该错误对应的进程退出码。
    pub fn exit_code(&self) -> u8 {
        match self {
            LicError::Cancelled => 1,
            _ => 2,
        }
    }

    /// 面向用户的补充提示（可为空）。
    pub fn hint(&self) -> Option<String> {
        match self {
            LicError::UnknownLicense { suggestions, .. } if !suggestions.is_empty() => {
                Some(format!(
                    "did you mean: {}? (run `lic list` for every built-in license)",
                    suggestions.join(", ")
                ))
            }
            LicError::UnknownLicense { .. } => {
                Some("run `lic list` for every built-in license".to_string())
            }
            LicError::FileExists { .. } => {
                Some("pass `--force` to overwrite (or answer `y` when prompted)".to_string())
            }
            LicError::Cancelled => None,
        }
    }
}
