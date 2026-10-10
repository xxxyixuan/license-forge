`lic` 是一个用来生成 LICENSE 文件的命令行工具。

它为作者名、年份和项目名提供了好用的默认值（需要时可以覆盖），
并把 17 个许可证模板编译进二进制——因此不需要联网，也不需要缓存。
许可证正文取自
[GitHub Licenses API](https://docs.github.com/en/rest/licenses) 与
[SPDX License List](https://spdx.org/licenses/)。

```
agpl-3.0      (GNU Affero General Public License v3.0)
apache-2.0    (Apache License 2.0)
artistic-2.0  (Artistic License 2.0)
bsd-2-clause  (BSD 2-Clause "Simplified" License)
bsd-3-clause  (BSD 3-Clause "New" or "Revised" License)
bsd-4-clause  (BSD 4-Clause "Original" or "Old" License)
bsl-1.0       (Boost Software License 1.0)
cc0-1.0       (Creative Commons Zero v1.0 Universal)
gpl-3.0       (GNU General Public License v3.0)
isc           (ISC License)
lgpl-3.0      (GNU Lesser General Public License v3.0)
mit           (MIT License)
mpl-2.0       (Mozilla Public License 2.0)
mulanpsl-2.0  (木兰宽松许可证 第2版)
unlicense     (The Unlicense)
wtfpl         (Do What The Fuck You Want To Public License)
zlib          (zlib License)
```

SPDX 标识符和常用别名也能用：`lic MIT`、`lic apache`、`lic gpl`、`lic mulan`。

## 安装

### 从源码构建

需要 Rust 1.75 或更高版本：

```sh
cargo install --git https://github.com/xxxyixuan/license-forge
```

或者在本地源码目录里安装：

```sh
cargo install --path .
```

crate 名是 `license-forge`，二进制名是 `lic`。

## 用法

#### 打印许可证

把许可证打印到 stdout，只要在 `lic` 后面跟上许可证名：

```sh
$ lic mit
```

#### 写入文件

用 `-o` 标志写入文件，或者用 shell 的重定向：

```sh
$ lic mit -o LICENSE
$ lic mit > LICENSE
```

#### 自定义作者、年份和项目名

```sh
$ lic --author "Alice L" --year 2025 isc
$ lic -n "Alice L" -y 2025 -p "myapp" bsd-4-clause
```

省略 `--year` 时使用当前年份。

省略 `--project` 时使用当前目录名作为项目名。

作者名按以下顺序确定：

```
- 命令行标志：--author、-n
- 环境变量：LICENSE_FULL_NAME、LICENSE_AUTHOR
- gitconfig 与 hgconfig
- 系统用户名
- 空字符串
```

如果你的名字已经写在 `$HOME/.gitconfig` 里，几乎总是可以省略 `--author`。

#### 列出全部许可证

```sh
$ lic list
$ lic ls
$ lic --list
```

#### 查看许可证详情

```sh
$ lic info mit
$ lic --info mit
```

#### 输出格式

```sh
$ lic --format json mit
$ lic --format markdown mit
$ lic --format text mit
```

`text` 是默认值。JSON 输出包含 SPDX 标识符、渲染后的正文、
解析出的作者/年份/项目名，以及该许可证的 permissions、conditions、limitations。

#### 非交互用法

```sh
$ lic --author "CI Bot" --year 2025 --project "myapp" --yes -o ./LICENSE mit
```

#### 选项

```
-n, --author <AUTHOR>     版权持有人
-y, --year <YEAR>         年份
-p, --project <PROJECT>   项目名
-o, --output <FILE>       写入 FILE，而不是 stdout
    --format <FORMAT>     text（默认）、markdown、json
    --yes                 不提问
    --force               覆盖已存在的输出文件
    --offline             只用内置模板与缓存
    --auth <AUTH>         GitHub API 认证（token 或 username:token）；可解析，尚未生效
    --cache-ttl <DAYS>    模板缓存过期天数；可解析，尚未生效
    --list                等价于 `lic list`
    --info <LICENSE>      等价于 `lic info <LICENSE>`
    --update              等价于 `lic update`
-h, --help                打印帮助
-V, --version             打印版本
```

输出文件已存在时，如果 stdin 是终端，`lic` 会先询问是否覆盖；
非交互场景下不加 `--force` 会直接拒绝。

退出码：`0` 成功、`1` 用户取消、`2` 出错。
许可证名拼错时会给出模糊匹配建议：

```sh
$ lic apachee
error: unknown license `apachee`
hint: did you mean: apache-2.0? (run `lic list` for every built-in license)
```

## 贡献

欢迎提交新功能、修 bug 的 PR 和建议！改动较大时请先开 issue 讨论。

## 许可证

[MIT](LICENSE)
