# Dev Log

## 2026-02-03

Prompt:
我希望将所有的开发过程都记录在一个文档中，包括我的Prompt输入

改动:
- 新增 Rust 迁移脚手架与适配层：`kernel/extended/hidumper/rust/lib.rs`、`kernel/extended/hidumper/hidumper_rust_ffi.c`
- 在 `hidumper` 模块中接入 Rust（Make）：`kernel/extended/hidumper/Makefile`
- 在 `hidumper` 适配器中切换部分接口到 Rust：`kernel/extended/hidumper/los_hidumper.c`
- 支持模块 Makefile 注入额外对象：`tools/build/mk/module.mk`、`config.mk`

Prompt:
请将相关的修改进行提交

改动:
- 提交 hidumper Rust 接入与构建支持相关改动

Prompt:
请在我的github账号下新建一个项目，并将本地项目提交到远程github上

改动:
- 创建 GitHub 仓库 `dong-qiu/kernel_liteos_a_rust`（public）
- 新增远程 `github` 并推送当前分支

Prompt:
1, 2, 3

改动:
- 切换默认分支为 `main`
- 新增 GitHub Actions CI（Rust hidumper smoke 编译）
- README 添加 CI 徽章与 Rust 迁移说明

Prompt:
1

改动:
- 删除远端 `master` 分支

Prompt:
1

改动:
- 删除本地 `master` 分支

Prompt:
在 README 补充迁移路线图（简短但清晰）

改动:
- README 添加 Rust 迁移路线图（简短版）

Prompt:
续深化 README（比如“当前完成进度 / 里程碑 / 如何贡献”）

改动:
- README 添加当前进度、里程碑、贡献方式小节

Prompt:
完善 README（比如“构建步骤 / Rust 编译开关 / CI 说明 / 迁移规范模板”）

改动:
- README 添加构建步骤、Rust 编译开关、CI 说明、迁移模板

Prompt:
先在 CI 中增加基础格式/静态检查步骤

改动:
- CI 增加 rustfmt 格式检查步骤

Prompt:
1

改动:
- CI 在 rustc 编译中开启 `-D warnings`

Prompt:
1

改动:
- 添加 hidumper Rust 的 `Cargo.toml`
- CI 增加 `cargo clippy` 检查
