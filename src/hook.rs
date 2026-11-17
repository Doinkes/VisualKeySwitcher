//! Low-level keyboard hook: custom hotkey detection and hotkey capture.
//!
//! The hook callback does only bookkeeping and (when needed) posts a message
//! to the UI thread. No sleeps, file I/O or layout switching happen here.

use std::cell::RefCell;
use std::sync::atomic::{AtomicIsize, Ordering};

use crate::util::{self, MOD_ALT, MOD_CTRL, MOD_SHIFT, MOD_WIN};
use crate::win;

static HOOK_HANDLE: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone, Copy, Debug)]
pub struct HookConfig {
    pub mods: u8,
    pub vk: u16,
    pub block_native: bool,
    /// `true` — switch when the chord is pressed, `false` — when released.
    pub trigger_on_press: bool,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            mods: MOD_CTRL | MOD_SHIFT,
            vk: 0,
            block_native: true,
            trigger_on_press: true,
        }
    }
}

pub enum HookResult {
    Pass,
    Swallow,
    PassSwitch,
    SwallowSwitch,
    CaptureComplete { mods: u8, vk: u16 },
    CaptureCancel,
    CaptureInvalid(&'static str),
    CapturePreview(String),
}

struct HookState {
    ctrl: bool,
    shift: bool,
    alt: bool,
    win: bool,
    /// No non-modifier key was pressed since the current hold started.
    mods_clean: bool,
    /// The configured modifier-only chord is fully held.
    armed: bool,
    /// Already fired for the current hold.
    triggered: bool,
    /// Key currently suppressed together with its key-up.
    suppress_vk: u16,
    space_blocked: bool,
    cfg: HookConfig,
    capturing: bool,
    capture_had_mod: bool,
    capture_last_mods: u8,
}

impl HookState {
    fn new() -> Self {
        Self {
            ctrl: false,
            shift: false,
            alt: false,
            win: false,
            mods_clean: true,
            armed: false,
            triggered: false,
            suppress_vk: 0,
            space_blocked: false,
            cfg: HookConfig::default(),
            capturing: false,
            capture_had_mod: false,
            capture_last_mods: 0,
        }
    }

    fn mods(&self) -> u8 {
        util::mods_from_state(self.ctrl, self.shift, self.alt, self.win)
    }

    fn set_mod(&mut self, vk: u16, down: bool) {
        match util::modifier_bit(vk) {
            Some(MOD_CTRL) => self.ctrl = down,
            Some(MOD_SHIFT) => self.shift = down,
            Some(MOD_ALT) => self.alt = down,
            Some(MOD_WIN) => self.win = down,
            _ => {}
        }
    }

    fn reset_hold(&mut self) {
        self.mods_clean = true;
        self.armed = false;
        self.triggered = false;
        self.space_blocked = false;
    }

    fn end_capture(&mut self) {
        self.capturing = false;
        self.capture_had_mod = false;
        self.capture_last_mods = 0;
        self.suppress_vk = 0;
        self.reset_hold();
    }

    fn process(&mut self, vk: u16, down: bool, injected: bool, is_mod: bool) -> HookResult {
        if is_mod {
            self.set_mod(vk, down);
        }

        if self.capturing {
            return self.process_capture(vk, down, injected, is_mod);
        }

        let mods = self.mods();

        // Beginning of a fresh modifier-only hold.
        if is_mod && down && mods == util::modifier_bit(vk).unwrap_or(0) {
            self.mods_clean = true;
            self.armed = false;
            self.triggered = false;
            self.suppress_vk = 0;
        }

        if injected {
            return HookResult::Pass;
        }

        // --- hotkey with an explicit trigger key -------------------------
        if !is_mod && self.cfg.vk != 0 && vk == self.cfg.vk {
            if down {
                if self.suppress_vk == vk {
                    return HookResult::Swallow;
                }
                if mods == self.cfg.mods {
                    self.suppress_vk = vk;
                    if self.cfg.trigger_on_press {
                        return HookResult::SwallowSwitch;
                    }
                    return HookResult::Swallow;
                }
            } else if self.suppress_vk == vk {
                self.suppress_vk = 0;
                if self.cfg.trigger_on_press {
                    return HookResult::Swallow;
                }
                return HookResult::SwallowSwitch;
            }
        }

        // --- modifier-only hotkey ----------------------------------------
        if self.cfg.vk == 0 {
            if is_mod && down {
                if mods == self.cfg.mods && self.mods_clean && !self.triggered {
                    if self.cfg.trigger_on_press {
                        self.armed = false;
                        self.triggered = true;
                        return HookResult::PassSwitch;
                    }
                    self.armed = true;
                } else if (mods & !self.cfg.mods) != 0 || !self.mods_clean {
                    self.armed = false;
                }
            } else if is_mod && !down && !self.cfg.trigger_on_press {
                if self.armed && !self.triggered && self.mods_clean && !mouse_buttons_down() {
                    self.armed = false;
                    self.triggered = true;
                    return HookResult::PassSwitch;
                }
            }
        }

        // Any non-modifier key cancels a pending modifier-only trigger.
        if !is_mod && down {
            self.mods_clean = false;
            self.armed = false;
        }

        // Win+Space is handled by the shell and cannot be disabled through
        // HKCU\Keyboard Layout\Toggle, so swallow it ourselves.
        if self.cfg.block_native && !is_mod && vk == win::VK_SPACE {
            if down {
                if (mods & MOD_WIN) != 0 && (mods & (MOD_CTRL | MOD_ALT)) == 0 {
                    self.space_blocked = true;
                    return HookResult::Swallow;
                }
            } else if self.space_blocked {
                self.space_blocked = false;
                return HookResult::Swallow;
            }
        }

        if is_mod && !down && self.mods() == 0 {
            self.reset_hold();
        }

        HookResult::Pass
    }

    fn process_capture(&mut self, vk: u16, down: bool, injected: bool, is_mod: bool) -> HookResult {
        if injected {
            return HookResult::Swallow;
        }

        if is_mod {
            if down {
                self.capture_had_mod = true;
                self.capture_last_mods = self.mods();
                return HookResult::CapturePreview(util::format_hotkey(self.capture_last_mods, 0));
            }
            if self.mods() == 0 {
                if self.capture_had_mod {
                    let mods = self.capture_last_mods;
                    self.end_capture();
                    return match validate_capture(mods, 0) {
                        Ok(()) => HookResult::CaptureComplete { mods, vk: 0 },
                        Err(e) => HookResult::CaptureInvalid(e),
                    };
                }
                return HookResult::Swallow;
            }
            return HookResult::CapturePreview(util::format_hotkey(self.mods(), 0));
        }

        if down {
            if vk == win::VK_ESCAPE {
                self.end_capture();
                return HookResult::CaptureCancel;
            }
            let mods = self.mods();
            self.end_capture();
            return match validate_capture(mods, vk) {
                Ok(()) => HookResult::CaptureComplete { mods, vk },
                Err(e) => HookResult::CaptureInvalid(e),
            };
        }

        HookResult::Swallow
    }
}

fn validate_capture(mods: u8, vk: u16) -> Result<(), &'static str> {
    if vk == 0 {
        if mods.count_ones() < 2 {
            return Err("Нужно минимум два модификатора, например Ctrl + Shift.");
        }
        if (mods & MOD_WIN) != 0 {
            return Err("Сочетание только из модификаторов не может содержать Win.");
        }
        return Ok(());
    }
    if util::is_modifier(vk) {
        return Err("Этот ключ является модификатором.");
    }
    if mods == 0 && !util::lone_key_allowed(vk) {
        return Err("Добавьте модификатор: Ctrl, Alt, Shift или Win.");
    }
    if (mods & MOD_WIN) != 0 && vk == 0x4C {
        // Win+L
        return Err("Win + L зарезервировано Windows (блокировка экрана).");
    }
    Ok(())
}

fn mouse_buttons_down() -> bool {
    unsafe {
        win::is_key_down(win::VK_LBUTTON as i32)
            || win::is_key_down(win::VK_RBUTTON as i32)
            || win::is_key_down(win::VK_MBUTTON as i32)
            || win::is_key_down(win::VK_XBUTTON1 as i32)
            || win::is_key_down(win::VK_XBUTTON2 as i32)
    }
}

thread_local! {
    static STATE: RefCell<HookState> = RefCell::new(HookState::new());
}

pub fn set_config(cfg: HookConfig) {
    STATE.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            s.cfg = cfg;
            s.suppress_vk = 0;
            s.space_blocked = false;
            s.reset_hold();
        }
    });
}

pub fn start_capture() {
    STATE.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            s.capturing = true;
            s.capture_had_mod = false;
            s.capture_last_mods = 0;
            s.suppress_vk = 0;
            s.space_blocked = false;
            s.reset_hold();
        }
    });
}

pub fn cancel_capture() {
    STATE.with(|s| {
        if let Ok(mut s) = s.try_borrow_mut() {
            s.end_capture();
        }
    });
}

pub fn is_capturing() -> bool {
    STATE.with(|s| s.try_borrow().map(|s| s.capturing).unwrap_or(false))
}

unsafe extern "system" fn keyboard_proc(
    code: i32,
    wp: win::WPARAM,
    lp: win::LPARAM,
) -> win::LRESULT {
    if code < 0 {
        return win::CallNextHookEx(std::ptr::null_mut(), code, wp, lp);
    }
    let kb = &*(lp as *const win::KBDLLHOOKSTRUCT);
    let vk = (kb.vkCode & 0xFFFF) as u16;
    let down = wp == win::WM_KEYDOWN as usize || wp == win::WM_SYSKEYDOWN as usize;
    let injected = (kb.flags & win::LLKHF_INJECTED) != 0;
    let is_mod = util::is_modifier(vk);

    let result = STATE.with(|s| match s.try_borrow_mut() {
        Ok(mut st) => st.process(vk, down, injected, is_mod),
        Err(_) => HookResult::Pass,
    });

    match result {
        HookResult::Pass => win::CallNextHookEx(std::ptr::null_mut(), code, wp, lp),
        HookResult::Swallow => 1,
        HookResult::PassSwitch => {
            crate::app::request_switch();
            win::CallNextHookEx(std::ptr::null_mut(), code, wp, lp)
        }
        HookResult::SwallowSwitch => {
            crate::app::request_switch();
            1
        }
        HookResult::CaptureComplete { mods, vk } => {
            crate::app::capture_complete(mods, vk);
            1
        }
        HookResult::CaptureCancel => {
            crate::app::capture_cancel();
            1
        }
        HookResult::CaptureInvalid(reason) => {
            crate::app::capture_invalid(reason);
            1
        }
        HookResult::CapturePreview(text) => {
            crate::app::capture_preview(&text);
            1
        }
    }
}

pub fn install() -> bool {
    unsafe {
        let h = win::SetWindowsHookExW(
            win::WH_KEYBOARD_LL,
            Some(keyboard_proc),
            win::GetModuleHandleW(std::ptr::null()),
            0,
        );
        if h.is_null() {
            return false;
        }
        HOOK_HANDLE.store(h as isize, Ordering::SeqCst);
        true
    }
}

pub fn uninstall() {
    let h = HOOK_HANDLE.swap(0, Ordering::SeqCst);
    if h != 0 {
        unsafe {
            win::UnhookWindowsHookEx(h as win::HHOOK);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(mods: u8, vk: u16) -> HookState {
        let mut s = HookState::new();
        s.cfg = HookConfig {
            mods,
            vk,
            block_native: true,
            trigger_on_press: true,
        };
        s
    }

    fn st_on_release(mods: u8, vk: u16) -> HookState {
        let mut s = st(mods, vk);
        s.cfg.trigger_on_press = false;
        s
    }

    fn ev(s: &mut HookState, vk: u16, down: bool) -> HookResult {
        s.process(vk, down, false, util::is_modifier(vk))
    }

    #[test]
    fn modifier_only_fires_on_release() {
        let mut s = st_on_release(MOD_CTRL | MOD_SHIFT, 0);
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, true),
            HookResult::Pass
        ));
        assert!(matches!(ev(&mut s, win::VK_SHIFT, true), HookResult::Pass));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, false),
            HookResult::PassSwitch
        ));
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, false),
            HookResult::Pass
        ));
        // A new hold must fire again.
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, true),
            HookResult::Pass
        ));
        assert!(matches!(ev(&mut s, win::VK_SHIFT, true), HookResult::Pass));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, false),
            HookResult::PassSwitch
        ));
    }

    #[test]
    fn modifier_plus_letter_does_not_fire() {
        let mut s = st_on_release(MOD_CTRL | MOD_SHIFT, 0);
        ev(&mut s, win::VK_CONTROL, true);
        ev(&mut s, win::VK_SHIFT, true);
        assert!(matches!(ev(&mut s, 0x41, true), HookResult::Pass)); // A
        assert!(matches!(ev(&mut s, 0x41, false), HookResult::Pass));
        assert!(matches!(ev(&mut s, win::VK_SHIFT, false), HookResult::Pass));
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, false),
            HookResult::Pass
        ));
    }

    #[test]
    fn extra_modifier_cancels() {
        let mut s = st_on_release(MOD_CTRL | MOD_SHIFT, 0);
        ev(&mut s, win::VK_CONTROL, true);
        ev(&mut s, win::VK_SHIFT, true);
        assert!(matches!(ev(&mut s, win::VK_MENU, true), HookResult::Pass));
        assert!(matches!(ev(&mut s, win::VK_MENU, false), HookResult::Pass));
        assert!(matches!(ev(&mut s, win::VK_SHIFT, false), HookResult::Pass));
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, false),
            HookResult::Pass
        ));
    }

    #[test]
    fn modifier_only_fires_on_press() {
        let mut s = st(MOD_CTRL | MOD_SHIFT, 0);
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, true),
            HookResult::Pass
        ));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, true),
            HookResult::PassSwitch
        ));
        // Holding the chord must not fire again.
        assert!(matches!(ev(&mut s, win::VK_SHIFT, true), HookResult::Pass));
        assert!(matches!(ev(&mut s, win::VK_SHIFT, false), HookResult::Pass));
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, false),
            HookResult::Pass
        ));
        // A fresh hold fires again.
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, true),
            HookResult::Pass
        ));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, true),
            HookResult::PassSwitch
        ));
    }

    #[test]
    fn explicit_hotkey_on_release() {
        let mut s = st_on_release(MOD_CTRL | MOD_ALT, 0x4C);
        ev(&mut s, win::VK_CONTROL, true);
        ev(&mut s, win::VK_MENU, true);
        assert!(matches!(ev(&mut s, 0x4C, true), HookResult::Swallow));
        assert!(matches!(ev(&mut s, 0x4C, false), HookResult::SwallowSwitch));
    }

    #[test]
    fn explicit_hotkey_is_swallowed() {
        let mut s = st(MOD_CTRL | MOD_ALT, 0x4C); // Ctrl+Alt+L
        ev(&mut s, win::VK_CONTROL, true);
        ev(&mut s, win::VK_MENU, true);
        assert!(matches!(ev(&mut s, 0x4C, true), HookResult::SwallowSwitch));
        // Key auto-repeat must not switch again.
        assert!(matches!(ev(&mut s, 0x4C, true), HookResult::Swallow));
        assert!(matches!(ev(&mut s, 0x4C, false), HookResult::Swallow));
    }

    #[test]
    fn explicit_hotkey_requires_exact_modifiers() {
        let mut s = st(MOD_CTRL | MOD_ALT, 0x4C);
        ev(&mut s, win::VK_CONTROL, true);
        ev(&mut s, win::VK_SHIFT, true);
        assert!(matches!(ev(&mut s, 0x4C, true), HookResult::Pass));
        ev(&mut s, 0x4C, false);
        ev(&mut s, win::VK_SHIFT, false);
        ev(&mut s, win::VK_CONTROL, false);
    }

    #[test]
    fn win_space_is_blocked_when_requested() {
        let mut s = st(MOD_CTRL | MOD_SHIFT, 0);
        ev(&mut s, win::VK_LWIN, true);
        assert!(matches!(
            ev(&mut s, win::VK_SPACE, true),
            HookResult::Swallow
        ));
        assert!(matches!(
            ev(&mut s, win::VK_SPACE, false),
            HookResult::Swallow
        ));
        ev(&mut s, win::VK_LWIN, false);

        let mut s2 = st(MOD_CTRL | MOD_SHIFT, 0);
        s2.cfg.block_native = false;
        ev(&mut s2, win::VK_LWIN, true);
        assert!(matches!(ev(&mut s2, win::VK_SPACE, true), HookResult::Pass));
        ev(&mut s2, win::VK_SPACE, false);
        ev(&mut s2, win::VK_LWIN, false);
    }

    #[test]
    fn capture_modifier_only() {
        let mut s = st(MOD_CTRL | MOD_SHIFT, 0);
        s.capturing = true;
        assert!(matches!(
            ev(&mut s, win::VK_CONTROL, true),
            HookResult::CapturePreview(_)
        ));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, true),
            HookResult::CapturePreview(_)
        ));
        assert!(matches!(
            ev(&mut s, win::VK_SHIFT, false),
            HookResult::CapturePreview(_)
        ));
        match ev(&mut s, win::VK_CONTROL, false) {
            HookResult::CaptureComplete { mods, vk } => {
                assert_eq!(mods, MOD_CTRL | MOD_SHIFT);
                assert_eq!(vk, 0);
            }
            _ => panic!("expected CaptureComplete"),
        }
        assert!(!s.capturing);
    }

    #[test]
    fn capture_with_key() {
        let mut s = st(MOD_CTRL | MOD_SHIFT, 0);
        s.capturing = true;
        ev(&mut s, win::VK_CONTROL, true);
        match ev(&mut s, 0x4B, true) {
            HookResult::CaptureComplete { mods, vk } => {
                assert_eq!(mods, MOD_CTRL);
                assert_eq!(vk, 0x4B);
            }
            _ => panic!("expected CaptureComplete"),
        }
        assert!(!s.capturing);
    }

    #[test]
    fn capture_rejects_plain_letter_and_esc_cancels() {
        let mut s = st(MOD_CTRL | MOD_SHIFT, 0);
        s.capturing = true;
        assert!(matches!(
            ev(&mut s, 0x41, true),
            HookResult::CaptureInvalid(_)
        ));
        assert!(!s.capturing);

        s.capturing = true;
        assert!(matches!(
            ev(&mut s, win::VK_ESCAPE, true),
            HookResult::CaptureCancel
        ));
        assert!(!s.capturing);
    }
}
