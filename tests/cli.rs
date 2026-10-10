//! 端到端 CLI 测试。
//!
//! 覆盖 0.1.0（MVP）验收点：
//! - 生成到 stdout / 文件
//! - 作者、年份、项目名的智能默认值与命令行覆盖
//! - 17 个内置许可证全部可渲染（含木兰 MulanPSL-2.0）
//! - `list` / `ls` / `--list`、`info` / `--info`、`update` / `--update` 等价形式
//! - text / markdown / json 三种输出格式
//! - 无效许可证名的模糊匹配建议
//! - 退出码 0 / 1 / 2
//!
//! 关于测试驱动：设计文档 §3.1 建议用 `assert_cmd`，但它在 `spawn` 时会无条件给
//! 子进程开一个 stdin 管道（`cmd.rs` 里 `self.cmd.stdin(Stdio::piped())`）。
//! 在部分 Windows 环境（含本项目的开发机）下创建 stdin 管道会直接失败并报
//! `os error 231 (ERROR_PIPE_BUSY)`，导致所有用例无法运行。因此这里改用
//! `std::process::Command` + cargo 注入的 `CARGO_BIN_EXE_lic`，
//! 行为等价、零额外依赖，且在任何环境下都能跑。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// 全部 17 个内置许可证的 key。
const KEYS: [&str; 17] = [
    "mit",
    "apache-2.0",
    "gpl-3.0",
    "agpl-3.0",
    "lgpl-3.0",
    "mpl-2.0",
    "bsd-3-clause",
    "bsd-2-clause",
    "bsd-4-clause",
    "isc",
    "zlib",
    "bsl-1.0",
    "artistic-2.0",
    "unlicense",
    "cc0-1.0",
    "wtfpl",
    "mulanpsl-2.0",
];

/// 一次 `lic` 调用的结果。
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    /// 断言退出码，并在失败时把完整输出打出来，便于定位。
    fn expect_code(&self, expected: i32) -> &Self {
        assert_eq!(
            self.code, expected,
            "unexpected exit code\n--- stdout ---\n{}\n--- stderr ---\n{}",
            self.stdout, self.stderr
        );
        self
    }

    fn expect_stdout_contains(&self, needle: &str) -> &Self {
        assert!(
            self.stdout.contains(needle),
            "stdout does not contain {needle:?}\n--- stdout ---\n{}",
            self.stdout
        );
        self
    }

    fn expect_stderr_contains(&self, needle: &str) -> &Self {
        assert!(
            self.stderr.contains(needle),
            "stderr does not contain {needle:?}\n--- stderr ---\n{}",
            self.stderr
        );
        self
    }

    fn expect_stdout_empty(&self) -> &Self {
        assert!(
            self.stdout.is_empty(),
            "stdout should be empty, got:\n{}",
            self.stdout
        );
        self
    }

    /// 把 stdout 解析成 JSON。
    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|err| {
            panic!(
                "stdout is not valid JSON: {err}\n--- stdout ---\n{}",
                self.stdout
            )
        })
    }
}

/// 运行 `lic`，清掉会影响智能默认值判定的环境变量，保证结果不依赖开发机的 git 配置。
fn run(args: &[&str], cwd: Option<&Path>, envs: &[(&str, &str)]) -> Run {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_lic"));
    cmd.args(args);
    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }
    for key in [
        "LICENSE_FULL_NAME",
        "LICENSE_AUTHOR",
        "GITHUB_TOKEN",
        "LICENSE_FORGE_CACHE_DIR",
    ] {
        cmd.env_remove(key);
    }
    for (key, value) in envs {
        cmd.env(key, value);
    }

    let output = cmd.output().expect("failed to run `lic`");
    Run {
        code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    }
}

/// 在当前目录运行。
fn lic(args: &[&str]) -> Run {
    run(args, None, &[])
}

/// 在指定目录运行。
fn lic_in(dir: &Path, args: &[&str]) -> Run {
    run(args, Some(dir), &[])
}

/// 把 `PathBuf` 转成命令行参数（测试里的临时目录都是合法 UTF-8）。
fn arg(path: &Path) -> &str {
    path.to_str().expect("temp paths are valid UTF-8")
}

// ---------------------------------------------------------------------------
// 生成
// ---------------------------------------------------------------------------

#[test]
fn every_builtin_license_renders() {
    for key in KEYS {
        let run = lic(&[
            "--author",
            "Alice",
            "--year",
            "2025",
            "--project",
            "myapp",
            key,
        ]);
        run.expect_code(0);
        assert!(
            run.stdout.len() > 200,
            "`{key}` produced suspiciously little output"
        );
        assert!(
            !run.stdout.contains("{{"),
            "`{key}` left an unrendered placeholder"
        );
    }
}

#[test]
fn mit_goes_to_stdout_by_default() {
    lic(&["--author", "Alice", "--year", "2025", "mit"])
        .expect_code(0)
        .expect_stdout_contains("MIT License")
        .expect_stdout_contains("Copyright (c) 2025 Alice")
        .expect_stdout_contains("Permission is hereby granted");
}

#[test]
fn writes_to_file_with_short_and_long_flag() {
    let dir = tempfile::tempdir().unwrap();

    lic_in(
        dir.path(),
        &[
            "--author", "Alice", "--year", "2025", "mit", "-o", "LICENSE",
        ],
    )
    .expect_code(0)
    .expect_stdout_empty()
    .expect_stderr_contains("wrote LICENSE");

    let written = fs::read_to_string(dir.path().join("LICENSE")).unwrap();
    assert!(written.contains("Copyright (c) 2025 Alice"));

    lic_in(
        dir.path(),
        &[
            "--author",
            "Bob",
            "--year",
            "2026",
            "--output",
            "LICENSE.txt",
            "isc",
        ],
    )
    .expect_code(0);

    let written = fs::read_to_string(dir.path().join("LICENSE.txt")).unwrap();
    assert!(written.contains("Copyright (c) 2026 Bob"));
}

#[test]
fn existing_file_needs_force_in_non_interactive_mode() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("LICENSE");
    fs::write(&target, "keep me").unwrap();

    // 非交互（--yes）且没有 --force：退出码 2，文件不被改动
    lic_in(
        dir.path(),
        &["--yes", "--author", "Alice", "mit", "-o", "LICENSE"],
    )
    .expect_code(2)
    .expect_stderr_contains("file already exists")
    .expect_stderr_contains("hint:");
    assert_eq!(fs::read_to_string(&target).unwrap(), "keep me");

    // 带上 --force 才覆盖
    lic_in(
        dir.path(),
        &[
            "--yes", "--force", "--author", "Alice", "mit", "-o", "LICENSE",
        ],
    )
    .expect_code(0);
    assert!(fs::read_to_string(&target).unwrap().contains("Alice"));
}

#[test]
fn existing_file_without_force_exits_with_two() {
    // 测试进程的 stdin 不是终端，所以不会进入交互提问分支，
    // 这条用例锁定的是「非交互下已存在文件 -> 退出码 2」。
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("LICENSE"), "keep me").unwrap();

    lic_in(dir.path(), &["mit", "-o", "LICENSE"])
        .expect_code(2)
        .expect_stderr_contains("file already exists");
}

#[test]
fn output_to_a_directory_fails_cleanly() {
    let dir = tempfile::tempdir().unwrap();
    let nested = dir.path().join("sub");
    fs::create_dir(&nested).unwrap();

    lic(&["--author", "A", "mit", "-o", arg(&nested)])
        .expect_code(2)
        .expect_stderr_contains("is a directory");
}

#[test]
fn missing_parent_directory_reports_the_path() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("nope").join("LICENSE");

    lic(&["--author", "A", "mit", "-o", arg(&missing)])
        .expect_code(2)
        .expect_stderr_contains("failed to write");
}

// ---------------------------------------------------------------------------
// 智能默认值
// ---------------------------------------------------------------------------

#[test]
fn author_year_project_flags_win_over_defaults() {
    lic(&[
        "--author",
        "CI Bot",
        "--year",
        "2030",
        "--project",
        "pipeline",
        "mulanpsl-2.0",
    ])
    .expect_code(0)
    .expect_stdout_contains("Copyright (c) 2030 CI Bot")
    .expect_stdout_contains("pipeline is licensed under Mulan PSL v2.");
}

#[test]
fn short_flags_are_supported() {
    lic(&["-n", "Alice", "-y", "2025", "-p", "myapp", "bsd-4-clause"])
        .expect_code(0)
        .expect_stdout_contains("Copyright (c) 2025, Alice");
}

#[test]
fn year_defaults_to_current_year() {
    let now = time::OffsetDateTime::now_local().unwrap_or_else(|_| time::OffsetDateTime::now_utc());
    let expected = now.year().to_string();

    lic(&["--author", "Alice", "mit"])
        .expect_code(0)
        .expect_stdout_contains(&format!("Copyright (c) {expected} Alice"));
}

#[test]
fn project_defaults_to_current_directory_name() {
    let dir = tempfile::tempdir().unwrap();
    let project = dir.path().join("myapp");
    fs::create_dir(&project).unwrap();

    lic_in(
        &project,
        &["--author", "Alice", "--year", "2025", "mulanpsl-2.0"],
    )
    .expect_code(0)
    .expect_stdout_contains("myapp is licensed under Mulan PSL v2.");
}

#[test]
fn author_can_come_from_environment() {
    run(
        &["--year", "2025", "mit"],
        None,
        &[("LICENSE_FULL_NAME", "Env Person")],
    )
    .expect_code(0)
    .expect_stdout_contains("Copyright (c) 2025 Env Person");

    run(
        &["--year", "2025", "mit"],
        None,
        &[("LICENSE_AUTHOR", "Alt Person")],
    )
    .expect_code(0)
    .expect_stdout_contains("Copyright (c) 2025 Alt Person");
}

#[test]
fn explicit_flag_beats_environment() {
    run(
        &["--author", "From Flag", "--year", "2025", "mit"],
        None,
        &[("LICENSE_FULL_NAME", "From Env")],
    )
    .expect_code(0)
    .expect_stdout_contains("Copyright (c) 2025 From Flag");
}

// ---------------------------------------------------------------------------
// list / info / update
// ---------------------------------------------------------------------------

#[test]
fn list_has_three_equivalent_spellings() {
    for args in [["list"].as_slice(), &["ls"], &["--list"]] {
        lic(args)
            .expect_code(0)
            .expect_stdout_contains("built-in licenses")
            .expect_stdout_contains("mulanpsl-2.0")
            .expect_stdout_contains("MIT License");
    }
}

#[test]
fn info_has_two_equivalent_spellings() {
    for args in [["info", "mit"].as_slice(), &["--info", "mit"]] {
        lic(args)
            .expect_code(0)
            .expect_stdout_contains("MIT License")
            .expect_stdout_contains("SPDX ID")
            .expect_stdout_contains("include-copyright");
    }
}

#[test]
fn update_explains_there_is_nothing_to_do() {
    for args in [["update"].as_slice(), &["--update"]] {
        lic(args)
            .expect_code(0)
            .expect_stdout_contains("no cache to refresh");
    }
}

#[test]
fn list_output_can_be_redirected_to_a_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("licenses.txt");

    lic(&["list", "-o", arg(&target)]).expect_code(0);
    assert!(fs::read_to_string(&target)
        .unwrap()
        .contains("mulanpsl-2.0"));
}

// ---------------------------------------------------------------------------
// 名称解析与模糊匹配
// ---------------------------------------------------------------------------

#[test]
fn spdx_identifiers_and_aliases_are_accepted() {
    lic(&["--author", "A", "--year", "2025", "MIT"])
        .expect_code(0)
        .expect_stdout_contains("MIT License");

    lic(&["--author", "A", "--year", "2025", "apache"])
        .expect_code(0)
        .expect_stdout_contains("Apache License");

    lic(&["--author", "A", "--year", "2025", "MulanPSL-2.0"])
        .expect_code(0)
        .expect_stdout_contains("木兰宽松许可证");

    lic(&["--author", "A", "--year", "2025", "mulan"])
        .expect_code(0)
        .expect_stdout_contains("木兰宽松许可证");
}

#[test]
fn unknown_license_suggests_similar_names() {
    lic(&["apachee"])
        .expect_code(2)
        .expect_stderr_contains("unknown license")
        .expect_stderr_contains("did you mean")
        .expect_stderr_contains("apache-2.0");

    lic(&["totally-bogus-name"])
        .expect_code(2)
        .expect_stderr_contains("run `lic list`");
}

// ---------------------------------------------------------------------------
// 输出格式
// ---------------------------------------------------------------------------

#[test]
fn json_format_exposes_the_documented_fields() {
    let run = lic(&[
        "--author",
        "Alice",
        "--year",
        "2025",
        "--project",
        "myapp",
        "--format",
        "json",
        "mit",
    ]);
    run.expect_code(0);

    let value = run.json();
    assert_eq!(value["spdx_id"], "MIT");
    assert_eq!(value["name"], "MIT License");
    assert_eq!(value["author"], "Alice");
    assert_eq!(value["year"], "2025");
    assert_eq!(value["project"], "myapp");
    assert_eq!(value["category"], "permissive");
    assert!(value["text"].as_str().unwrap().contains("Alice"));
    for field in ["permissions", "conditions", "limitations"] {
        assert!(value[field].is_array(), "{field} should be an array");
    }
    assert!(value["conditions"]
        .as_array()
        .unwrap()
        .contains(&serde_json::Value::from("include-copyright")));
}

#[test]
fn markdown_format_has_a_header() {
    lic(&[
        "--author", "Alice", "--year", "2025", "mit", "--format", "markdown",
    ])
    .expect_code(0)
    .expect_stdout_contains("# MIT License")
    .expect_stdout_contains("- **SPDX ID**: `MIT`")
    .expect_stdout_contains("Permission is hereby granted");
}

#[test]
fn text_format_is_the_default() {
    let default = lic(&["--author", "A", "--year", "2025", "mit"]);
    let explicit = lic(&["--author", "A", "--year", "2025", "--format", "text", "mit"]);
    assert_eq!(default.stdout, explicit.stdout);
}

#[test]
fn global_flags_work_on_subcommands() {
    let run = lic(&["info", "mit", "--format", "json"]);
    run.expect_code(0);
    assert_eq!(run.json()["spdx_id"], "MIT");

    let run = lic(&["list", "--format", "json"]);
    run.expect_code(0);
    assert_eq!(run.json().as_array().unwrap().len(), KEYS.len());
}

// ---------------------------------------------------------------------------
// 帮助、版本与向前兼容标志
// ---------------------------------------------------------------------------

#[test]
fn no_arguments_prints_quick_start() {
    lic(&[])
        .expect_code(0)
        .expect_stdout_contains("Usage:")
        .expect_stdout_contains("Quick start:");
}

#[test]
fn help_lists_every_subcommand_and_flag() {
    let run = lic(&["--help"]);
    run.expect_code(0);
    for needle in [
        "list",
        "info",
        "update",
        "--author",
        "--year",
        "--project",
        "--output",
        "--format",
        "--auth",
        "--update",
        "--offline",
        "--cache-ttl",
        "--yes",
        "--force",
        "--list",
        "--info",
    ] {
        run.expect_stdout_contains(needle);
    }
}

#[test]
fn version_is_reported() {
    lic(&["--version"])
        .expect_code(0)
        .expect_stdout_contains("0.1.0");
}

#[test]
fn offline_flag_is_accepted_and_changes_nothing() {
    let with = lic(&["--author", "Alice", "--year", "2025", "--offline", "mit"]);
    let without = lic(&["--author", "Alice", "--year", "2025", "mit"]);
    with.expect_code(0);
    assert_eq!(with.stdout, without.stdout);
}

#[test]
fn auth_flag_is_accepted() {
    lic(&[
        "--auth",
        "alice:ghp_secret",
        "--author",
        "A",
        "--year",
        "2025",
        "mit",
    ])
    .expect_code(0);

    lic(&[
        "--auth",
        "ghp_secret",
        "--author",
        "A",
        "--year",
        "2025",
        "mit",
    ])
    .expect_code(0);
}

#[test]
fn cache_ttl_flag_is_accepted() {
    lic(&["--cache-ttl", "7", "--author", "A", "--year", "2025", "mit"]).expect_code(0);
}

/// 保证 `PathBuf` 辅助函数被用到（避免 dead_code 警告）。
#[test]
fn binary_path_helper_works() {
    let path: PathBuf = PathBuf::from(env!("CARGO_BIN_EXE_lic"));
    assert!(path.exists(), "{} should exist", path.display());
}
