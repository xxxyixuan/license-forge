#!/usr/bin/env python3
"""重新生成 src/templates/ 下的许可证模板。

数据源（权威来源，非人工转录）：
  - GitHub Licenses API  https://api.github.com/licenses/{key}
      提供正文字段 `body`，以及 permissions / conditions / limitations 元数据。
  - SPDX License List    https://spdx.org/licenses/{id}.json
      提供 `licenseText`，用于收录 GitHub 未收录的许可证（如木兰 MulanPSL-2.0）。

脚本做两件事：
  1. 把各来源的占位符写法统一成 {{author}} / {{year}} / {{project}}；
  2. 写出 src/templates/<key>.txt，并生成 scripts/templates.lock.json 记录
     每个模板的来源 URL、上游标识符与 sha256，便于审计和 diff。

用法：
    python scripts/fetch_templates.py            # 已有缓存则直接复用
    python scripts/fetch_templates.py --refresh  # 忽略缓存，重新联网抓取

注意：模板正文是「原样收录」的，脚本不会改写许可证条款文字。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import time
import urllib.request

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
RAW_DIR = os.path.join(HERE, ".cache")
OUT_DIR = os.path.join(ROOT, "src", "templates")
LOCK_PATH = os.path.join(HERE, "templates.lock.json")

UA = "license-forge-template-fetcher/0.1.0"

# 内置许可证对应的 GitHub Licenses API key。
GITHUB_KEYS = [
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
]

# GitHub Licenses API 未收录、改从 SPDX License List 获取的许可证。
SPDX_IDS = [
    "MulanPSL-2.0",
]

# 精确 token 替换表。必须是完整 token，避免误伤 URL 里的尖括号（如 <https://fsf.org/>）。
NORMALIZE = [
    ("[year]", "{{year}}"),
    ("[yyyy]", "{{year}}"),
    ("[Year]", "{{year}}"),
    ("[fullname]", "{{author}}"),
    ("[name of copyright owner]", "{{author}}"),
    ("[name of copyright holder]", "{{author}}"),
    ("[Software Name]", "{{project}}"),
    ("<year>", "{{year}}"),
    ("<name of author>", "{{author}}"),
    ("<program>", "{{project}}"),
]


def http_get_json(url: str, retries: int = 4):
    """带重试的 GET，返回解析后的 JSON。"""
    last: Exception | None = None
    for attempt in range(retries):
        request = urllib.request.Request(
            url,
            headers={"User-Agent": UA, "Accept": "application/vnd.github+json"},
        )
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                return json.loads(response.read().decode("utf-8"))
        except Exception as exc:  # noqa: BLE001 - 网络问题统一重试
            last = exc
            time.sleep(2 * (attempt + 1))
    raise RuntimeError(f"failed to fetch {url}: {last}")


def fetch_cached(kind: str, name: str, url: str, refresh: bool):
    """抓取并缓存到 scripts/.cache/，避免重复请求触发速率限制。"""
    os.makedirs(RAW_DIR, exist_ok=True)
    path = os.path.join(RAW_DIR, f"{kind}-{name}.json")
    if os.path.exists(path) and not refresh:
        with open(path, encoding="utf-8") as handle:
            return json.load(handle), False

    data = http_get_json(url)
    with open(path, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(data, handle, ensure_ascii=False, indent=2)
    time.sleep(1)  # 对匿名 GitHub API 友好一点
    return data, True


def normalize(text: str) -> str:
    """统一占位符、换行符，并保证文件以单个换行结尾。"""
    for old, new in NORMALIZE:
        text = text.replace(old, new)
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    return text.rstrip("\n") + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--refresh",
        action="store_true",
        help="忽略 scripts/.cache/，重新联网抓取",
    )
    args = parser.parse_args()

    os.makedirs(OUT_DIR, exist_ok=True)
    lock: dict[str, dict] = {}
    written = 0

    jobs = [
        (key, "github", f"https://api.github.com/licenses/{key}")
        for key in GITHUB_KEYS
    ] + [
        (spdx_id, "spdx", f"https://spdx.org/licenses/{spdx_id}.json")
        for spdx_id in SPDX_IDS
    ]

    for name, kind, url in jobs:
        data, fetched = fetch_cached(kind, name, url, args.refresh)

        if kind == "github":
            key = name
            text = data.get("body") or ""
            spdx_id = data.get("spdx_id") or name
        else:
            key = name.lower()
            text = data.get("licenseText") or ""
            spdx_id = data.get("licenseId") or name

        if not text.strip():
            print(f"!! {name}: upstream returned an empty text", file=sys.stderr)
            return 1

        normalized = normalize(text)
        out_path = os.path.join(OUT_DIR, f"{key}.txt")
        with open(out_path, "w", encoding="utf-8", newline="\n") as handle:
            handle.write(normalized)

        lock[key] = {
            "spdx_id": spdx_id,
            "source": url,
            "sha256": hashlib.sha256(normalized.encode("utf-8")).hexdigest(),
            "bytes": len(normalized.encode("utf-8")),
            "placeholders": sorted(
                {token for token, _ in NORMALIZE if token in text}
            ),
        }

        flag = "fetched" if fetched else "cached "
        print(f"{flag}  {key:<16} {len(normalized.encode('utf-8')):>7} B")
        written += 1

    with open(LOCK_PATH, "w", encoding="utf-8", newline="\n") as handle:
        json.dump(lock, handle, ensure_ascii=False, indent=2, sort_keys=True)
        handle.write("\n")

    print(f"\n{written} templates -> {OUT_DIR}")
    print(f"lock file        -> {LOCK_PATH}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
