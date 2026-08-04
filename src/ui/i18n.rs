use serde::{Deserialize, Serialize};

/// Persisted language preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LanguagePreference {
    #[default]
    #[serde(rename = "auto")]
    Auto,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "zh-Hans")]
    ZhHans,
    #[serde(rename = "zh-Hant")]
    ZhHant,
}

impl LanguagePreference {
    pub const ALL: [Self; 4] = [Self::Auto, Self::English, Self::ZhHans, Self::ZhHant];

    /// Fixed option labels, independent of the active UI locale.
    pub fn label(self) -> &'static str {
        match self {
            Self::Auto => "Auto",
            Self::English => "English",
            Self::ZhHans => "简体中文",
            Self::ZhHant => "繁體中文",
        }
    }

    pub fn cycle(self, forward: bool) -> Self {
        let idx = Self::ALL.iter().position(|p| *p == self).unwrap_or(0);
        let next = if forward {
            (idx + 1) % Self::ALL.len()
        } else {
            (idx + Self::ALL.len() - 1) % Self::ALL.len()
        };
        Self::ALL[next]
    }
}

/// Resolved active UI locale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    En,
    ZhHans,
    ZhHant,
}

impl Locale {
    pub fn resolve(pref: LanguagePreference) -> Self {
        match pref {
            LanguagePreference::English => Self::En,
            LanguagePreference::ZhHans => Self::ZhHans,
            LanguagePreference::ZhHant => Self::ZhHant,
            LanguagePreference::Auto => {
                let raw = sys_locale::get_locale().unwrap_or_default();
                Self::from_locale_str(&raw)
            }
        }
    }

    /// Resolve Auto rules against a locale string (also used by unit tests).
    pub fn from_locale_str(raw: &str) -> Self {
        let s = raw.trim().to_lowercase().replace('_', "-");
        if s.is_empty() {
            return Self::En;
        }

        let is_zh = s == "zh" || s.starts_with("zh-");
        if is_zh {
            if s.contains("hant")
                || has_tag(&s, "tw")
                || has_tag(&s, "hk")
                || has_tag(&s, "mo")
            {
                return Self::ZhHant;
            }
            if s.contains("hans") || has_tag(&s, "cn") || has_tag(&s, "sg") {
                return Self::ZhHans;
            }
            // Bare or ambiguous Chinese → Simplified
            return Self::ZhHans;
        }

        if s == "en" || s.starts_with("en-") {
            return Self::En;
        }

        Self::En
    }
}

fn has_tag(locale: &str, tag: &str) -> bool {
    locale.split(['-', '.']).any(|p| p == tag)
}

/// Localized TUI chrome strings. Every locale fills every field.
#[derive(Debug, Clone, Copy)]
pub struct UiStrings {
    // Toolbar
    pub toolbar_dev: &'static str,
    pub toolbar_pause: &'static str,
    pub toolbar_resume: &'static str,
    pub toolbar_clear: &'static str,
    pub toolbar_follow: &'static str,
    pub toolbar_wrap: &'static str,
    pub toolbar_export: &'static str,
    pub toolbar_settings: &'static str,
    pub toolbar_quit: &'static str,
    pub none_device: &'static str,

    // Filters
    pub filter_tag: &'static str,
    pub filter_message: &'static str,
    pub filter_level: &'static str,
    pub filter_click_hint: &'static str,

    // Find
    pub find_prefix: &'static str,
    pub find_help_suffix: &'static str,
    pub find_zero_matches: &'static str,

    // Empty state
    pub empty_logs: &'static str,

    // Status bar
    pub status_live: &'static str,
    pub status_idle: &'static str,
    pub focus_logs: &'static str,
    pub focus_level: &'static str,
    pub focus_find: &'static str,
    pub focus_modal: &'static str,
    pub wrap_on: &'static str,
    pub wrap_off: &'static str,

    // Ephemeral status
    pub status_exported_to: &'static str,
    pub status_copied: &'static str,
    pub status_copy_failed: &'static str,
    pub tip_keyboard_enhancement: &'static str,

    // Devices modal
    pub modal_devices_title: &'static str,
    pub modal_devices_help: &'static str,

    // Export modal / menu
    pub modal_export_title: &'static str,
    pub modal_export_filtered: &'static str,
    pub modal_export_all: &'static str,
    pub modal_export_cancel: &'static str,
    pub modal_export_path_prompt: &'static str,
    pub modal_export_filtered_title: &'static str,
    pub modal_export_all_title: &'static str,

    // Settings modal
    pub modal_settings_title: &'static str,
    pub modal_settings_help: &'static str,
    pub settings_adb: &'static str,
    pub settings_adb_auto: &'static str,
    pub settings_adb_custom: &'static str,
    pub settings_adb_locked_hint: &'static str,
    pub settings_preset: &'static str,
    pub settings_custom: &'static str,
    pub settings_language: &'static str,
    pub settings_adb_not_found: &'static str,

    // Filter edit modal
    pub modal_tag_filter_title: &'static str,
    pub modal_message_filter_title: &'static str,
    pub filter_tag_contains: &'static str,
    pub filter_message_contains: &'static str,
    pub filter_live_hint: &'static str,
}

impl UiStrings {
    pub fn for_locale(locale: Locale) -> Self {
        match locale {
            Locale::En => Self::en(),
            Locale::ZhHans => Self::zh_hans(),
            Locale::ZhHant => Self::zh_hant(),
        }
    }

    fn en() -> Self {
        Self {
            toolbar_dev: "Device",
            toolbar_pause: "Pause",
            toolbar_resume: "Resume",
            toolbar_clear: "Clear",
            toolbar_follow: "Follow",
            toolbar_wrap: "Wrap",
            toolbar_export: "Export",
            toolbar_settings: "Settings",
            toolbar_quit: "Quit",
            none_device: "(none)",

            filter_tag: "Tag",
            filter_message: "Message",
            filter_level: "Level",
            filter_click_hint: "  (click Tag/Message)",

            find_prefix: "Find:[",
            find_help_suffix: "  (Enter next · Shift+Enter prev · Esc close)",
            find_zero_matches: "0 matches",

            empty_logs: "No logs — press [d] to select a device",

            status_live: "● Live",
            status_idle: "○ Idle",
            focus_logs: "focus:logs",
            focus_level: "focus:level",
            focus_find: "focus:find",
            focus_modal: "focus:modal",
            wrap_on: "wrap:on",
            wrap_off: "wrap:off",

            status_exported_to: "Exported to {}",
            status_copied: "Copied selection",
            status_copy_failed: "Copy failed: {}",
            tip_keyboard_enhancement:
                "Tip: keyboard enhancement unavailable — use Windows Terminal for full key support",

            modal_devices_title: " Devices ",
            modal_devices_help: "Select device  (↑↓ · enter · r refresh · esc exit)",

            modal_export_title: " Export ",
            modal_export_filtered: "[1]/f] export filtered",
            modal_export_all: "[2]/a] export all",
            modal_export_cancel: "esc cancel",
            modal_export_path_prompt: "Path (enter confirm · esc cancel):",
            modal_export_filtered_title: " export filtered ",
            modal_export_all_title: " export all ",

            modal_settings_title: " Settings ",
            modal_settings_help:
                "↑/↓ move · ←/→ adjust · ADB: e edit · r restore · esc exit edit/dismiss · Custom: type digits · enter dismiss",
            settings_adb: "ADB",
            settings_adb_auto: "Auto",
            settings_adb_custom: "Custom",
            settings_adb_locked_hint: "  (e edit · r restore)",
            settings_preset: "Preset",
            settings_custom: "Custom",
            settings_language: "Language",
            settings_adb_not_found: "  (adb not found)",

            modal_tag_filter_title: " Tag filter ",
            modal_message_filter_title: " Message filter ",
            filter_tag_contains: "Tag contains:",
            filter_message_contains: "Message contains:",
            filter_live_hint: "Live filter · esc clear/close · enter done",
        }
    }

    fn zh_hans() -> Self {
        Self {
            toolbar_dev: "设备",
            toolbar_pause: "暂停",
            toolbar_resume: "继续",
            toolbar_clear: "清除",
            toolbar_follow: "跟随",
            toolbar_wrap: "换行",
            toolbar_export: "导出",
            toolbar_settings: "设置",
            toolbar_quit: "退出",
            none_device: "(无)",

            filter_tag: "标签",
            filter_message: "消息",
            filter_level: "级别",
            filter_click_hint: "  (点击标签/消息)",

            find_prefix: "查找:[",
            find_help_suffix: "  (Enter 下一个 · Shift+Enter 上一个 · Esc 关闭)",
            find_zero_matches: "0 个匹配",

            empty_logs: "暂无日志 — 按 [d] 选择设备",

            status_live: "● 实时",
            status_idle: "○ 空闲",
            focus_logs: "焦点:日志",
            focus_level: "焦点:级别",
            focus_find: "焦点:查找",
            focus_modal: "焦点:弹窗",
            wrap_on: "换行:开",
            wrap_off: "换行:关",

            status_exported_to: "已导出到 {}",
            status_copied: "已复制选区",
            status_copy_failed: "复制失败: {}",
            tip_keyboard_enhancement:
                "提示: 键盘增强不可用 — 请使用 Windows Terminal 以获得完整按键支持",

            modal_devices_title: " 设备 ",
            modal_devices_help: "选择设备  (↑↓ · Enter · r 刷新 · Esc)",

            modal_export_title: " 导出 ",
            modal_export_filtered: "[1]/f] 导出筛选结果",
            modal_export_all: "[2]/a] 导出全部",
            modal_export_cancel: "Esc 取消",
            modal_export_path_prompt: "路径 (Enter 确认 · Esc 取消):",
            modal_export_filtered_title: " 导出筛选结果 ",
            modal_export_all_title: " 导出全部 ",

            modal_settings_title: " 设置 ",
            modal_settings_help:
                "↑/↓ 移动 · ←/→ 调整 · ADB: e 编辑 · r 自动 · Esc 退出编辑/关闭 · 自定义: 输入数字 · Enter 关闭",
            settings_adb: "ADB",
            settings_adb_auto: "自动",
            settings_adb_custom: "自定义",
            settings_adb_locked_hint: "  (e 编辑 · r 恢复自动)",
            settings_preset: "预设",
            settings_custom: "自定义",
            settings_language: "语言",
            settings_adb_not_found: "  (未找到 adb)",

            modal_tag_filter_title: " 标签筛选 ",
            modal_message_filter_title: " 消息筛选 ",
            filter_tag_contains: "标签包含:",
            filter_message_contains: "消息包含:",
            filter_live_hint: "实时筛选 · Esc 清空/关闭 · Enter 完成",
        }
    }

    fn zh_hant() -> Self {
        Self {
            toolbar_dev: "裝置",
            toolbar_pause: "暫停",
            toolbar_resume: "繼續",
            toolbar_clear: "清除",
            toolbar_follow: "跟隨",
            toolbar_wrap: "換行",
            toolbar_export: "匯出",
            toolbar_settings: "設定",
            toolbar_quit: "退出",
            none_device: "(無)",

            filter_tag: "標籤",
            filter_message: "訊息",
            filter_level: "級別",
            filter_click_hint: "  (點擊標籤/訊息)",

            find_prefix: "尋找:[",
            find_help_suffix: "  (Enter 下一個 · Shift+Enter 上一個 · Esc 關閉)",
            find_zero_matches: "0 個符合",

            empty_logs: "暫無日誌 — 按 [d] 選擇裝置",

            status_live: "● 即時",
            status_idle: "○ 閒置",
            focus_logs: "焦點:日誌",
            focus_level: "焦點:級別",
            focus_find: "焦點:尋找",
            focus_modal: "焦點:彈窗",
            wrap_on: "換行:開",
            wrap_off: "換行:關",

            status_exported_to: "已匯出到 {}",
            status_copied: "已複製選區",
            status_copy_failed: "複製失敗: {}",
            tip_keyboard_enhancement:
                "提示: 鍵盤增強不可用 — 請使用 Windows Terminal 以獲得完整按鍵支援",

            modal_devices_title: " 裝置 ",
            modal_devices_help: "選擇裝置  (↑↓ · Enter · r 重新整理 · Esc)",

            modal_export_title: " 匯出 ",
            modal_export_filtered: "[1]/f] 匯出篩選結果",
            modal_export_all: "[2]/a] 匯出全部",
            modal_export_cancel: "Esc 取消",
            modal_export_path_prompt: "路徑 (Enter 確認 · Esc 取消):",
            modal_export_filtered_title: " 匯出篩選結果 ",
            modal_export_all_title: " 匯出全部 ",

            modal_settings_title: " 設定 ",
            modal_settings_help:
                "↑/↓ 移動 · ←/→ 調整 · ADB: e 編輯 · r 自動 · Esc 退出編輯/關閉 · 自訂: 輸入數字 · Enter 關閉",
            settings_adb: "ADB",
            settings_adb_auto: "自動",
            settings_adb_custom: "自訂",
            settings_adb_locked_hint: "  (e 編輯 · r 恢復自動)",
            settings_preset: "預設",
            settings_custom: "自訂",
            settings_language: "語言",
            settings_adb_not_found: "  (找不到 adb)",

            modal_tag_filter_title: " 標籤篩選 ",
            modal_message_filter_title: " 訊息篩選 ",
            filter_tag_contains: "標籤包含:",
            filter_message_contains: "訊息包含:",
            filter_live_hint: "即時篩選 · Esc 清空/關閉 · Enter 完成",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_preference_cycles() {
        assert_eq!(
            LanguagePreference::Auto.cycle(true),
            LanguagePreference::English
        );
        assert_eq!(
            LanguagePreference::ZhHant.cycle(true),
            LanguagePreference::Auto
        );
        assert_eq!(
            LanguagePreference::English.cycle(false),
            LanguagePreference::Auto
        );
    }

    #[test]
    fn language_labels_are_fixed() {
        assert_eq!(LanguagePreference::Auto.label(), "Auto");
        assert_eq!(LanguagePreference::English.label(), "English");
        assert_eq!(LanguagePreference::ZhHans.label(), "简体中文");
        assert_eq!(LanguagePreference::ZhHant.label(), "繁體中文");
    }

    #[test]
    fn auto_resolve_simplified() {
        assert_eq!(Locale::from_locale_str("zh-CN"), Locale::ZhHans);
        assert_eq!(Locale::from_locale_str("zh-Hans"), Locale::ZhHans);
        assert_eq!(Locale::from_locale_str("zh_CN"), Locale::ZhHans);
        assert_eq!(Locale::from_locale_str("zh-SG"), Locale::ZhHans);
        assert_eq!(Locale::from_locale_str("zh-Hans-CN"), Locale::ZhHans);
    }

    #[test]
    fn auto_resolve_traditional() {
        assert_eq!(Locale::from_locale_str("zh-TW"), Locale::ZhHant);
        assert_eq!(Locale::from_locale_str("zh-HK"), Locale::ZhHant);
        assert_eq!(Locale::from_locale_str("zh-MO"), Locale::ZhHant);
        assert_eq!(Locale::from_locale_str("zh-Hant"), Locale::ZhHant);
        assert_eq!(Locale::from_locale_str("zh_TW"), Locale::ZhHant);
    }

    #[test]
    fn auto_resolve_bare_zh() {
        assert_eq!(Locale::from_locale_str("zh"), Locale::ZhHans);
    }

    #[test]
    fn auto_resolve_english() {
        assert_eq!(Locale::from_locale_str("en"), Locale::En);
        assert_eq!(Locale::from_locale_str("en-US"), Locale::En);
        assert_eq!(Locale::from_locale_str("en_GB"), Locale::En);
    }

    #[test]
    fn auto_resolve_unsupported_and_missing() {
        assert_eq!(Locale::from_locale_str("ja-JP"), Locale::En);
        assert_eq!(Locale::from_locale_str("de-DE"), Locale::En);
        assert_eq!(Locale::from_locale_str(""), Locale::En);
        assert_eq!(Locale::from_locale_str("   "), Locale::En);
    }

    #[test]
    fn explicit_preference_ignores_system() {
        assert_eq!(
            Locale::resolve(LanguagePreference::English),
            Locale::En
        );
        assert_eq!(
            Locale::resolve(LanguagePreference::ZhHans),
            Locale::ZhHans
        );
        assert_eq!(
            Locale::resolve(LanguagePreference::ZhHant),
            Locale::ZhHant
        );
    }

    #[test]
    fn ui_strings_cover_all_locales() {
        let _ = UiStrings::for_locale(Locale::En);
        let _ = UiStrings::for_locale(Locale::ZhHans);
        let _ = UiStrings::for_locale(Locale::ZhHant);
    }

    #[test]
    fn language_serde_tags() {
        assert_eq!(
            serde_json::to_string(&LanguagePreference::Auto).unwrap(),
            "\"auto\""
        );
        assert_eq!(
            serde_json::to_string(&LanguagePreference::English).unwrap(),
            "\"en\""
        );
        assert_eq!(
            serde_json::to_string(&LanguagePreference::ZhHans).unwrap(),
            "\"zh-Hans\""
        );
        assert_eq!(
            serde_json::to_string(&LanguagePreference::ZhHant).unwrap(),
            "\"zh-Hant\""
        );
    }
}
