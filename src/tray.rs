//! Tray icon: lifecycle and context menu.

use crate::config::Config;
use crate::util;
use crate::win;

pub const CMD_SETTINGS: usize = 100;
pub const CMD_TOGGLE_VISUAL: usize = 101;
pub const CMD_TOGGLE_SOUND: usize = 102;
pub const CMD_TOGGLE_BLOCK: usize = 103;
pub const CMD_EXIT: usize = 104;

pub struct Tray {
    nid: win::NOTIFYICONDATAW,
    added: bool,
}

impl Tray {
    pub fn new(hwnd: win::HWND, icon: win::HICON, tip: &str) -> Self {
        unsafe {
            let mut nid: win::NOTIFYICONDATAW = std::mem::zeroed();
            nid.cbSize = std::mem::size_of::<win::NOTIFYICONDATAW>() as win::DWORD;
            nid.hWnd = hwnd;
            nid.uID = 1;
            nid.uFlags = win::NIF_MESSAGE | win::NIF_ICON | win::NIF_TIP;
            nid.uCallbackMessage = crate::app::WM_TRAY;
            nid.hIcon = icon;
            copy_wide(&mut nid.szTip, tip);
            Self { nid, added: false }
        }
    }

    pub fn add(&mut self) -> bool {
        unsafe {
            if self.added {
                return true;
            }
            self.added = win::Shell_NotifyIconW(win::NIM_ADD, &mut self.nid) != 0;
            if !self.added {
                util::log(&format!(
                    "Не удалось добавить иконку в трей: {}",
                    win::error_text(win::last_error())
                ));
            }
            self.added
        }
    }

    /// Re-adds the icon (e.g. after Explorer restarted).
    pub fn readd(&mut self) {
        self.added = false;
        self.add();
    }

    pub fn set_tip(&mut self, tip: &str) {
        if !self.added {
            return;
        }
        unsafe {
            self.nid.uFlags = win::NIF_TIP;
            copy_wide(&mut self.nid.szTip, tip);
            let _ = win::Shell_NotifyIconW(win::NIM_MODIFY, &mut self.nid);
        }
    }

    pub fn balloon(&mut self, title: &str, text: &str, flags: win::DWORD) {
        if !self.added {
            return;
        }
        unsafe {
            self.nid.uFlags = win::NIF_INFO | win::NIF_TIP;
            self.nid.dwInfoFlags = flags;
            copy_wide(&mut self.nid.szInfoTitle, title);
            copy_wide(&mut self.nid.szInfo, text);
            let _ = win::Shell_NotifyIconW(win::NIM_MODIFY, &mut self.nid);
            self.nid.uFlags = win::NIF_TIP;
        }
    }

    pub fn remove(&mut self) {
        if self.added {
            unsafe {
                let _ = win::Shell_NotifyIconW(win::NIM_DELETE, &mut self.nid);
            }
            self.added = false;
        }
    }
}

unsafe fn copy_wide(dst: &mut [u16], text: &str) {
    for v in dst.iter_mut() {
        *v = 0;
    }
    for (dst, src) in dst.iter_mut().zip(text.encode_utf16()) {
        *dst = src;
    }
}

unsafe fn append(menu: win::HMENU, flags: win::UINT, id: usize, text: &str) {
    let w = win::wide(text);
    win::AppendMenuW(menu, flags, id, w.as_ptr());
}

pub fn show_menu(hwnd: win::HWND, cfg: &Config) -> usize {
    unsafe {
        let menu = win::CreatePopupMenu();
        if menu.is_null() {
            return 0;
        }
        append(
            menu,
            win::MF_STRING | win::MF_DEFAULT,
            CMD_SETTINGS,
            "Настройки…",
        );
        append(menu, win::MF_SEPARATOR, 0, "");
        append(
            menu,
            win::MF_STRING
                | if cfg.visual {
                    win::MF_CHECKED
                } else {
                    win::MF_UNCHECKED
                },
            CMD_TOGGLE_VISUAL,
            "Показывать индикатор",
        );
        append(
            menu,
            win::MF_STRING
                | if cfg.sound {
                    win::MF_CHECKED
                } else {
                    win::MF_UNCHECKED
                },
            CMD_TOGGLE_SOUND,
            "Воспроизводить звук",
        );
        append(
            menu,
            win::MF_STRING
                | if cfg.block_native {
                    win::MF_CHECKED
                } else {
                    win::MF_UNCHECKED
                },
            CMD_TOGGLE_BLOCK,
            "Отключить системное переключение",
        );
        append(menu, win::MF_SEPARATOR, 0, "");
        append(menu, win::MF_STRING, CMD_EXIT, "Выход");

        let mut pt: win::POINT = std::mem::zeroed();
        win::GetCursorPos(&mut pt);
        win::SetForegroundWindow(hwnd);
        let cmd = win::TrackPopupMenu(
            menu,
            win::TPM_RETURNCMD | win::TPM_RIGHTBUTTON | win::TPM_NONOTIFY,
            pt.x,
            pt.y,
            0,
            hwnd,
            std::ptr::null(),
        );
        win::PostMessageW(hwnd, win::WM_NULL, 0, 0);
        win::DestroyMenu(menu);
        if cmd > 0 {
            cmd as usize
        } else {
            0
        }
    }
}
