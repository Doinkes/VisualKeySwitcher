//! On-screen language indicator (classic box, modern colored box, color bar).

use crate::app::{app_mut, App};
use crate::layout::SwitchOutcome;
use crate::win;

pub const TIMER_HIDE: usize = 1;

#[derive(Clone)]
pub struct OverlayState {
    pub tag: String,
    pub langid: u16,
    pub visual_type: u8,
    pub failed: bool,
    pub dpi: u32,
}

impl Default for OverlayState {
    fn default() -> Self {
        Self {
            tag: String::new(),
            langid: 0,
            visual_type: 1,
            failed: false,
            dpi: 96,
        }
    }
}

pub fn lang_color(langid: u16) -> u32 {
    match langid {
        0x0419 => win::rgb(255, 107, 53), // ru-RU, orange (as in the old version)
        0x0409 => win::rgb(74, 144, 217), // en-US, blue
        _ => win::rgb(110, 110, 120),
    }
}

pub unsafe fn register(hinst: win::HINSTANCE) -> bool {
    let class = win::wide("VKS_OverlayWnd");
    let wc = win::WNDCLASSEXW {
        cbSize: std::mem::size_of::<win::WNDCLASSEXW>() as win::UINT,
        style: win::CS_HREDRAW | win::CS_VREDRAW,
        lpfnWndProc: Some(proc_),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: hinst,
        hIcon: std::ptr::null_mut(),
        hCursor: std::ptr::null_mut(),
        hbrBackground: std::ptr::null_mut(),
        lpszMenuName: std::ptr::null(),
        lpszClassName: class.as_ptr(),
        hIconSm: std::ptr::null_mut(),
    };
    win::RegisterClassExW(&wc) != 0
}

pub unsafe fn create(hinst: win::HINSTANCE) -> win::HWND {
    let class = win::wide("VKS_OverlayWnd");
    let title = win::wide("");
    let hwnd = win::CreateWindowExW(
        win::WS_EX_TOPMOST
            | win::WS_EX_TOOLWINDOW
            | win::WS_EX_NOACTIVATE
            | win::WS_EX_TRANSPARENT
            | win::WS_EX_LAYERED,
        class.as_ptr(),
        title.as_ptr(),
        win::WS_POPUP,
        0,
        0,
        10,
        10,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        hinst,
        std::ptr::null_mut(),
    );
    if !hwnd.is_null() {
        win::SetLayeredWindowAttributes(hwnd, 0, 255, win::LWA_ALPHA);
    }
    hwnd
}

fn fallback_work_area() -> (win::RECT, u32) {
    unsafe {
        let w = win::GetSystemMetrics(win::SM_CXSCREEN).max(640);
        let h = win::GetSystemMetrics(win::SM_CYSCREEN).max(480);
        (
            win::RECT {
                left: 0,
                top: 0,
                right: w,
                bottom: h,
            },
            win::GetDpiForSystem().max(96),
        )
    }
}

fn place(visual_type: u8, work: win::RECT, dpi: u32) -> (i32, i32, i32, i32) {
    let s = |v: i32| win::scale(v, dpi);
    match visual_type {
        0 => {
            let w = s(200);
            let h = s(200);
            let x = work.left + (work.width() - w) / 2;
            let y = work.top + (work.height() - h) / 2;
            (x, y, w, h)
        }
        1 => {
            let w = s(200);
            let h = s(200);
            let x = work.left + (work.width() - w) / 2;
            let y = work.top + work.height() * 3 / 4 - h / 2;
            (x, y, w, h)
        }
        _ => {
            let margin = s(40);
            let w = (work.width() - margin * 2).max(s(80));
            let h = s(80);
            let x = work.left + margin;
            let y = work.top + work.height() * 3 / 4 - h / 2;
            (x, y, w, h)
        }
    }
}

pub fn show(app: &mut App, outcome: &SwitchOutcome) {
    show_impl(app, outcome, false);
}

/// Shows the indicator even if it is disabled in the settings (used by the
/// "Проверить" button).
pub fn show_preview(app: &mut App, outcome: &SwitchOutcome) {
    show_impl(app, outcome, true);
}

fn show_impl(app: &mut App, outcome: &SwitchOutcome, force: bool) {
    if outcome.no_op {
        return;
    }
    if (!force && !app.cfg.visual) || app.hwnd_overlay.is_null() {
        return;
    }

    let (work, dpi) = unsafe {
        if win::IsWindow(outcome.hwnd) != 0 {
            win::monitor_info_for(outcome.hwnd)
        } else {
            None
        }
    }
    .unwrap_or_else(fallback_work_area);

    let (x, y, w, h) = place(app.cfg.visual_type, work, dpi);

    app.overlay = OverlayState {
        tag: outcome.tag.clone(),
        langid: outcome.langid,
        visual_type: app.cfg.visual_type,
        failed: !outcome.success,
        dpi,
    };

    unsafe {
        let hwnd = app.hwnd_overlay;
        win::KillTimer(hwnd, TIMER_HIDE);
        win::SetWindowPos(
            hwnd,
            win::HWND_TOPMOST,
            x,
            y,
            w,
            h,
            win::SWP_NOACTIVATE | win::SWP_SHOWWINDOW,
        );
        win::SetLayeredWindowAttributes(hwnd, 0, 255, win::LWA_ALPHA);
        win::InvalidateRect(hwnd, std::ptr::null(), 1);
        win::UpdateWindow(hwnd);
        // Bring to the top without stealing focus.
        win::SetWindowPos(
            hwnd,
            win::HWND_TOPMOST,
            0,
            0,
            0,
            0,
            win::SWP_NOMOVE | win::SWP_NOSIZE | win::SWP_NOACTIVATE | win::SWP_SHOWWINDOW,
        );
        let duration = app.cfg.overlay_ms.clamp(150, 5000);
        win::SetTimer(hwnd, TIMER_HIDE, duration, None);
    }
}

unsafe fn paint(hwnd: win::HWND) {
    let app = app_mut();
    let mut ps: win::PAINTSTRUCT = std::mem::zeroed();
    let hdc = win::BeginPaint(hwnd, &mut ps);
    if hdc.is_null() {
        win::EndPaint(hwnd, &ps);
        return;
    }

    let mut rc: win::RECT = std::mem::zeroed();
    win::GetClientRect(hwnd, &mut rc);

    let state = app.overlay.clone();
    let bg = if state.visual_type == 0 {
        win::rgb(0, 0, 0)
    } else {
        lang_color(state.langid)
    };

    let bg_brush = win::CreateSolidBrush(bg);
    win::FillRect(hdc, &rc, bg_brush);
    win::DeleteObject(bg_brush);

    if state.visual_type != 2 {
        let face = win::wide("Segoe UI");
        let font = win::CreateFontW(
            -win::scale(112, state.dpi),
            0,
            0,
            0,
            win::FW_BOLD,
            0,
            0,
            0,
            win::DEFAULT_CHARSET,
            win::OUT_DEFAULT_PRECIS,
            win::CLIP_DEFAULT_PRECIS,
            win::CLEARTYPE_QUALITY,
            win::DEFAULT_PITCH | win::FF_DONTCARE,
            face.as_ptr(),
        );
        let old = win::SelectObject(hdc, font);
        win::SetBkMode(hdc, win::TRANSPARENT);
        win::SetTextColor(hdc, win::rgb(255, 255, 255));
        let mut text_rc = rc;
        let text = win::wide(&state.tag);
        win::DrawTextW(
            hdc,
            text.as_ptr(),
            -1,
            &mut text_rc,
            win::DT_CENTER | win::DT_VCENTER | win::DT_SINGLELINE | win::DT_NOPREFIX,
        );
        win::SelectObject(hdc, old);
        win::DeleteObject(font);
    }

    if state.failed {
        let red = win::CreateSolidBrush(win::rgb(225, 45, 45));
        let thickness = win::scale(4, state.dpi).max(2);
        for i in 0..thickness {
            let r = win::RECT {
                left: i,
                top: i,
                right: rc.right - i,
                bottom: rc.bottom - i,
            };
            win::FrameRect(hdc, &r, red);
        }
        win::DeleteObject(red);
    }

    win::EndPaint(hwnd, &ps);
}

pub unsafe extern "system" fn proc_(
    hwnd: win::HWND,
    msg: win::UINT,
    wp: win::WPARAM,
    lp: win::LPARAM,
) -> win::LRESULT {
    match msg {
        win::WM_PAINT => {
            paint(hwnd);
            0
        }
        win::WM_ERASEBKGND => 1,
        win::WM_NCHITTEST => -1, // HTTRANSPARENT
        win::WM_MOUSEACTIVATE => win::MA_NOACTIVATE,
        win::WM_TIMER => {
            if wp == TIMER_HIDE {
                win::KillTimer(hwnd, TIMER_HIDE);
                win::ShowWindow(hwnd, win::SW_HIDE);
            }
            0
        }
        win::WM_DESTROY => 0,
        _ => win::DefWindowProcW(hwnd, msg, wp, lp),
    }
}
