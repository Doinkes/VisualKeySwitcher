//! Small utilities: logging, paths, hotkey text, modifier flags.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::win;

pub const MOD_CTRL: u8 = 1 << 0;
pub const MOD_SHIFT: u8 = 1 << 1;
pub const MOD_ALT: u8 = 1 << 2;
pub const MOD_WIN: u8 = 1 << 3;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static DEBUG: AtomicBool = AtomicBool::new(false);

pub fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn data_dir() -> PathBuf {
    std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(exe_dir)
        .join("VisualKeySwitcher")
}

pub fn config_path() -> PathBuf {
    data_dir().join("config.ini")
}

pub fn log_path() -> PathBuf {
    data_dir().join("log.txt")
}

pub fn init_logging(path: PathBuf, debug: bool) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Keep the log bounded.
    if let Ok(meta) = std::fs::metadata(&path) {
        if meta.len() > 256 * 1024 {
            let _ = std::fs::remove_file(&path);
        }
    }
    let _ = LOG_PATH.set(path);
    DEBUG.store(debug, Ordering::Relaxed);
}

pub fn set_debug(enabled: bool) {
    DEBUG.store(enabled, Ordering::Relaxed);
}

fn timestamp() -> String {
    unsafe {
        let mut st: win::SYSTEMTIME = std::mem::zeroed();
        win::GetLocalTime(&mut st);
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
            st.wYear, st.wMonth, st.wDay, st.wHour, st.wMinute, st.wSecond, st.wMilliseconds
        )
    }
}

pub fn log(msg: &str) {
    let Some(path) = LOG_PATH.get() else { return };
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let empty = f.metadata().map(|m| m.len() == 0).unwrap_or(false);
        if empty {
            // UTF-8 BOM so Notepad and friends open Cyrillic correctly.
            let _ = f.write_all(&[0xEF, 0xBB, 0xBF]);
        }
        let _ = writeln!(f, "[{}] {}", timestamp(), msg);
    }
}

pub fn debug(msg: &str) {
    if DEBUG.load(Ordering::Relaxed) {
        log(msg);
    }
}

pub fn is_modifier(vk: u16) -> bool {
    matches!(
        vk,
        win::VK_SHIFT
            | win::VK_LSHIFT
            | win::VK_RSHIFT
            | win::VK_CONTROL
            | win::VK_LCONTROL
            | win::VK_RCONTROL
            | win::VK_MENU
            | win::VK_LMENU
            | win::VK_RMENU
            | win::VK_LWIN
            | win::VK_RWIN
    )
}

/// Modifier bit represented by a virtual key, if it is a modifier.
pub fn modifier_bit(vk: u16) -> Option<u8> {
    match vk {
        win::VK_SHIFT | win::VK_LSHIFT | win::VK_RSHIFT => Some(MOD_SHIFT),
        win::VK_CONTROL | win::VK_LCONTROL | win::VK_RCONTROL => Some(MOD_CTRL),
        win::VK_MENU | win::VK_LMENU | win::VK_RMENU => Some(MOD_ALT),
        win::VK_LWIN | win::VK_RWIN => Some(MOD_WIN),
        _ => None,
    }
}

/// Keys that are safe to use without any modifier.
pub fn lone_key_allowed(vk: u16) -> bool {
    matches!(
        vk,
        win::VK_CAPITAL
            | win::VK_SCROLL
            | win::VK_PAUSE
            | win::VK_SNAPSHOT
            | win::VK_NUMLOCK
            | win::VK_APPS
            | win::VK_INSERT
    ) || (vk >= win::VK_F1 && vk <= win::VK_F24)
}

pub fn vk_name(vk: u16) -> String {
    match vk {
        0 => String::new(),
        win::VK_BACK => "Backspace".into(),
        win::VK_TAB => "Tab".into(),
        win::VK_CLEAR => "Clear".into(),
        win::VK_RETURN => "Enter".into(),
        win::VK_PAUSE => "Pause".into(),
        win::VK_CAPITAL => "Caps Lock".into(),
        win::VK_ESCAPE => "Esc".into(),
        win::VK_SPACE => "Space".into(),
        win::VK_PRIOR => "Page Up".into(),
        win::VK_NEXT => "Page Down".into(),
        win::VK_END => "End".into(),
        win::VK_HOME => "Home".into(),
        win::VK_LEFT => "Left".into(),
        win::VK_UP => "Up".into(),
        win::VK_RIGHT => "Right".into(),
        win::VK_DOWN => "Down".into(),
        win::VK_SNAPSHOT => "Print Screen".into(),
        win::VK_INSERT => "Insert".into(),
        win::VK_DELETE => "Delete".into(),
        win::VK_LWIN | win::VK_RWIN => "Win".into(),
        win::VK_APPS => "Menu".into(),
        win::VK_SLEEP => "Sleep".into(),
        win::VK_NUMLOCK => "Num Lock".into(),
        win::VK_SCROLL => "Scroll Lock".into(),
        win::VK_MULTIPLY => "*".into(),
        win::VK_ADD => "+".into(),
        win::VK_SEPARATOR => "Sep".into(),
        win::VK_SUBTRACT => "-".into(),
        win::VK_DECIMAL => ".".into(),
        win::VK_DIVIDE => "/".into(),
        win::VK_OEM_1 => ";".into(),
        win::VK_OEM_PLUS => "=".into(),
        win::VK_OEM_COMMA => ",".into(),
        win::VK_OEM_MINUS => "-".into(),
        win::VK_OEM_PERIOD => ".".into(),
        win::VK_OEM_2 => "/".into(),
        win::VK_OEM_3 => "`".into(),
        win::VK_OEM_4 => "[".into(),
        win::VK_OEM_5 => "\\".into(),
        win::VK_OEM_6 => "]".into(),
        win::VK_OEM_7 => "'".into(),
        win::VK_OEM_8 => "OEM".into(),
        win::VK_OEM_102 => "<".into(),
        _ if vk >= win::VK_F1 && vk <= win::VK_F24 => format!("F{}", vk - win::VK_F1 + 1),
        _ if vk >= 0x41 && vk <= 0x5A => ((vk as u8) as char).to_string(),
        _ if vk >= 0x30 && vk <= 0x39 => ((vk as u8) as char).to_string(),
        _ if vk >= win::VK_NUMPAD0 && vk <= win::VK_NUMPAD9 => {
            format!("Num {}", vk - win::VK_NUMPAD0)
        }
        _ => format!("VK{:02X}", vk),
    }
}

pub fn format_mods(mods: u8) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if mods & MOD_CTRL != 0 {
        parts.push("Ctrl");
    }
    if mods & MOD_ALT != 0 {
        parts.push("Alt");
    }
    if mods & MOD_SHIFT != 0 {
        parts.push("Shift");
    }
    if mods & MOD_WIN != 0 {
        parts.push("Win");
    }
    parts.join(" + ")
}

pub fn format_hotkey(mods: u8, vk: u16) -> String {
    if vk == 0 {
        let m = format_mods(mods);
        if m.is_empty() {
            "не задано".to_string()
        } else {
            m
        }
    } else {
        let m = format_mods(mods);
        if m.is_empty() {
            vk_name(vk)
        } else {
            format!("{} + {}", m, vk_name(vk))
        }
    }
}

/// Mask used for matching the currently pressed modifier keys.
pub fn mods_from_state(ctrl: bool, shift: bool, alt: bool, win: bool) -> u8 {
    (if ctrl { MOD_CTRL } else { 0 })
        | (if shift { MOD_SHIFT } else { 0 })
        | (if alt { MOD_ALT } else { 0 })
        | (if win { MOD_WIN } else { 0 })
}

pub fn iso_tag(iso: &str) -> String {
    if iso.is_empty() {
        return "??".into();
    }
    iso.chars().take(3).collect::<String>().to_uppercase()
}
