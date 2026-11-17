//! System integration: Windows input-language hotkey toggle and autostart.

use std::ffi::c_void;

use crate::util;
use crate::win::{self, HKEY};

const TOGGLE_KEY: &str = "Keyboard Layout\\Toggle";
const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "VisualKeySwitcher";

const TOGGLE_LANGUAGE: &str = "Language Hotkey";
const TOGGLE_LAYOUT: &str = "Layout Hotkey";
const TOGGLE_LEGACY: &str = "Hotkey";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToggleBackup {
    pub language: Option<String>,
    pub layout: Option<String>,
    pub legacy: Option<String>,
}

impl ToggleBackup {
    pub fn is_disabled_by_us(&self) -> bool {
        self.language.as_deref() == Some("3")
            && self.layout.as_deref() == Some("3")
            && self.legacy.as_deref() == Some("3")
    }
}

unsafe fn open_key(sub: &str, sam: u32) -> Option<HKEY> {
    let sub_w = win::wide(sub);
    let mut key: HKEY = std::ptr::null_mut();
    let rc = win::RegOpenKeyExW(win::HKEY_CURRENT_USER, sub_w.as_ptr(), 0, sam, &mut key);
    if rc == win::ERROR_SUCCESS as i32 && !key.is_null() {
        Some(key)
    } else {
        None
    }
}

unsafe fn get_string(key: HKEY, name: &str) -> Option<String> {
    let name_w = win::wide(name);
    let mut ty: u32 = 0;
    let mut cb: u32 = 0;
    let rc = win::RegQueryValueExW(
        key,
        name_w.as_ptr(),
        std::ptr::null_mut(),
        &mut ty,
        std::ptr::null_mut(),
        &mut cb,
    );
    if rc != win::ERROR_SUCCESS as i32 || cb == 0 {
        return None;
    }
    let mut buf = vec![0u8; cb as usize];
    let mut cb2 = cb;
    let rc = win::RegQueryValueExW(
        key,
        name_w.as_ptr(),
        std::ptr::null_mut(),
        &mut ty,
        buf.as_mut_ptr(),
        &mut cb2,
    );
    if rc != win::ERROR_SUCCESS as i32 {
        return None;
    }
    buf.truncate(cb2 as usize);
    let words: Vec<u16> = buf
        .chunks_exact(2)
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    Some(win::from_wide(&words))
}

unsafe fn get_dword(key: HKEY, name: &str) -> Option<u32> {
    let name_w = win::wide(name);
    let mut ty: u32 = 0;
    let mut value: u32 = 0;
    let mut cb: u32 = 4;
    let rc = win::RegQueryValueExW(
        key,
        name_w.as_ptr(),
        std::ptr::null_mut(),
        &mut ty,
        &mut value as *mut u32 as *mut u8,
        &mut cb,
    );
    if rc == win::ERROR_SUCCESS as i32 && ty == win::REG_DWORD {
        Some(value)
    } else {
        None
    }
}

unsafe fn set_string(key: HKEY, name: &str, value: &str) -> bool {
    let name_w = win::wide(name);
    let data = win::wide(value);
    let rc = win::RegSetValueExW(
        key,
        name_w.as_ptr(),
        0,
        win::REG_SZ,
        data.as_ptr() as *const u8,
        (data.len() * 2) as u32,
    );
    rc == win::ERROR_SUCCESS as i32
}

unsafe fn delete_value(key: HKEY, name: &str) -> bool {
    let name_w = win::wide(name);
    let rc = win::RegDeleteValueW(key, name_w.as_ptr());
    rc == win::ERROR_SUCCESS as i32 || rc == win::ERROR_FILE_NOT_FOUND as i32
}

unsafe fn create_key(sub: &str, sam: u32) -> Option<HKEY> {
    let sub_w = win::wide(sub);
    let mut key: HKEY = std::ptr::null_mut();
    let mut disp: u32 = 0;
    let rc = win::RegCreateKeyExW(
        win::HKEY_CURRENT_USER,
        sub_w.as_ptr(),
        0,
        std::ptr::null(),
        0,
        sam,
        std::ptr::null::<c_void>(),
        &mut key,
        &mut disp,
    );
    if rc == win::ERROR_SUCCESS as i32 {
        Some(key)
    } else {
        None
    }
}

// ------------------------------------------------------------ toggle

pub fn read_toggle() -> ToggleBackup {
    unsafe {
        let Some(key) = open_key(TOGGLE_KEY, win::KEY_READ) else {
            return ToggleBackup::default();
        };
        let b = ToggleBackup {
            language: get_string(key, TOGGLE_LANGUAGE),
            layout: get_string(key, TOGGLE_LAYOUT),
            legacy: get_string(key, TOGGLE_LEGACY),
        };
        win::RegCloseKey(key);
        b
    }
}

/// Writes the three toggle values and asks Windows to re-read them immediately.
fn apply_toggle_values(backup: &ToggleBackup) -> bool {
    unsafe {
        let Some(key) = create_key(TOGGLE_KEY, win::KEY_SET_VALUE) else {
            return false;
        };
        let mut ok = true;
        ok &= match &backup.language {
            Some(v) => set_string(key, TOGGLE_LANGUAGE, v),
            None => delete_value(key, TOGGLE_LANGUAGE),
        };
        ok &= match &backup.layout {
            Some(v) => set_string(key, TOGGLE_LAYOUT, v),
            None => delete_value(key, TOGGLE_LAYOUT),
        };
        ok &= match &backup.legacy {
            Some(v) => set_string(key, TOGGLE_LEGACY, v),
            None => delete_value(key, TOGGLE_LEGACY),
        };
        win::RegCloseKey(key);
        // Documented API: Windows re-reads HKCU\Keyboard Layout\Toggle.
        let spi = win::SystemParametersInfoW(win::SPI_SETLANGTOGGLE, 0, std::ptr::null_mut(), 0);
        if spi == 0 {
            util::log("SPI_SETLANGTOGGLE вернул ошибку");
        }
        ok
    }
}

/// Disables the built-in layout switching hotkeys. Returns whether the
/// registry values were actually written and verified.
pub fn disable_native_toggle() -> bool {
    let none = ToggleBackup {
        language: Some("3".into()),
        layout: Some("3".into()),
        legacy: Some("3".into()),
    };
    if !apply_toggle_values(&none) {
        util::log("Не удалось записать HKCU\\Keyboard Layout\\Toggle");
        return false;
    }
    let check = read_toggle();
    let ok = check.is_disabled_by_us();
    if !ok {
        util::log("Значения Toggle не подтвердились после записи");
    }
    ok
}

pub fn restore_native_toggle(backup: &ToggleBackup) -> bool {
    if !apply_toggle_values(backup) {
        util::log("Не удалось восстановить HKCU\\Keyboard Layout\\Toggle");
        return false;
    }
    let check = read_toggle();
    let ok = check == *backup;
    if !ok {
        util::log(&format!(
            "Восстановление Toggle не подтвердилось: прочитано {:?}",
            check
        ));
    }
    ok
}

// ------------------------------------------------------------ autostart

pub fn exe_path() -> String {
    unsafe {
        let mut buf = [0u16; 32768];
        let n = win::GetModuleFileNameW(std::ptr::null_mut(), buf.as_mut_ptr(), buf.len() as u32);
        if n == 0 {
            return String::new();
        }
        win::from_wide(&buf[..n as usize])
    }
}

pub fn autostart_enabled() -> bool {
    unsafe {
        let Some(key) = open_key(RUN_KEY, win::KEY_READ) else {
            return false;
        };
        let value = get_string(key, RUN_VALUE);
        win::RegCloseKey(key);
        value.map(|v| !v.trim().is_empty()).unwrap_or(false)
    }
}

pub fn set_autostart(enabled: bool) -> bool {
    unsafe {
        let Some(key) = create_key(RUN_KEY, win::KEY_SET_VALUE) else {
            return false;
        };
        let ok = if enabled {
            let exe = exe_path();
            set_string(key, RUN_VALUE, &format!("\"{}\"", exe))
        } else {
            delete_value(key, RUN_VALUE)
        };
        win::RegCloseKey(key);
        ok
    }
}

pub fn refresh_autostart_path() {
    if !autostart_enabled() {
        return;
    }
    unsafe {
        let Some(key) = open_key(RUN_KEY, win::KEY_READ) else {
            return;
        };
        let current = get_string(key, RUN_VALUE);
        win::RegCloseKey(key);
        let desired = format!("\"{}\"", exe_path());
        if current.as_deref() != Some(desired.as_str()) {
            set_autostart(true);
        }
    }
}

// ------------------------------------------------------------ migration

#[derive(Default)]
pub struct LegacySettings {
    pub exists: bool,
    pub visual_marker: Option<u32>,
    pub sound: Option<u32>,
    pub hotkey_mods: Option<u32>,
    pub visual_type: Option<u32>,
    pub sound_pack: Option<String>,
}

pub fn read_legacy_settings() -> LegacySettings {
    unsafe {
        let Some(key) = open_key("Software\\VisualKeySwitcher", win::KEY_READ) else {
            return LegacySettings::default();
        };
        let s = LegacySettings {
            exists: true,
            visual_marker: get_dword(key, "VisualMarker"),
            sound: get_dword(key, "Sound"),
            hotkey_mods: get_dword(key, "Hotkey"),
            visual_type: get_dword(key, "VisualType"),
            sound_pack: get_string(key, "SoundPack"),
        };
        win::RegCloseKey(key);
        s
    }
}
