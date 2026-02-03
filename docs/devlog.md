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
