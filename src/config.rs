//! Persistent configuration (UTF-8 INI in %APPDATA%\VisualKeySwitcher).

use std::collections::HashMap;
use std::path::Path;

use crate::system::{self, ToggleBackup};
use crate::util::{self, MOD_CTRL, MOD_SHIFT};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Next,
    Lang(String), // lowercase ISO 639-1, e.g. "ru" / "en"
}

impl Action {
    pub fn to_ini(&self) -> String {
        match self {
            Action::Next => "next".to_string(),
            Action::Lang(iso) => format!("lang:{}", iso),
        }
    }

    pub fn from_ini(s: &str) -> Action {
        let s = s.trim();
        if let Some(iso) = s.strip_prefix("lang:") {
            let iso = iso.trim().to_lowercase();
            if !iso.is_empty() {
                return Action::Lang(iso);
            }
        }
        Action::Next
    }

    pub fn label(&self) -> String {
        match self {
            Action::Next => "Следующий язык".to_string(),
            Action::Lang(iso) => match iso.as_str() {
                "ru" => "Всегда русский".to_string(),
                "en" => "Всегда английский".to_string(),
                other => format!("Всегда «{}»", other.to_uppercase()),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub visual: bool,
    pub sound: bool,
    pub autostart: bool,
    pub block_native: bool,
    pub hotkey_mods: u8,
    pub hotkey_vk: u16,
    pub trigger_on_press: bool,
    pub action: Action,
    pub visual_type: u8,
    pub sound_pack: String,
    pub overlay_ms: u32,
    pub debug: bool,
    pub toggle_backup: Option<ToggleBackup>,
    /// True while our program has the Windows toggle disabled. Persisted so
    /// a crash can be recovered on the next start.
    pub block_applied: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            visual: true,
            sound: true,
            autostart: false,
            block_native: true,
            hotkey_mods: MOD_CTRL | MOD_SHIFT,
            hotkey_vk: 0,
            trigger_on_press: true,
            action: Action::Next,
            visual_type: 1,
            sound_pack: String::new(),
            overlay_ms: 700,
            debug: false,
            toggle_backup: None,
            block_applied: false,
        }
    }
}

fn parse_bool(v: Option<&String>, default: bool) -> bool {
    match v.map(|s| s.trim()) {
        Some("1") | Some("true") | Some("yes") | Some("on") => true,
        Some("0") | Some("false") | Some("no") | Some("off") => false,
        _ => default,
    }
}

fn parse_u32(v: Option<&String>, default: u32) -> u32 {
    v.and_then(|s| s.trim().parse::<u32>().ok())
        .unwrap_or(default)
}

fn parse_u16(v: Option<&String>, default: u16) -> u16 {
    v.and_then(|s| s.trim().parse::<u16>().ok())
        .unwrap_or(default)
}

fn parse_u8(v: Option<&String>, default: u8) -> u8 {
    v.and_then(|s| s.trim().parse::<u8>().ok())
        .unwrap_or(default)
}

fn read_ini(path: &Path) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let Ok(mut text) = std::fs::read_to_string(path) else {
        return map;
    };
    if text.starts_with('\u{feff}') {
        text.remove(0);
    }
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty()
            || line.starts_with(';')
            || line.starts_with('#')
            || line.starts_with('[')
        {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            map.insert(k.trim().to_ascii_lowercase(), v.trim().to_string());
        }
    }
    map
}

impl Config {
    pub fn load_or_migrate(path: &Path) -> Config {
        let existed = path.is_file();
        let map = read_ini(path);
        let mut cfg = Config::default();

        if existed {
            cfg.visual = parse_bool(map.get("visual"), cfg.visual);
            cfg.sound = parse_bool(map.get("sound"), cfg.sound);
            cfg.autostart = parse_bool(map.get("autostart"), cfg.autostart);
            cfg.block_native = parse_bool(map.get("blocksystemhotkeys"), cfg.block_native);
            cfg.hotkey_mods = parse_u8(map.get("hotkeymods"), cfg.hotkey_mods);
            cfg.hotkey_vk = parse_u16(map.get("hotkeyvk"), cfg.hotkey_vk);
            cfg.trigger_on_press = parse_bool(map.get("triggeronpress"), true);
            if let Some(v) = map.get("action") {
                cfg.action = Action::from_ini(v);
            }
            cfg.visual_type = parse_u8(map.get("visualtype"), cfg.visual_type).min(2);
            cfg.sound_pack = map.get("soundpack").cloned().unwrap_or_default();
            cfg.overlay_ms = parse_u32(map.get("overlayms"), cfg.overlay_ms).clamp(150, 5000);
            cfg.debug = parse_bool(map.get("debug"), cfg.debug);
            cfg.block_applied = parse_bool(map.get("systemblockapplied"), cfg.block_applied);

            if parse_bool(map.get("backupvalid"), false) {
                cfg.toggle_backup = Some(ToggleBackup {
                    language: map
                        .get("systemlanguagehotkey")
                        .filter(|s| !s.is_empty())
                        .cloned(),
                    layout: map
                        .get("systemlayouthotkey")
                        .filter(|s| !s.is_empty())
                        .cloned(),
                    legacy: map.get("systemhotkey").filter(|s| !s.is_empty()).cloned(),
                });
            }
        } else {
            // One-time import of settings from the old C version.
            let legacy = system::read_legacy_settings();
            if legacy.exists {
                if let Some(v) = legacy.visual_marker {
                    cfg.visual = v != 0;
                }
                if let Some(v) = legacy.sound {
                    cfg.sound = v != 0;
                }
                if let Some(v) = legacy.hotkey_mods {
                    cfg.hotkey_mods = match v {
                        3 => MOD_CTRL | MOD_SHIFT,
                        6 => util::MOD_SHIFT | util::MOD_ALT,
                        _ => MOD_CTRL | MOD_SHIFT,
                    };
                }
                if let Some(v) = legacy.visual_type {
                    cfg.visual_type = v.min(2) as u8;
                }
                if let Some(pack) = legacy.sound_pack {
                    if !pack.is_empty() && pack != "default" {
                        cfg.sound_pack = pack;
                    }
                }
                util::log("Настройки импортированы из старой версии (реестр)");
            }
        }

        if !existed {
            cfg.save(path);
        }
        cfg
    }

    pub fn save(&self, path: &Path) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut s = String::new();
        s.push_str("; VisualKeySwitcher 2.x\n");
        s.push_str("; true/false можно писать как 1/0\n\n");
        s.push_str(&format!("Visual={}\n", self.visual as u8));
        s.push_str(&format!("Sound={}\n", self.sound as u8));
        s.push_str(&format!("AutoStart={}\n", self.autostart as u8));
        s.push_str(&format!("BlockSystemHotkeys={}\n", self.block_native as u8));
        s.push_str(&format!("HotkeyMods={}\n", self.hotkey_mods));
        s.push_str(&format!("HotkeyVk={}\n", self.hotkey_vk));
        s.push_str(&format!("TriggerOnPress={}\n", self.trigger_on_press as u8));
        s.push_str(&format!("Action={}\n", self.action.to_ini()));
        s.push_str(&format!("VisualType={}\n", self.visual_type));
        s.push_str(&format!("SoundPack={}\n", self.sound_pack));
        s.push_str(&format!("OverlayMs={}\n", self.overlay_ms));
        s.push_str(&format!("Debug={}\n", self.debug as u8));
        s.push_str(&format!(
            "SystemBlockApplied={}\n",
            self.block_applied as u8
        ));
        s.push_str(&format!(
            "BackupValid={}\n",
            self.toggle_backup.is_some() as u8
        ));
        if let Some(b) = &self.toggle_backup {
            s.push_str(&format!(
                "SystemLanguageHotkey={}\n",
                b.language.as_deref().unwrap_or("")
            ));
            s.push_str(&format!(
                "SystemLayoutHotkey={}\n",
                b.layout.as_deref().unwrap_or("")
            ));
            s.push_str(&format!(
                "SystemHotkey={}\n",
                b.legacy.as_deref().unwrap_or("")
            ));
        }
        let tmp = path.with_extension("ini.tmp");
        let mut bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        bytes.extend_from_slice(s.as_bytes());
        if std::fs::write(&tmp, &bytes).is_ok() {
            if std::fs::rename(&tmp, path).is_err() {
                let _ = std::fs::write(path, &bytes);
                let _ = std::fs::remove_file(&tmp);
            }
        }
    }
}
