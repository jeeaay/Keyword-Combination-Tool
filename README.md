# Keyword Composer

一个使用 Rust + `eframe/egui` 构建的桌面关键词组合工具。  
支持输入前缀、核心词、后缀，批量生成组合结果，并提供前后缀本地记忆与一键复制。

## 功能特性

- 前缀、核心词、后缀多行输入（按行解析）
- 自动去除空白、忽略空行、去重
- 允许前缀/后缀为空，但至少要有一类非空输入才可生成
- 结果区支持鼠标选择复制
- 支持“复制全部”写入系统剪贴板
- 自动记忆前缀和后缀，启动时恢复，支持一键回填

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
```

如果终端无法直接找到 MSVC，可使用仓库内脚本（临时注入路径，不改系统全局变量）：

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 build
```

### 3) 运行应用

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 run
```

### 4) 运行测试

```powershell
powershell -ExecutionPolicy Bypass -File .\msvc-cargo.ps1 test
```

## 使用说明

1. 在左侧输入区分别填写前缀、核心词、后缀（每行一个条目）。
2. 点击“生成结果”。
3. 在右侧结果区查看组合，可鼠标拖拽选择局部内容复制。
4. 点击“复制全部”将全部结果写入剪贴板。
5. 前缀/后缀会自动记忆，下次启动可直接回填复用。

## 本地数据

前后缀记忆默认保存为 `keyword-memory.json`，优先路径：

- `%LOCALAPPDATA%\keyword\keyword-memory.json`
- `%APPDATA%\keyword\keyword-memory.json`
- 若以上环境变量不可用，则回退到当前工作目录

## 项目结构

```text
.
├─ src/
│  ├─ main.rs      # 应用入口与窗口配置
│  ├─ app.rs       # 主要状态、UI、组合逻辑、持久化与测试
│  └─ theme.rs     # 主题与视觉样式
├─ msvc-cargo.ps1  # 临时注入 MSVC 路径并转发 cargo 命令
├─ Cargo.toml
└─ README.md
```

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
