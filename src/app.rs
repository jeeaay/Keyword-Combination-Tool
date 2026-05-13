use std::{env, fs, path::PathBuf};

use arboard::Clipboard;
use eframe::egui::{
    self, Align, Button, CentralPanel, Frame, Layout, Margin, RichText, ScrollArea, Stroke,
    TextEdit, Ui, Vec2,
};
use serde::{Deserialize, Serialize};

use crate::theme;

/// `KeywordApp` 承载整个 GUI 应用的界面状态和布局渲染入口。
pub struct KeywordApp {
    pub input: InputState,
    pub actions: ActionPanelState,
    pub results: ResultPanelState,
    memory: RememberedEntriesState,
}

/// `InputState` 预留前缀、核心词、后缀三块输入区域的状态。
pub struct InputState {
    pub prefix: EditorPaneState,
    pub keyword: EditorPaneState,
    pub suffix: EditorPaneState,
}

/// `EditorPaneState` 描述单个多行编辑区的显示文案与内容缓存。
pub struct EditorPaneState {
    pub title: &'static str,
    pub hint: &'static str,
    pub placeholder: &'static str,
    pub buffer: String,
}

/// `ActionPanelState` 预留生成、复制和校验反馈等操作区状态。
pub struct ActionPanelState {
    pub generate_enabled: bool,
    pub copy_enabled: bool,
    pub insert_spaces: bool,
    pub primary_label: &'static str,
    pub secondary_label: &'static str,
    pub status_message: String,
    pub generate_hint: String,
    pub copy_hint: String,
}

/// `ResultPanelState` 预留结果文本、复制反馈和统计信息。
pub struct ResultPanelState {
    pub preview_text: String,
    pub total_count: usize,
    pub feedback: String,
    pub feedback_tone: FeedbackTone,
}

/// `FeedbackTone` 描述结果区反馈文案的语义强度，便于统一成功、失败和提示色。
pub enum FeedbackTone {
    Info,
    Success,
    Error,
}

/// `RememberedEntriesState` 保存已恢复的前后缀集合及本地存储状态说明。
struct RememberedEntriesState {
    prefixes: Vec<String>,
    suffixes: Vec<String>,
    insert_spaces: bool,
    storage_path: Option<PathBuf>,
    storage_status: String,
}

/// `EditorInteraction` 描述输入卡片本帧触发的主要交互，便于父层决定是否更新记忆。
enum EditorInteraction {
    None,
    ReusedMemory,
    RemoveMemory,
}

/// `MemoryBucket` 区分前缀与后缀两类本地记忆，用于统一删除与反馈逻辑。
enum MemoryBucket {
    Prefix,
    Suffix,
}

/// `PersistedMemory` 描述写入本地 JSON 文件的前后缀持久化模型。
#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedMemory {
    prefixes: Vec<String>,
    suffixes: Vec<String>,
    #[serde(default)]
    insert_spaces: bool,
}

impl KeywordApp {
    /// 创建应用初始状态，注入统一主题并在启动时恢复已保存的前后缀记忆。
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_theme(&cc.egui_ctx);
        let memory = Self::load_remembered_entries();
        let mut actions = ActionPanelState::new();
        actions.insert_spaces = memory.insert_spaces;

        Self {
            input: InputState::new(),
            actions,
            results: ResultPanelState::new(),
            memory,
        }
    }

    /// 渲染主工作区：上方三列输入，下方操作与结果，突出输入作为首要交互区域。
    fn render_workspace(&mut self, ctx: &egui::Context) {
        let prefix_memory = self.memory.prefixes.clone();
        let suffix_memory = self.memory.suffixes.clone();

        CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(theme::PANEL)
                    .inner_margin(Margin::same(18)),
            )
            .show(ctx, |ui| {
                let top_height = (ui.available_height() * 0.68).max(360.0);
                ui.allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), top_height),
                    Layout::top_down(Align::Min),
                    |ui| {
                        ui.columns(3, |columns| {
                            match Self::render_editor_with_memory(
                                &mut columns[0],
                                &mut self.input.prefix,
                                Some(&prefix_memory),
                            ) {
                                EditorInteraction::ReusedMemory => self.results.set_info_feedback(
                                    "已将记忆前缀回填到输入区；重复条目会自动忽略。",
                                ),
                                EditorInteraction::RemoveMemory => {
                                    self.clear_remembered_entries(MemoryBucket::Prefix);
                                }
                                EditorInteraction::None => {}
                            }
                            match Self::render_editor_with_memory(
                                &mut columns[1],
                                &mut self.input.keyword,
                                None,
                            ) {
                                EditorInteraction::ReusedMemory => self.results.set_info_feedback(
                                    "核心词输入区已更新，可直接生成新的组合结果。",
                                ),
                                EditorInteraction::RemoveMemory | EditorInteraction::None => {}
                            }
                            match Self::render_editor_with_memory(
                                &mut columns[2],
                                &mut self.input.suffix,
                                Some(&suffix_memory),
                            ) {
                                EditorInteraction::ReusedMemory => self.results.set_info_feedback(
                                    "已将记忆后缀回填到输入区；重复条目会自动忽略。",
                                ),
                                EditorInteraction::RemoveMemory => {
                                    self.clear_remembered_entries(MemoryBucket::Suffix);
                                }
                                EditorInteraction::None => {}
                            }
                        });
                    },
                );

                ui.add_space(12.0);
                ui.columns(2, |columns| {
                    self.render_action_panel(&mut columns[0]);
                    self.render_results_panel(&mut columns[1]);
                });

                ui.add_space(8.0);
                ui.colored_label(theme::ACCENT_SOFT, &self.memory.storage_status);
            });
    }

    /// 依据当前输入内容刷新操作区按钮状态与提示文案。
    fn refresh_action_state(&mut self) {
        let prefixes = Self::parse_entries(&self.input.prefix.buffer);
        let keywords = Self::parse_entries(&self.input.keyword.buffer);
        let suffixes = Self::parse_entries(&self.input.suffix.buffer);
        let has_any_input = Self::has_any_entries(&prefixes, &keywords, &suffixes);
        let has_copyable_results = self.results.has_copyable_results();
        let input_summary = Self::format_entry_summary(&prefixes, &keywords, &suffixes);

        self.actions.generate_enabled = has_any_input;
        self.actions.copy_enabled = has_copyable_results;
        self.actions.status_message = if has_copyable_results {
            format!("{}。最近一次生成了 {} 条结果。", input_summary, self.results.total_count)
        } else if has_any_input {
            format!("{input_summary}。已检测到有效输入，可以开始生成关键词组合。")
        } else {
            "请输入至少一个非空条目；纯空格和空行不会参与生成。".to_owned()
        };
        self.actions.secondary_label = if has_copyable_results {
            "结果区支持鼠标拖拽选择局部文本；“复制全部” 会将当前结果写入系统剪贴板。"
        } else if has_any_input {
            "生成时会自动按行解析，并将本次使用到的非空前后缀保存到本地记忆。"
        } else {
            "前缀和后缀可留空；应用启动时会恢复已记忆前后缀，并支持一键回填。"
        };
        self.actions.generate_hint = if has_any_input {
            format!("{input_summary}；点击后会生成全部有效组合。")
        } else {
            "生成按钮当前不可用：请先输入至少一个非空前缀、核心词或后缀。".to_owned()
        };
        self.actions.copy_hint = if has_copyable_results {
            format!(
                "复制全部会将当前 {} 条结果按换行文本写入系统剪贴板。",
                self.results.total_count
            )
        } else {
            "复制按钮当前不可用：请先生成至少一条结果。".to_owned()
        };
    }

    /// 处理“生成结果”动作，并在成功后持久化本次使用的非空前后缀。
    fn handle_generate(&mut self) {
        let prefixes = Self::parse_entries(&self.input.prefix.buffer);
        let keywords = Self::parse_entries(&self.input.keyword.buffer);
        let suffixes = Self::parse_entries(&self.input.suffix.buffer);

        if !Self::has_any_entries(&prefixes, &keywords, &suffixes) {
            self.results.total_count = 0;
            self.results.preview_text.clear();
            self.results
                .set_error_feedback("空输入已拦截：系统会自动忽略空白和空行。");
            self.actions.status_message = "未检测到有效输入，暂不执行生成。".to_owned();
            return;
        }

        let combinations =
            Self::build_combinations(&prefixes, &keywords, &suffixes, self.actions.insert_spaces);
        let persistence_feedback = self.persist_generated_memory(&prefixes, &suffixes);

        self.results.total_count = combinations.len();
        self.results.preview_text = combinations.join("\n");
        self.results.set_success_feedback(format!(
            "生成完成：共 {} 条结果，均按“前缀 + 核心词 + 后缀”顺序拼接。{}{}",
            self.results.total_count,
            if self.actions.insert_spaces {
                "已按非空片段自动插入空格。"
            } else {
                ""
            },
            persistence_feedback
        ));
        self.actions.status_message = format!(
            "生成成功：{} 条结果已写入右侧预览区。",
            self.results.total_count
        );
    }

    /// 将当前结果全文写入系统剪贴板，并给出成功或失败的明确反馈。
    fn handle_copy_all(&mut self) {
        if !self.results.has_copyable_results() {
            self.results
                .set_error_feedback("复制失败：当前没有可复制的生成结果。");
            self.actions.status_message = "结果区为空，暂不执行复制。".to_owned();
            return;
        }

        let preview_text = self.results.preview_text.clone();
        match Clipboard::new().and_then(|mut clipboard| clipboard.set_text(preview_text)) {
            Ok(()) => {
                self.results.set_success_feedback(format!(
                    "复制成功：已将 {} 条结果写入系统剪贴板。",
                    self.results.total_count
                ));
                self.actions.status_message = "复制成功：系统剪贴板已更新。".to_owned();
            }
            Err(err) => {
                self.results
                    .set_error_feedback(format!("复制失败：无法写入系统剪贴板，错误：{err}"));
                self.actions.status_message = "复制失败：请检查系统剪贴板是否可用。".to_owned();
            }
        }
    }

    /// 在启动时读取本地 JSON 文件，并恢复已记忆的前后缀条目。
    fn load_remembered_entries() -> RememberedEntriesState {
        let storage_path = match Self::storage_file_path() {
            Ok(path) => path,
            Err(err) => {
                return RememberedEntriesState {
                    prefixes: Vec::new(),
                    suffixes: Vec::new(),
                    insert_spaces: false,
                    storage_path: None,
                    storage_status: format!("本地记忆不可用：{err}"),
                };
            }
        };

        if !storage_path.exists() {
            return RememberedEntriesState {
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                insert_spaces: false,
                storage_path: Some(storage_path.clone()),
                storage_status: format!(
                    "本地记忆文件尚未创建；首次生成或调整偏好后会保存到 {}。",
                    storage_path.display()
                ),
            };
        }

        match fs::read_to_string(&storage_path) {
            Ok(raw) => match serde_json::from_str::<PersistedMemory>(&raw) {
                Ok(memory) => {
                    let prefixes = Self::normalize_entries(memory.prefixes);
                    let suffixes = Self::normalize_entries(memory.suffixes);

                    RememberedEntriesState {
                        storage_status: format!(
                            "已从 {} 恢复 {} 个前缀、{} 个后缀；添加空格为 {}。",
                            storage_path.display(),
                            prefixes.len(),
                            suffixes.len(),
                            if memory.insert_spaces { "开启" } else { "关闭" }
                        ),
                        prefixes,
                        suffixes,
                        insert_spaces: memory.insert_spaces,
                        storage_path: Some(storage_path),
                    }
                }
                Err(err) => RememberedEntriesState {
                    prefixes: Vec::new(),
                    suffixes: Vec::new(),
                    insert_spaces: false,
                    storage_path: Some(storage_path.clone()),
                    storage_status: format!(
                        "读取本地记忆失败：{}，错误：{}。",
                        storage_path.display(),
                        err
                    ),
                },
            },
            Err(err) => RememberedEntriesState {
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                insert_spaces: false,
                storage_path: Some(storage_path.clone()),
                storage_status: format!(
                    "打开本地记忆失败：{}，错误：{}。",
                    storage_path.display(),
                    err
                ),
            },
        }
    }

    /// 在生成成功后合并并保存新的前后缀记忆，同时返回界面反馈文案。
    fn persist_generated_memory(&mut self, prefixes: &[String], suffixes: &[String]) -> String {
        let mut persisted = self.snapshot_persisted_memory();

        let added_prefixes = Self::merge_unique_entries(&mut persisted.prefixes, prefixes);
        let added_suffixes = Self::merge_unique_entries(&mut persisted.suffixes, suffixes);

        match self.store_persisted_memory(persisted) {
            Ok(_) => {
                if prefixes.is_empty() && suffixes.is_empty() {
                    "本次未提供非空前后缀；已同步保存“添加空格”设置。".to_owned()
                } else if added_prefixes == 0 && added_suffixes == 0 {
                    "本次前后缀均已存在于本地记忆中；“添加空格”设置也已同步保存。".to_owned()
                } else {
                    format!(
                        "已保存 {} 个新增前缀和 {} 个新增后缀到本地记忆，并同步保存“添加空格”设置。",
                        added_prefixes, added_suffixes
                    )
                }
            }
            Err(err) => {
                self.memory.storage_status = format!("本地记忆保存失败：{err}");
                format!("本地记忆保存失败：{err}")
            }
        }
    }

    /// 解析本地记忆文件的目标路径，优先使用 Windows 的 `LOCALAPPDATA` 目录。
    fn storage_file_path() -> Result<PathBuf, String> {
        if let Ok(local_app_data) = env::var("LOCALAPPDATA") {
            return Ok(PathBuf::from(local_app_data)
                .join("keyword")
                .join("keyword-memory.json"));
        }

        if let Ok(app_data) = env::var("APPDATA") {
            return Ok(PathBuf::from(app_data)
                .join("keyword")
                .join("keyword-memory.json"));
        }

        env::current_dir()
            .map(|dir| dir.join("keyword-memory.json"))
            .map_err(|err| format!("无法解析存储目录：{err}"))
    }

    /// 将当前记忆模型写入本地 JSON 文件，并确保父目录存在。
    fn write_persisted_memory(memory: &PersistedMemory) -> Result<PathBuf, String> {
        let storage_path = Self::storage_file_path()?;

        if let Some(parent) = storage_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("无法创建存储目录 {}：{err}", parent.display()))?;
        }

        let json = serde_json::to_string_pretty(memory)
            .map_err(|err| format!("无法序列化本地记忆：{err}"))?;

        fs::write(&storage_path, json)
            .map_err(|err| format!("无法写入本地记忆文件 {}：{err}", storage_path.display()))?;

        Ok(storage_path)
    }

    /// 基于当前内存状态创建一份完整的持久化快照，包含前后缀与“添加空格”偏好。
    fn snapshot_persisted_memory(&self) -> PersistedMemory {
        PersistedMemory {
            prefixes: self.memory.prefixes.clone(),
            suffixes: self.memory.suffixes.clone(),
            insert_spaces: self.actions.insert_spaces,
        }
    }

    /// 将完整持久化快照写入磁盘，并同步刷新内存中的记忆与偏好状态。
    fn store_persisted_memory(&mut self, persisted: PersistedMemory) -> Result<PathBuf, String> {
        let path = Self::write_persisted_memory(&persisted)?;
        self.memory.prefixes = persisted.prefixes;
        self.memory.suffixes = persisted.suffixes;
        self.memory.insert_spaces = persisted.insert_spaces;
        self.memory.storage_path = Some(path.clone());
        self.memory.storage_status = format!(
            "本地记忆已更新：{} 个前缀、{} 个后缀；添加空格为 {}，文件位于 {}。",
            self.memory.prefixes.len(),
            self.memory.suffixes.len(),
            if self.memory.insert_spaces { "开启" } else { "关闭" },
            path.display()
        );
        Ok(path)
    }

    /// 将条目集合规范化为去空白、去重且保持原始顺序的列表。
    fn normalize_entries(entries: Vec<String>) -> Vec<String> {
        let mut normalized = Vec::new();

        for entry in entries {
            let trimmed = entry.trim();

            if trimmed.is_empty() {
                continue;
            }

            if normalized.iter().all(|existing| existing != trimmed) {
                normalized.push(trimmed.to_owned());
            }
        }

        normalized
    }

    /// 合并新增条目到目标集合中，并返回本次实际新增的数量。
    fn merge_unique_entries(target: &mut Vec<String>, incoming: &[String]) -> usize {
        let mut added = 0;

        for entry in incoming {
            if target.iter().all(|existing| existing != entry) {
                target.push(entry.clone());
                added += 1;
            }
        }

        added
    }

    /// 将多行文本解析为去空白、去重后的有效条目，并忽略纯空白行。
    fn parse_entries(buffer: &str) -> Vec<String> {
        let mut entries = Vec::new();

        for line in buffer.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                continue;
            }

            if entries.iter().all(|existing| existing != trimmed) {
                entries.push(trimmed.to_owned());
            }
        }

        entries
    }

    /// 判断三类输入中是否至少存在一个可参与生成的有效条目。
    fn has_any_entries(prefixes: &[String], keywords: &[String], suffixes: &[String]) -> bool {
        !prefixes.is_empty() || !keywords.is_empty() || !suffixes.is_empty()
    }

    /// 按“前缀 + 核心词 + 后缀”顺序生成所有有效组合，并兼容缺失维度。
    fn build_combinations(
        prefixes: &[String],
        keywords: &[String],
        suffixes: &[String],
        insert_spaces: bool,
    ) -> Vec<String> {
        if !Self::has_any_entries(prefixes, keywords, suffixes) {
            return Vec::new();
        }

        let prefix_values: Vec<&str> = if prefixes.is_empty() {
            vec![""]
        } else {
            prefixes.iter().map(String::as_str).collect()
        };
        let keyword_values: Vec<&str> = if keywords.is_empty() {
            vec![""]
        } else {
            keywords.iter().map(String::as_str).collect()
        };
        let suffix_values: Vec<&str> = if suffixes.is_empty() {
            vec![""]
        } else {
            suffixes.iter().map(String::as_str).collect()
        };
        let mut combinations = Vec::new();

        for prefix in &prefix_values {
            for keyword in &keyword_values {
                for suffix in &suffix_values {
                    combinations.push(Self::compose_keyword(
                        prefix,
                        keyword,
                        suffix,
                        insert_spaces,
                    ));
                }
            }
        }

        combinations
    }

    /// 将单条“前缀 + 核心词 + 后缀”组合成最终关键词，可按需在非空片段间插入空格。
    fn compose_keyword(prefix: &str, keyword: &str, suffix: &str, insert_spaces: bool) -> String {
        if !insert_spaces {
            return format!("{prefix}{keyword}{suffix}");
        }

        [prefix, keyword, suffix]
            .into_iter()
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// 生成输入条目统计文案，帮助界面在按钮和摘要区域展示当前解析状态。
    fn format_entry_summary(
        prefixes: &[String],
        keywords: &[String],
        suffixes: &[String],
    ) -> String {
        format!(
            "已解析前缀 {} 项、核心词 {} 项、后缀 {} 项",
            prefixes.len(),
            keywords.len(),
            suffixes.len()
        )
    }

    /// 渲染单个输入卡片，并在标题栏提供轻量记忆回填入口。
    fn render_editor_with_memory(
        ui: &mut Ui,
        state: &mut EditorPaneState,
        memory_entries: Option<&[String]>,
    ) -> EditorInteraction {
        let mut interaction = EditorInteraction::None;
        Frame::group(ui.style())
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::card_radius())
            .inner_margin(Margin::same(14))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(state.title)
                            .size(18.0)
                            .strong()
                            .color(theme::TEXT_PRIMARY),
                    );

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("清空").clicked() {
                            if Self::should_remove_memory_on_clear(&state.buffer, memory_entries) {
                                interaction = EditorInteraction::RemoveMemory;
                            } else {
                                state.buffer.clear();
                            }
                        }

                        if let Some(entries) = memory_entries {
                            if !entries.is_empty() && ui.button("回填全部").clicked() {
                                interaction = if Self::append_entries_to_buffer(&mut state.buffer, entries) > 0 {
                                    EditorInteraction::ReusedMemory
                                } else {
                                    EditorInteraction::None
                                };
                            }
                        }
                    });
                });
                ui.add_space(4.0);
                ui.label(RichText::new(state.hint).color(theme::TEXT_MUTED));
                ui.add_space(10.0);
                Frame::NONE
                    .fill(theme::INPUT_BG)
                    .stroke(Stroke::new(1.0, theme::BORDER))
                    .corner_radius(theme::card_radius())
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.add(
                            TextEdit::multiline(&mut state.buffer)
                                .desired_rows(18)
                                .hint_text(state.placeholder)
                                .text_color(theme::TEXT_PRIMARY)
                                .frame(false)
                                .lock_focus(true)
                                .desired_width(f32::INFINITY),
                        );
                    });
            });
        interaction
    }

    /// 判断当前“清空”点击是否应升级为删除已记忆前后缀的动作。
    fn should_remove_memory_on_clear(buffer: &str, memory_entries: Option<&[String]>) -> bool {
        buffer.trim().is_empty()
            && memory_entries
                .map(|entries| !entries.is_empty())
                .unwrap_or(false)
    }

    /// 删除指定类型的本地记忆，并同步刷新界面反馈与存储状态。
    fn clear_remembered_entries(&mut self, bucket: MemoryBucket) {
        let persisted = match bucket {
            MemoryBucket::Prefix if self.memory.prefixes.is_empty() => {
                self.results
                    .set_info_feedback("当前没有可删除的已记忆前缀。");
                return;
            }
            MemoryBucket::Suffix if self.memory.suffixes.is_empty() => {
                self.results
                    .set_info_feedback("当前没有可删除的已记忆后缀。");
                return;
            }
            MemoryBucket::Prefix => PersistedMemory {
                prefixes: Vec::new(),
                suffixes: self.memory.suffixes.clone(),
                insert_spaces: self.actions.insert_spaces,
            },
            MemoryBucket::Suffix => PersistedMemory {
                prefixes: self.memory.prefixes.clone(),
                suffixes: Vec::new(),
                insert_spaces: self.actions.insert_spaces,
            },
        };

        match self.store_persisted_memory(persisted) {
            Ok(_) => {
                self.results.set_success_feedback(match bucket {
                    MemoryBucket::Prefix => "已删除全部已记忆前缀；再次生成时会按新输入重新记录。".to_owned(),
                    MemoryBucket::Suffix => "已删除全部已记忆后缀；再次生成时会按新输入重新记录。".to_owned(),
                });
            }
            Err(err) => {
                self.memory.storage_status = format!("本地记忆保存失败：{err}");
                self.results
                    .set_error_feedback(format!("删除本地记忆失败：{err}"));
            }
        }
    }

    /// 将单个已记忆条目追加到输入框末尾，并避免写入重复内容。
    fn append_entry_to_buffer(buffer: &mut String, entry: &str) -> bool {
        let trimmed = entry.trim();

        if trimmed.is_empty() {
            return false;
        }

        if Self::parse_entries(buffer)
            .iter()
            .any(|existing| existing == trimmed)
        {
            return false;
        }

        if !buffer.trim().is_empty() && !buffer.ends_with('\n') {
            buffer.push('\n');
        }

        buffer.push_str(trimmed);
        true
    }

    /// 将多个已记忆条目批量回填到输入框，并返回本次实际追加的数量。
    fn append_entries_to_buffer(buffer: &mut String, entries: &[String]) -> usize {
        let mut appended = 0;

        for entry in entries {
            if Self::append_entry_to_buffer(buffer, entry) {
                appended += 1;
            }
        }

        appended
    }

    /// 渲染操作区，并在按钮点击时触发当前已实现的生成流程。
    fn render_action_panel(&mut self, ui: &mut Ui) {
        Frame::group(ui.style())
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::card_radius())
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("操作区")
                        .size(20.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(6.0);
                ui.label(RichText::new(&self.actions.status_message).color(theme::TEXT_MUTED));
                ui.add_space(14.0);

                let spacing_changed = ui
                    .checkbox(
                    &mut self.actions.insert_spaces,
                    RichText::new("添加空格").color(theme::TEXT_PRIMARY),
                )
                    .changed();
                if spacing_changed {
                    self.persist_spacing_preference();
                }
                ui.label(
                    RichText::new("勾选后，仅在相邻非空片段之间插入空格。")
                        .size(13.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(12.0);

                ui.horizontal_wrapped(|ui| {
                    let generate_clicked = ui
                        .add_enabled(
                            self.actions.generate_enabled,
                            Button::new(self.actions.primary_label)
                                .min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked();
                    let copy_clicked = ui
                        .add_enabled(
                            self.actions.copy_enabled,
                            Button::new("复制全部").min_size(Vec2::new(120.0, 36.0)),
                        )
                        .clicked();

                    if generate_clicked {
                        self.handle_generate();
                        self.refresh_action_state();
                    }

                    if copy_clicked {
                        self.handle_copy_all();
                        self.refresh_action_state();
                    }
                });

                ui.add_space(10.0);
                ui.label(
                    RichText::new(format!("生成提示：{}", self.actions.generate_hint))
                        .size(13.0)
                        .color(theme::TEXT_MUTED),
                );
                ui.label(
                    RichText::new(format!("复制提示：{}", self.actions.copy_hint))
                        .size(13.0)
                        .color(theme::TEXT_MUTED),
                );

                ui.add_space(14.0);
                ui.separator();
                ui.add_space(10.0);
                ui.label(
                    RichText::new(self.actions.secondary_label)
                        .size(13.0)
                        .color(theme::ACCENT_SOFT),
                );
            });
    }

    /// 持久化“添加空格”复选框状态，并让下次启动自动恢复当前偏好。
    fn persist_spacing_preference(&mut self) {
        match self.store_persisted_memory(self.snapshot_persisted_memory()) {
            Ok(_) => {
                self.results.set_info_feedback(format!(
                    "已保存“添加空格”设置：{}。下次启动会自动恢复。",
                    if self.actions.insert_spaces { "开启" } else { "关闭" }
                ));
            }
            Err(err) => {
                self.memory.storage_status = format!("本地记忆保存失败：{err}");
                self.results
                    .set_error_feedback(format!("保存“添加空格”设置失败：{err}"));
            }
        }
    }

    /// 渲染结果区，并提供只读可选中的文本视图与复制反馈展示。
    fn render_results_panel(&mut self, ui: &mut Ui) {
        let state = &self.results;
        let preview_text = state.preview_text.as_str();
        let show_empty_state = !state.has_copyable_results();
        let has_pending_input = self.actions.generate_enabled;

        Frame::group(ui.style())
            .fill(theme::SURFACE)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::card_radius())
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("结果区")
                        .size(20.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new(format!("候选结果数：{}", state.total_count))
                        .color(theme::TEXT_MUTED),
                );
                ui.add_space(4.0);
                
                ui.add_space(10.0);

                if show_empty_state {
                    Self::render_result_empty_state(ui, has_pending_input);
                } else {
                    Frame::NONE
                        .fill(theme::INPUT_BG)
                        .stroke(Stroke::new(1.0, theme::BORDER))
                        .corner_radius(theme::card_radius())
                        .inner_margin(Margin::same(8))
                        .show(ui, |ui| {
                            let row_height = ui.text_style_height(&egui::TextStyle::Body);
                            let fixed_height = row_height * 6.0 + 12.0;

                            ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .max_height(fixed_height)
                                .show(ui, |ui| {
                                    let label = egui::Label::new(
                                        RichText::new(preview_text).color(theme::TEXT_PRIMARY),
                                    )
                                    .selectable(true)
                                    .wrap();
                                    ui.add(label);
                                });
                        });
                }

                ui.add_space(10.0);
                ui.colored_label(state.feedback_color(), &state.feedback);
            });
    }

    /// 渲染结果区空态，在未生成结果时明确说明下一步操作和当前阻塞原因。
    fn render_result_empty_state(ui: &mut Ui, has_pending_input: bool) {
        Frame::group(ui.style())
            .fill(theme::PANEL)
            .stroke(Stroke::new(1.0, theme::BORDER))
            .corner_radius(theme::card_radius())
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("结果区暂时为空")
                        .size(18.0)
                        .strong()
                        .color(theme::TEXT_PRIMARY),
                );
                ui.add_space(6.0);
                if has_pending_input {
                    ui.label(
                        RichText::new("已检测到有效输入，点击左侧“生成结果”后会在这里展示完整组合。")
                            .color(theme::TEXT_MUTED),
                    );
                } else {
                    ui.label(
                        RichText::new("先在左侧输入至少一个非空前缀、核心词或后缀，再生成候选结果。")
                            .color(theme::TEXT_MUTED),
                    );
                }
                ui.add_space(10.0);
                ui.label(RichText::new("空白行会被忽略，前缀和后缀允许留空。").color(theme::ACCENT_SOFT));
                
            });
    }
}

impl eframe::App for KeywordApp {
    /// 在每一帧中按“上层三列输入 + 下层操作结果”的新布局刷新界面。
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.refresh_action_state();
        self.render_workspace(ctx);
    }
}

impl InputState {
    /// 创建输入区默认状态，并预置符合关键词工具场景的引导文案。
    fn new() -> Self {
        Self {
            prefix: EditorPaneState::new(
                "前缀",
                "用于放置常用修饰词、渠道词、意图词。",
                "例如:\n免费\n热门\nAI",
            ),
            keyword: EditorPaneState::new(
                "核心词",
                "后续生成逻辑会以这里作为主要组合中心。",
                "例如:\n关键词工具\n桌面应用",
            ),
            suffix: EditorPaneState::new(
                "后缀",
                "适合放置行业词、场景词和转化词。",
                "例如:\n推荐\n下载\n教程",
            ),
        }
    }
}

impl EditorPaneState {
    /// 创建单个编辑卡片的标题、提示文案和占位内容。
    fn new(title: &'static str, hint: &'static str, placeholder: &'static str) -> Self {
        Self {
            title,
            hint,
            placeholder,
            buffer: String::new(),
        }
    }
}

impl ActionPanelState {
    /// 创建操作区默认状态，并补充本地记忆行为的说明文案。
    fn new() -> Self {
        Self {
            generate_enabled: false,
            copy_enabled: false,
            insert_spaces: false,
            primary_label: "生成结果",
            secondary_label: "前缀和后缀可留空；生成成功后会自动写入本地记忆。",
            status_message: "请输入至少一个非空条目后开始生成。".to_owned(),
            generate_hint: "生成按钮当前不可用：请先输入至少一个非空前缀、核心词或后缀。".to_owned(),
            copy_hint: "复制按钮当前不可用：请先生成至少一条结果。".to_owned(),
        }
    }
}

impl ResultPanelState {
    /// 创建结果区默认状态，并补充可选择文本与复制全部的使用提示。
    fn new() -> Self {
        Self {
            preview_text: String::new(),
            total_count: 0,
            feedback: "结果生成后会在此处展示；支持局部选择复制，也支持一键复制全部。"
                .to_owned(),
            feedback_tone: FeedbackTone::Info,
        }
    }

    /// 判断结果区当前是否存在可直接写入剪贴板的生成结果。
    fn has_copyable_results(&self) -> bool {
        self.total_count > 0 && !self.preview_text.trim().is_empty()
    }

    /// 设置普通提示反馈，适用于引导说明和非错误状态切换。
    fn set_info_feedback(&mut self, message: impl Into<String>) {
        self.feedback = message.into();
        self.feedback_tone = FeedbackTone::Info;
    }

    /// 设置成功反馈，突出生成或复制成功等正向结果。
    fn set_success_feedback(&mut self, message: impl Into<String>) {
        self.feedback = message.into();
        self.feedback_tone = FeedbackTone::Success;
    }

    /// 设置失败反馈，便于用户感知当前操作未生效的原因。
    fn set_error_feedback(&mut self, message: impl Into<String>) {
        self.feedback = message.into();
        self.feedback_tone = FeedbackTone::Error;
    }

    /// 根据当前反馈语义返回结果区应使用的提示色。
    fn feedback_color(&self) -> egui::Color32 {
        match self.feedback_tone {
            FeedbackTone::Info => theme::ACCENT_SOFT,
            FeedbackTone::Success => theme::ACCENT,
            FeedbackTone::Error => egui::Color32::from_rgb(255, 120, 120),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证多行输入会被裁剪空白、忽略空行并按首次出现顺序去重。
    #[test]
    fn parse_entries_trims_blanks_and_deduplicates() {
        let parsed = KeywordApp::parse_entries("  免费  \n\nAI\n免费\n  \n AI工具 ");

        assert_eq!(parsed, vec!["免费", "AI", "AI工具"]);
    }

    /// 验证组合逻辑在缺失部分维度时仍能输出稳定结果，并在完全空输入时返回空列表。
    #[test]
    fn build_combinations_handles_missing_dimensions() {
        let only_keywords =
            KeywordApp::build_combinations(&[], &["工具".to_owned(), "教程".to_owned()], &[], false);
        let prefix_and_suffix = KeywordApp::build_combinations(
            &["AI".to_owned()],
            &[],
            &["下载".to_owned(), "推荐".to_owned()],
            false,
        );
        let empty = KeywordApp::build_combinations(&[], &[], &[], false);

        assert_eq!(only_keywords, vec!["工具", "教程"]);
        assert_eq!(prefix_and_suffix, vec!["AI下载", "AI推荐"]);
        assert!(empty.is_empty());
    }

    /// 验证启用“添加空格”后，只在非空片段之间插入空格，并避免产生多余前后空格。
    #[test]
    fn build_combinations_inserts_spaces_only_between_non_empty_segments() {
        let keyword_and_suffix = KeywordApp::build_combinations(
            &[],
            &["关键词".to_owned()],
            &["下载".to_owned()],
            true,
        );
        let full = KeywordApp::build_combinations(
            &["免费".to_owned()],
            &["AI工具".to_owned()],
            &["下载".to_owned()],
            true,
        );

        assert_eq!(keyword_and_suffix, vec!["关键词 下载"]);
        assert_eq!(full, vec!["免费 AI工具 下载"]);
    }

    /// 验证只有在输入框已空且确实存在本地记忆时，二次点击“清空”才会触发删除记忆。
    #[test]
    fn should_remove_memory_on_clear_only_when_buffer_is_empty_and_memory_exists() {
        let remembered = vec!["免费".to_owned()];

        assert!(KeywordApp::should_remove_memory_on_clear("", Some(&remembered)));
        assert!(KeywordApp::should_remove_memory_on_clear("   ", Some(&remembered)));
        assert!(!KeywordApp::should_remove_memory_on_clear("AI", Some(&remembered)));
        assert!(!KeywordApp::should_remove_memory_on_clear("", Some(&[])));
        assert!(!KeywordApp::should_remove_memory_on_clear("", None));
    }

    /// 验证旧版本地记忆文件缺少“添加空格”字段时，会兼容回退为关闭状态。
    #[test]
    fn persisted_memory_defaults_insert_spaces_when_field_is_missing() {
        let persisted: PersistedMemory =
            serde_json::from_str(r#"{"prefixes":["免费"],"suffixes":["下载"]}"#).unwrap();

        assert_eq!(persisted.prefixes, vec!["免费"]);
        assert_eq!(persisted.suffixes, vec!["下载"]);
        assert!(!persisted.insert_spaces);
    }

    /// 验证批量回填会保留换行结构，并避免把已存在条目重复写回输入框。
    #[test]
    fn append_entries_to_buffer_avoids_duplicates_and_preserves_newlines() {
        let mut buffer = "免费".to_owned();
        let entries = vec!["免费".to_owned(), "AI".to_owned(), "教程".to_owned()];

        let appended = KeywordApp::append_entries_to_buffer(&mut buffer, &entries);

        assert_eq!(appended, 2);
        assert_eq!(buffer, "免费\nAI\n教程");
    }

    /// 验证操作区状态会根据输入与结果同步刷新按钮可用性和提示文案。
    #[test]
    fn refresh_action_state_updates_button_hints() {
        let mut app = KeywordApp {
            input: InputState::new(),
            actions: ActionPanelState::new(),
            results: ResultPanelState::new(),
            memory: RememberedEntriesState {
                prefixes: Vec::new(),
                suffixes: Vec::new(),
                insert_spaces: false,
                storage_path: None,
                storage_status: String::new(),
            },
        };

        app.refresh_action_state();
        assert!(!app.actions.generate_enabled);
        assert!(!app.actions.copy_enabled);
        assert!(app.actions.generate_hint.contains("当前不可用"));

        app.input.prefix.buffer = "免费".to_owned();
        app.input.keyword.buffer = "AI工具".to_owned();
        app.results.preview_text = "免费AI工具".to_owned();
        app.results.total_count = 1;

        app.refresh_action_state();
        assert!(app.actions.generate_enabled);
        assert!(app.actions.copy_enabled);
        assert!(app.actions.status_message.contains("最近一次生成了 1 条结果"));
        assert!(app.actions.copy_hint.contains("1 条结果"));
    }
}
