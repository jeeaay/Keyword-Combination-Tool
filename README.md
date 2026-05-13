# Keyword Composer

一个使用 Rust + `eframe/egui` 构建的桌面关键词组合工具。  
支持输入前缀、核心词、后缀，批量生成组合结果，并提供前后缀本地记忆、组合空格偏好持久化与一键复制。

## 功能特性

- 前缀、核心词、后缀多行输入（按行解析）
- 三列大输入区布局，优先保证输入区域可见性与可操作性
- 自动去除空白、忽略空行、去重
- 允许前缀/后缀为空，但至少要有一类非空输入才可生成
- 支持“添加空格”选项，仅在相邻非空片段之间插入空格
- 结果区支持鼠标选择复制
- 结果区默认显示 6 行高度，超出部分在框内滚动
- 支持“复制全部”写入系统剪贴板
- 自动记忆前缀和后缀，启动时恢复，支持一键回填
- 当前缀或后缀输入框已空时，再次点击“清空”可删除对应记忆
- “添加空格”复选框状态会写入本地记忆，启动时自动恢复
- Windows 发布版可隐藏控制台黑框，并支持 `.exe` 图标与窗口图标

## 环境要求

- Windows（PowerShell）
- Rust（建议 stable）
- MSVC 工具链（需可用 `link.exe`、`cl.exe`）

你可以用以下命令验证工具链：

```powershell
where.exe link
where.exe cl
rustc -V
cargo -V
```

## 快速开始

### 1) 安装依赖

```powershell
cargo fetch
```

### 2) 构建项目

如果你的终端已经能直接找到 `link.exe`，可直接：

```powershell
cargo build
cargo build --release
```

如果终端无法直接找到 MSVC，可使用仓库内脚本（临时注入路径，不改系统全局变量）：

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 build
# 编译发行版本
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 build --release
```

`msvc-cargo.ps1` 支持将后续参数原样转发给 `cargo`，例如 `build --release`、`run`、`test`。

### 3) 运行应用

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 run
```

### 4) 运行测试

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 test
```

## 使用说明

1. 在顶部三列输入区分别填写前缀、核心词、后缀（每行一个条目）。
2. 如需自然分词效果，可勾选操作区中的“添加空格”。
3. 点击“生成结果”。
4. 在下方结果区查看组合，可鼠标拖拽选择局部内容复制。
5. 点击“复制全部”将全部结果写入剪贴板。
6. 前缀/后缀会自动记忆，下次启动可直接回填复用。

## 交互细节

- 前缀、核心词、后缀均按多行解析，每行一个条目
- 空白行会被忽略，重复条目会自动去重
- 前缀和后缀可以为空，但三类输入至少要有一类非空
- 勾选“添加空格”后，仅在相邻非空片段之间插入空格
- 例如：
  - 前缀 `免费`，核心词 `AI工具`，后缀 `下载` -> `免费 AI工具 下载`
  - 前缀为空，核心词 `AI工具`，后缀 `下载` -> `AI工具 下载`
- 前缀和后缀卡片支持“回填全部”
- 如果前缀或后缀输入框已有内容，点击“清空”只会清空输入框
- 如果前缀或后缀输入框已经为空，并且存在对应记忆，再次点击“清空”会删除对应记忆
- 核心词输入框的“清空”只清空输入内容，不影响本地记忆
- 结果区默认固定为约 6 行高度，超出结果会在内部滚动区域显示

## 本地数据

本地记忆默认保存为 `keyword-memory.json`，优先路径：

- `%LOCALAPPDATA%\keyword\keyword-memory.json`
- `%APPDATA%\keyword\keyword-memory.json`
- 若以上环境变量不可用，则回退到当前工作目录

当前文件同时保存以下内容：

- 已记忆前缀 `prefixes`
- 已记忆后缀 `suffixes`
- “添加空格”偏好 `insert_spaces`

示例结构：

```json
{
  "prefixes": ["免费", "热门"],
  "suffixes": ["下载", "推荐"],
  "insert_spaces": true
}
```

## 项目结构

```text
.
├─ src/
│  ├─ main.rs      # 应用入口与窗口配置
│  ├─ app.rs       # 主要状态、UI、组合逻辑、持久化与测试
│  └─ theme.rs     # 主题与视觉样式
├─ assets/         # PNG / ICO 图标资源
├─ build.rs        # Windows 资源编译脚本（写入 exe 图标）
├─ msvc-cargo.ps1  # 临时注入 MSVC 路径并转发 cargo 命令
├─ Cargo.toml
└─ README.md
```

## 图标与窗口

- `build.rs` 会在 Windows 构建时将 `assets/keyword.ico` 写入生成的 `.exe`
- [main.rs](file:///h:/git/rust/keyword/src/main.rs) 会将 `assets/keyword.png` 作为运行时窗口图标嵌入程序
- Release 模式下已启用 `windows_subsystem = "windows"`，启动正式版时不会弹出黑色控制台窗口

## 发布说明

- Debug 可执行文件默认位于 `target\debug\keyword.exe`
- Release 可执行文件默认位于 `target\release\keyword.exe`
- 发布给普通用户时，通常只需要分发 `keyword.exe`
- `target\release` 下的 `deps`、`build`、`incremental`、`.pdb` 等内容主要用于构建或调试，通常不需要随程序一起打包

## 常见问题

### `link.exe` 找不到

- 先确认已安装 Visual Studio C++ 构建工具
- 在 PowerShell 中检查：

```powershell
where.exe link
```

- 如果当前终端找不到，优先使用：

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 build
```

### 增量编译 hard link warning

类似以下警告一般不影响构建正确性：

`hard linking files in the incremental compilation cache failed...`

这是文件系统对硬链接支持差异导致，Rust 会自动回退为复制。

### Release 版启动出现黑框

- 当前项目已在 Release 模式下关闭控制台子系统
- 请重新执行：

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 build --release
```

- 然后运行 `target\release\keyword.exe`

### 发布时需要打包整个 `target\release` 吗

- 一般不需要
- 对最终用户分发时，通常只需要 `target\release\keyword.exe`
- 本地记忆文件会在用户首次运行后自动创建到 `AppData` 对应目录
