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

Prompt:
B

改动:
- CI 改为 nightly + build-std 以支持 `thumbv7a-none-eabi`

Prompt:
1

改动:
- CI 固定 nightly 版本（`nightly-2026-01-22`）

Prompt:
1

改动:
- 新增 `rust-toolchain.toml` 并在 README 说明

Prompt:
把 CI 加速优化一下

改动:
- CI 增加 cargo/rustup 缓存

Prompt:
又出现类似的情况，每次提交到GitHub上后，显示Workflow的运行都是失败的，请分析原因

改动:
- 诊断 CI 失败原因：`cargo +nightly` 未使用固定工具链导致缺少 rust-src

Prompt:
使用方案2

改动:
- CI 改为使用 `rust-toolchain.toml` 的默认工具链

Prompt:
最新一轮CI结果没有变绿，请分析原因

改动:
- 诊断 CI 失败原因：`thumbv7a-none-eabi` 不是内置 Rust 目标

Prompt:
B

改动:
- 新增自定义目标 `targets/armv7a-none-eabi.json`
- CI 改用自定义 target JSON
- hidumper Makefile 默认 Rust 目标指向自定义 target JSON
- README 更新默认 Rust target 说明

Prompt:
还是没有解决这个问题，请继续分析原因

改动:
- 修复自定义 target JSON 类型错误（`target-pointer-width` 与 `target-c-int-width` 需为数字）

Prompt:
soft

改动:
- 自定义 target JSON 添加 `llvm-floatabi: soft`

Prompt:
CI还是没有变绿，请继续分析原因

改动:
- 修复自定义 target JSON 的 data-layout，补齐 `Fi8` 以匹配 LLVM 默认布局

Prompt:
我想继续迁移工作，请看看目前又哪些候选工作

改动:
- 规划下一批迁移候选（hidumper CPU usage 优先）

Prompt:
1

改动:
- 迁移 hidumper 的 CPU usage 输出逻辑到 Rust
- 新增 hidumper C 侧 FFI 辅助函数（进程信息/CPU usage/内存分配/格式化输出）

Prompt:
1

改动:
- 新增 `kernel/rust` 基础设施（log/types）
- hidumper Rust 引用 `kernel_rust` 基础 crate
- CI 增加 `kernel/rust` 的 build/clippy

Prompt:
1

改动:
- `kernel_rust` 日志改用通用 `KernelPrintk`
- 新增 `kernel/common/los_rust.c` 提供通用打印接口
- hidumper Rust 使用 `kernel_rust::log::printk_cstr` 打印 C 字符串

Prompt:
1

改动:
- `kernel_rust` 增加通用内存分配封装
- `kernel/common/los_rust.c` 增加 `KernelMemAlloc/KernelMemFree`

Prompt:
CI出现问题，请分析原因

改动:
- 修复 `kernel/rust` 子模块中的 `#![no_std]` 误用（仅 crate root 允许）

Prompt:
A

改动:
- 为 `kernel_rust` 的 unsafe API 补充 `# Safety` 文档以通过 clippy

Prompt:
1

改动:
- `kernel_rust` 增加 Errno/Result 类型与基础转换函数

Prompt:
1

改动:
- 新增 FFI 迁移模板文档 `docs/ffi_template.md`
- README 补充模板文档链接

Prompt:
1

改动:
- 迁移 trace 控制指令解析到 Rust（command validation/handle）
- trace Makefile 支持 `TRACE_USE_RUST`
- CI 增加 trace Rust build/clippy
