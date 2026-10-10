//! 核心领域层：数据模型、内置目录、模板渲染、智能默认值、错误类型。
//!
//! 注意：模块名为 `core`，与 Rust 内置 crate 同名。在本 crate 内引用时
//! 一律写全路径 `crate::core::...`，避免路径歧义。

pub mod catalog;
pub mod defaults;
pub mod error;
pub mod license;
pub mod template;

pub use error::LicError;
pub use license::resolve;
pub use template::Vars;
