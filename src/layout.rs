//! Layout discovery and verified, best-effort switching of the foreground
//! window's input language.

use std::time::{Duration, Instant};

use crate::config::Action;
use crate::util;
use crate::win;

#[derive(Clone, Debug)]
pub struct SwitchRequest {
    pub hwnd: win::HWND,
    pub tid: u32,
    pub initial_hkl: win::HKL,
    pub action: Action,
    pub block_native: bool,
    /// Language shown immediately by the optimistic preview (0 = none).
    pub preview_langid: u16,
}

// The raw Win32 handles are opaque values here; requests are sent from the UI
// thread to exactly one worker thread and outcomes travel back the same way.
unsafe impl Send for SwitchRequest {}

#[derive(Debug)]
pub struct SwitchOutcome {
    pub success: bool,
    pub no_op: bool,
    pub initial_hkl: win::HKL,
    pub target_hkl: win::HKL,
    pub hwnd: win::HWND,
    pub attempts: u32,
    pub elapsed_ms: u64,
    pub class: String,
    pub langid: u16,
    pub tag: String,
    pub preview_langid: u16,
}

unsafe impl Send for SwitchOutcome {}

/// What the optimistic preview should show.
pub enum Preview {
    /// A concrete target layout.
    Target(win::HKL),
    /// A specific language was requested but is not installed.
    Missing,
    /// Nothing to switch (e.g. only one layout installed).
    Nothing,
}

/// Predicts the target layout without switching. Used to paint the indicator
/// and play the sound at the exact moment the hotkey fires.
pub fn preview_target(req: &SwitchRequest) -> Preview {
    unsafe {
        let list = win::layouts();
        match &req.action {
            Action::Next => match next_layout(&list, req.initial_hkl) {
                Some(h) => Preview::Target(h),
                None => Preview::Nothing,
            },
            Action::Lang(iso) => match find_iso(&list, iso) {
                Some(h) => Preview::Target(h),
                None => Preview::Missing,
            },
        }
    }
}

pub fn same_layout(a: win::HKL, b: win::HKL) -> bool {
    win::language_id(a) == win::language_id(b)
}

fn next_layout(list: &[win::HKL], current: win::HKL) -> Option<win::HKL> {
    if list.len() < 2 {
        return None;
    }
    let cur_lang = win::language_id(current);
    let has_other = list.iter().any(|h| win::language_id(*h) != cur_lang);
    if !has_other {
        return None;
    }
    let start = list
        .iter()
        .position(|h| win::language_id(*h) == cur_lang)
        .unwrap_or(0);
    for off in 0..list.len() {
        let cand = list[(start + off) % list.len()];
        if win::language_id(cand) != cur_lang {
            return Some(cand);
        }
    }
    None
}

fn find_iso(list: &[win::HKL], iso: &str) -> Option<win::HKL> {
    let iso = iso.to_lowercase();
    list.iter()
        .copied()
        .find(|h| unsafe { win::lang_iso(win::language_id(*h)) } == iso)
}

fn choose_target(action: &Action, list: &[win::HKL], from: win::HKL) -> Option<win::HKL> {
    match action {
        Action::Next => next_layout(list, from),
        Action::Lang(iso) => find_iso(list, iso),
    }
}

unsafe fn focus_window(req: &SwitchRequest) -> win::HWND {
    let mut gti: win::GUITHREADINFO = std::mem::zeroed();
    gti.cbSize = std::mem::size_of::<win::GUITHREADINFO>() as win::DWORD;
    if win::GetGUIThreadInfo(req.tid, &mut gti) != 0 && !gti.hwndFocus.is_null() {
        gti.hwndFocus
    } else {
        req.hwnd
    }
}

unsafe fn post_request(hwnd: win::HWND, target: win::HKL, wparam: win::WPARAM) -> bool {
    if hwnd.is_null() || win::IsWindow(hwnd) == 0 {
        return false;
    }
    win::PostMessageW(
        hwnd,
        win::WM_INPUTLANGCHANGEREQUEST,
        wparam,
        target as win::LPARAM,
    ) != 0
}

unsafe fn wait_for(tid: u32, target: win::HKL, timeout_ms: u64) -> bool {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if same_layout(win::GetKeyboardLayout(tid), target) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        win::Sleep(6);
    }
}

unsafe fn try_switch(req: &SwitchRequest, target: win::HKL) -> (bool, u32) {
    let mut attempts = 0;

    // 1. The canonical method Windows itself uses: ask the focused window.
    attempts += 1;
    let focus = focus_window(req);
    if post_request(focus, target, 0) && wait_for(req.tid, target, 260) {
        return (true, attempts);
    }

    // 2. Try the top-level foreground window as well.
    attempts += 1;
    if focus != req.hwnd && post_request(req.hwnd, target, 0) && wait_for(req.tid, target, 220) {
        return (true, attempts);
    }

    // 3. Last resort for applications that ignore the request: attach to the
    //    target input queue and activate the layout for that thread.
    attempts += 1;
    let me = win::GetCurrentThreadId();
    let attached = win::AttachThreadInput(me, req.tid, 1) != 0;
    win::ActivateKeyboardLayout(target, 0);
    if attached {
        win::AttachThreadInput(me, req.tid, 0);
    }
    if wait_for(req.tid, target, 200) {
        return (true, attempts);
    }

    // 4. Some apps only accept the request with the documented wParam bit set.
    attempts += 1;
    if post_request(focus, target, 1) && wait_for(req.tid, target, 180) {
        return (true, attempts);
    }

    (false, attempts)
}

pub fn perform_switch(req: &SwitchRequest) -> SwitchOutcome {
    let started = Instant::now();
    unsafe {
        let list = win::layouts();
        let current = win::GetKeyboardLayout(req.tid);
        let from = if req.block_native {
            current
        } else {
            req.initial_hkl
        };
        let target = choose_target(&req.action, &list, from);

        let mut success = false;
        let mut no_op = false;
        let mut attempts = 0u32;
        let mut final_hkl = current;
        let mut target_hkl: win::HKL = std::ptr::null_mut();

        match target {
            None => {
                if matches!(&req.action, Action::Next) && list.len() < 2 {
                    no_op = true;
                }
                util::debug(&format!(
                    "Не найдена целевая раскладка: list={:?} action={:?}",
                    list.iter()
                        .map(|h| format!("{:04X}", win::language_id(*h)))
                        .collect::<Vec<_>>(),
                    req.action
                ));
            }
            Some(target_hkl_v) => {
                target_hkl = target_hkl_v;
                if same_layout(current, target_hkl) {
                    success = true;
                    final_hkl = current;
                } else {
                    let (ok, n) = try_switch(req, target_hkl);
                    attempts = n;
                    if ok {
                        // Let a competing (native) switch settle, then make sure
                        // the user really ends up on the requested layout.
                        win::Sleep(130);
                        let settled = win::GetKeyboardLayout(req.tid);
                        if same_layout(settled, target_hkl) {
                            success = true;
                            final_hkl = settled;
                        } else {
                            util::debug("Раскладка изменилась после переключения, корректирую");
                            attempts += 1;
                            let focus = focus_window(req);
                            let _ = post_request(focus, target_hkl, 0);
                            success = wait_for(req.tid, target_hkl, 260);
                            final_hkl = win::GetKeyboardLayout(req.tid);
                        }
                    } else {
                        final_hkl = win::GetKeyboardLayout(req.tid);
                    }
                }
            }
        }

        let langid = win::language_id(final_hkl);
        let iso = win::lang_iso(langid);
        let tag = util::iso_tag(&iso);
        let class = win::window_class(req.hwnd);
        let elapsed_ms = started.elapsed().as_millis() as u64;

        SwitchOutcome {
            success,
            no_op,
            initial_hkl: req.initial_hkl,
            target_hkl,
            hwnd: req.hwnd,
            attempts,
            elapsed_ms,
            class,
            langid,
            tag,
            preview_langid: req.preview_langid,
        }
    }
}
