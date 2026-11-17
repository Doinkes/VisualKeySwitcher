//! Settings window (Russian UI), built with plain Win32 controls.

use crate::app::{app_mut, App};
use crate::config::Action;
use crate::hook;
use crate::system;
use crate::util;
use crate::win;

const ID_HOTKEY_VALUE: i32 = 1001;
const ID_HOTKEY_BTN: i32 = 1002;
const ID_HOTKEY_HINT: i32 = 1003;
const ID_ACTION: i32 = 1011;
const ID_TRIGGER: i32 = 1012;
const ID_BLOCK: i32 = 1020;
const ID_BLOCK_HINT: i32 = 1021;
const ID_VISUAL: i32 = 1030;
const ID_SOUND: i32 = 1031;
const ID_VTYPE: i32 = 1033;
const ID_PACK: i32 = 1035;
const ID_TEST: i32 = 1036;
const ID_AUTOSTART: i32 = 1040;
const ID_STATUS: i32 = 1041;
const ID_CLOSE: i32 = 1050;

const HOTKEY_HINT: &str = "Сочетание работает независимо от Windows. «При нажатии» — быстрее, но Ctrl+Shift+буква тоже переключит язык. «При отпускании» — безопаснее для сочетаний.";
const CAPTURE_HINT: &str = "Нажмите нужное сочетание. Esc — отмена. Для сочетания только из модификаторов нужно минимум два, например Ctrl + Shift.";
const BLOCK_HINT: &str = "Ctrl+Shift, Alt+Shift и Win+Space перестанут переключать язык. Настройка применяется сразу; при выходе из программы исходные значения восстанавливаются.";

unsafe fn set_text(parent: win::HWND, id: i32, text: &str) {
    let child = win::GetDlgItem(parent, id);
    if !child.is_null() {
        win::set_window_text(child, text);
    }
}

unsafe fn make(
    parent: win::HWND,
    class: &str,
    text: &str,
    style: win::DWORD,
    ex_style: win::DWORD,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    id: i32,
    font: win::HFONT,
) -> win::HWND {
    let cls = win::wide(class);
    let txt = win::wide(text);
    let child = win::CreateWindowExW(
        ex_style,
        cls.as_ptr(),
        txt.as_ptr(),
        win::WS_CHILD | win::WS_VISIBLE | style,
        x,
        y,
        w,
        h,
        parent,
        id as isize as win::HMENU,
        win::GetModuleHandleW(std::ptr::null()),
        std::ptr::null_mut(),
    );
    if !child.is_null() && !font.is_null() {
        win::SendMessageW(child, win::WM_SETFONT, font as usize, 1);
    }
    child
}

unsafe fn create_controls(app: &mut App, hwnd: win::HWND) {
    let dpi = win::GetDpiForWindow(hwnd).max(96);
    app.settings_scale = dpi;
    app.settings_font = win::ui_font(dpi, 9, false);
    app.settings_font_bold = win::ui_font(dpi, 10, true);
    app.settings_font_big = win::ui_font(dpi, 11, true);

    let f = app.settings_font;
    let fbig = app.settings_font_big;
    let s = |v: i32| win::scale(v, dpi);

    // --- group 1: switching
    make(
        hwnd,
        "BUTTON",
        "Переключение языка",
        win::BS_GROUPBOX,
        0,
        s(12),
        s(10),
        s(560),
        s(200),
        0,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "Сочетание клавиш:",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(38),
        s(160),
        s(22),
        0,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "",
        win::SS_LEFTNOWORDWRAP | win::SS_NOPREFIX,
        0,
        s(190),
        s(33),
        s(250),
        s(30),
        ID_HOTKEY_VALUE,
        fbig,
    );
    make(
        hwnd,
        "BUTTON",
        "Задать…",
        win::BS_PUSHBUTTON | win::WS_TABSTOP,
        0,
        s(450),
        s(32),
        s(106),
        s(30),
        ID_HOTKEY_BTN,
        f,
    );
    make(
        hwnd,
        "STATIC",
        HOTKEY_HINT,
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(68),
        s(536),
        s(50),
        ID_HOTKEY_HINT,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "Действие:",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(128),
        s(160),
        s(22),
        0,
        f,
    );
    make(
        hwnd,
        "COMBOBOX",
        "",
        win::CBS_DROPDOWNLIST | win::WS_TABSTOP | win::WS_VSCROLL,
        0,
        s(190),
        s(123),
        s(250),
        s(200),
        ID_ACTION,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "Срабатывание:",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(162),
        s(160),
        s(22),
        0,
        f,
    );
    make(
        hwnd,
        "COMBOBOX",
        "",
        win::CBS_DROPDOWNLIST | win::WS_TABSTOP | win::WS_VSCROLL,
        0,
        s(190),
        s(157),
        s(330),
        s(200),
        ID_TRIGGER,
        f,
    );

    // --- group 2: system hotkeys
    make(
        hwnd,
        "BUTTON",
        "Системное переключение",
        win::BS_GROUPBOX,
        0,
        s(12),
        s(220),
        s(560),
        s(98),
        0,
        f,
    );
    make(
        hwnd,
        "BUTTON",
        "Отключить встроенные сочетания Windows",
        win::BS_AUTOCHECKBOX | win::WS_TABSTOP,
        0,
        s(24),
        s(242),
        s(536),
        s(22),
        ID_BLOCK,
        f,
    );
    make(
        hwnd,
        "STATIC",
        BLOCK_HINT,
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(268),
        s(536),
        s(42),
        ID_BLOCK_HINT,
        f,
    );

    // --- group 3: indicator and sound
    make(
        hwnd,
        "BUTTON",
        "Индикатор и звук",
        win::BS_GROUPBOX,
        0,
        s(12),
        s(328),
        s(560),
        s(150),
        0,
        f,
    );
    make(
        hwnd,
        "BUTTON",
        "Показывать индикатор",
        win::BS_AUTOCHECKBOX | win::WS_TABSTOP,
        0,
        s(24),
        s(350),
        s(220),
        s(22),
        ID_VISUAL,
        f,
    );
    make(
        hwnd,
        "BUTTON",
        "Воспроизводить звук",
        win::BS_AUTOCHECKBOX | win::WS_TABSTOP,
        0,
        s(290),
        s(350),
        s(270),
        s(22),
        ID_SOUND,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "Вид индикатора:",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(386),
        s(160),
        s(22),
        0,
        f,
    );
    make(
        hwnd,
        "COMBOBOX",
        "",
        win::CBS_DROPDOWNLIST | win::WS_TABSTOP | win::WS_VSCROLL,
        0,
        s(190),
        s(381),
        s(240),
        s(200),
        ID_VTYPE,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "Звуковой пакет:",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(416),
        s(160),
        s(22),
        0,
        f,
    );
    make(
        hwnd,
        "COMBOBOX",
        "",
        win::CBS_DROPDOWNLIST | win::WS_TABSTOP | win::WS_VSCROLL,
        0,
        s(190),
        s(411),
        s(240),
        s(200),
        ID_PACK,
        f,
    );
    make(
        hwnd,
        "BUTTON",
        "Проверить",
        win::BS_PUSHBUTTON | win::WS_TABSTOP,
        0,
        s(450),
        s(409),
        s(106),
        s(30),
        ID_TEST,
        f,
    );

    // --- group 4: misc
    make(
        hwnd,
        "BUTTON",
        "Прочее",
        win::BS_GROUPBOX,
        0,
        s(12),
        s(488),
        s(560),
        s(86),
        0,
        f,
    );
    make(
        hwnd,
        "BUTTON",
        "Запускать при входе в Windows",
        win::BS_AUTOCHECKBOX | win::WS_TABSTOP,
        0,
        s(24),
        s(510),
        s(536),
        s(22),
        ID_AUTOSTART,
        f,
    );
    make(
        hwnd,
        "STATIC",
        "",
        win::SS_LEFT | win::SS_NOPREFIX,
        0,
        s(24),
        s(536),
        s(536),
        s(30),
        ID_STATUS,
        f,
    );

    make(
        hwnd,
        "BUTTON",
        "Закрыть",
        win::BS_DEFPUSHBUTTON | win::WS_TABSTOP,
        0,
        s(468),
        s(582),
        s(96),
        s(30),
        ID_CLOSE,
        f,
    );
}

unsafe fn combo_reset(combo: win::HWND) {
    win::SendMessageW(combo, win::CB_RESETCONTENT, 0, 0);
}

unsafe fn combo_add(combo: win::HWND, text: &str) -> i32 {
    let w = win::wide(text);
    win::SendMessageW(combo, win::CB_ADDSTRING, 0, w.as_ptr() as win::LPARAM) as i32
}

unsafe fn combo_select(combo: win::HWND, index: i32) {
    win::SendMessageW(combo, win::CB_SETCURSEL, index.max(0) as usize, 0);
}

unsafe fn combo_current(combo: win::HWND) -> i32 {
    win::SendMessageW(combo, win::CB_GETCURSEL, 0, 0) as i32
}

unsafe fn combo_text(combo: win::HWND, index: i32) -> String {
    if index < 0 {
        return String::new();
    }
    let mut buf = [0u16; 260];
    let n = win::SendMessageW(
        combo,
        win::CB_GETLBTEXT,
        index as usize,
        buf.as_mut_ptr() as win::LPARAM,
    );
    if n <= 0 {
        return String::new();
    }
    win::from_wide(&buf[..n as usize])
}

unsafe fn check_state(parent: win::HWND, id: i32) -> bool {
    let h = win::GetDlgItem(parent, id);
    !h.is_null() && win::SendMessageW(h, win::BM_GETCHECK, 0, 0) as usize == win::BST_CHECKED
}

unsafe fn set_check(parent: win::HWND, id: i32, checked: bool) {
    let h = win::GetDlgItem(parent, id);
    if !h.is_null() {
        win::SendMessageW(
            h,
            win::BM_SETCHECK,
            if checked {
                win::BST_CHECKED
            } else {
                win::BST_UNCHECKED
            },
            0,
        );
    }
}

pub unsafe fn populate(app: &mut App, hwnd: win::HWND) {
    set_text(
        hwnd,
        ID_HOTKEY_VALUE,
        &util::format_hotkey(app.cfg.hotkey_mods, app.cfg.hotkey_vk),
    );
    set_text(hwnd, ID_HOTKEY_HINT, HOTKEY_HINT);
    set_text(hwnd, ID_HOTKEY_BTN, "Задать…");

    let action = win::GetDlgItem(hwnd, ID_ACTION);
    combo_reset(action);
    let actions = [
        Action::Next,
        Action::Lang("ru".into()),
        Action::Lang("en".into()),
    ];
    for a in actions.iter() {
        combo_add(action, &a.label());
    }
    let idx = actions
        .iter()
        .position(|a| *a == app.cfg.action)
        .unwrap_or(0) as i32;
    combo_select(action, idx);

    let trigger = win::GetDlgItem(hwnd, ID_TRIGGER);
    combo_reset(trigger);
    combo_add(trigger, "При нажатии (быстрее)");
    combo_add(trigger, "При отпускании (безопаснее)");
    combo_select(trigger, if app.cfg.trigger_on_press { 0 } else { 1 });

    set_check(hwnd, ID_BLOCK, app.cfg.block_native);
    set_check(hwnd, ID_VISUAL, app.cfg.visual);
    set_check(hwnd, ID_SOUND, app.cfg.sound);
    set_check(hwnd, ID_AUTOSTART, app.cfg.autostart);

    let vtype = win::GetDlgItem(hwnd, ID_VTYPE);
    combo_reset(vtype);
    combo_add(vtype, "Classic — чёрный квадрат");
    combo_add(vtype, "Modern — цветной квадрат");
    combo_add(vtype, "Squares — цветная полоса");
    combo_select(vtype, app.cfg.visual_type.min(2) as i32);

    app.sound.rescan();
    if app.cfg.sound_pack.is_empty() || !app.sound.packs().iter().any(|p| p == &app.cfg.sound_pack)
    {
        let chosen = app.sound.ensure_pack(&app.cfg.sound_pack);
        if !chosen.is_empty() {
            app.cfg.sound_pack = chosen;
            app.save();
        }
    }
    let pack = win::GetDlgItem(hwnd, ID_PACK);
    combo_reset(pack);
    let mut selected = 0;
    if app.sound.packs().is_empty() {
        combo_add(pack, "sound_pack не найден");
        win::EnableWindow(pack, 0);
    } else {
        win::EnableWindow(pack, 1);
        for (i, p) in app.sound.packs().iter().enumerate() {
            combo_add(pack, p);
            if p == &app.cfg.sound_pack {
                selected = i as i32;
            }
        }
    }
    combo_select(pack, selected);

    set_text(hwnd, ID_STATUS, &app.status);

    if hook::is_capturing() {
        show_capture_state(app);
    }
}

pub unsafe fn refresh(app: &mut App) {
    let hwnd = app.hwnd_settings;
    if hwnd.is_null() {
        return;
    }
    populate(app, hwnd);
}

pub unsafe fn set_status(app: &mut App, text: &str) {
    app.status = text.to_string();
    if !app.hwnd_settings.is_null() {
        set_text(app.hwnd_settings, ID_STATUS, text);
    }
}

pub unsafe fn preview_value(app: &mut App, text: &str) {
    if !app.hwnd_settings.is_null() {
        set_text(app.hwnd_settings, ID_HOTKEY_VALUE, text);
    }
}

pub unsafe fn show_capture_state(app: &mut App) {
    let hwnd = app.hwnd_settings;
    if hwnd.is_null() {
        return;
    }
    if hook::is_capturing() {
        set_text(hwnd, ID_HOTKEY_VALUE, "Нажмите сочетание…");
        set_text(hwnd, ID_HOTKEY_BTN, "Отмена");
        set_text(hwnd, ID_HOTKEY_HINT, CAPTURE_HINT);
    } else {
        set_text(
            hwnd,
            ID_HOTKEY_VALUE,
            &util::format_hotkey(app.cfg.hotkey_mods, app.cfg.hotkey_vk),
        );
        set_text(hwnd, ID_HOTKEY_BTN, "Задать…");
        set_text(hwnd, ID_HOTKEY_HINT, HOTKEY_HINT);
    }
}

pub unsafe fn open(app: &mut App) {
    if app.status.is_empty() && !app.cfg.block_native && crate::app::native_hotkey_conflict(app) {
        app.status = "Сочетание совпадает с системным: возможно двойное переключение. Включите блокировку или выберите другое.".to_string();
    }
    if !app.hwnd_settings.is_null() {
        show_capture_state(app);
        populate(app, app.hwnd_settings);
        win::ShowWindow(app.hwnd_settings, win::SW_RESTORE);
        win::SetForegroundWindow(app.hwnd_settings);
        return;
    }

    let class = win::wide("VKS_SettingsWnd");
    let wc = win::WNDCLASSEXW {
        cbSize: std::mem::size_of::<win::WNDCLASSEXW>() as win::UINT,
        style: win::CS_HREDRAW | win::CS_VREDRAW,
        lpfnWndProc: Some(proc_),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: app.hinst,
        hIcon: app.icon_big,
        hCursor: win::LoadCursorW(std::ptr::null_mut(), win::IDC_ARROW as *const u16),
        hbrBackground: (win::COLOR_BTNFACE + 1) as win::HBRUSH,
        lpszMenuName: std::ptr::null(),
        lpszClassName: class.as_ptr(),
        hIconSm: app.icon_small,
    };
    if win::RegisterClassExW(&wc) == 0 {
        let err = win::last_error();
        if err != 0 && err != 1410 {
            util::log(&format!(
                "RegisterClassExW(settings) ошибка: {}",
                win::error_text(err)
            ));
        }
    }

    let dpi = win::GetDpiForSystem().max(96);
    let w = win::scale(600, dpi);

    let (work, _) = unsafe { win::monitor_info_for(app.hwnd_main) }.unwrap_or((
        win::RECT {
            left: 0,
            top: 0,
            right: w,
            bottom: win::scale(660, dpi),
        },
        dpi,
    ));
    let h = win::scale(660, dpi).min(work.height().max(200));
    let x = work.left + (work.width() - w) / 2;
    let y = (work.top + (work.height() - h) / 2).max(work.top);

    let title = win::wide("VisualKeySwitcher — настройки");
    let hwnd = win::CreateWindowExW(
        0,
        class.as_ptr(),
        title.as_ptr(),
        win::WS_OVERLAPPED | win::WS_CAPTION | win::WS_SYSMENU | win::WS_MINIMIZEBOX,
        x,
        y,
        w,
        h,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        app.hinst,
        std::ptr::null_mut(),
    );
    if hwnd.is_null() {
        let err = win::last_error();
        util::log(&format!(
            "Не удалось создать окно настроек: {}",
            win::error_text(err)
        ));
        return;
    }

    app.hwnd_settings = hwnd;
    win::ShowWindow(hwnd, win::SW_SHOW);
    win::UpdateWindow(hwnd);
    win::SetForegroundWindow(hwnd);
}

fn action_for_index(index: i32) -> Action {
    match index {
        1 => Action::Lang("ru".into()),
        2 => Action::Lang("en".into()),
        _ => Action::Next,
    }
}

unsafe fn handle_command(app: &mut App, hwnd: win::HWND, id: i32, code: u16) {
    match id {
        ID_HOTKEY_BTN => {
            if hook::is_capturing() {
                hook::cancel_capture();
                set_status(app, "Захват отменён.");
            } else {
                hook::start_capture();
                set_status(app, "Нажмите сочетание клавиш. Esc — отмена.");
            }
            show_capture_state(app);
        }
        ID_ACTION => {
            if code == 1 {
                // CBN_SELCHANGE
                let combo = win::GetDlgItem(hwnd, ID_ACTION);
                let idx = combo_current(combo);
                app.cfg.action = action_for_index(idx);
                app.save();
                app.sync_hook();
                set_status(app, "Действие сохранено.");
            }
        }
        ID_TRIGGER => {
            if code == 1 {
                let combo = win::GetDlgItem(hwnd, ID_TRIGGER);
                app.cfg.trigger_on_press = combo_current(combo) == 0;
                app.save();
                app.sync_hook();
                set_status(
                    app,
                    if app.cfg.trigger_on_press {
                        "Срабатывание: при нажатии."
                    } else {
                        "Срабатывание: при отпускании."
                    },
                );
            }
        }
        ID_BLOCK => {
            let want = check_state(hwnd, ID_BLOCK);
            if want {
                if crate::app::apply_block(app) {
                    app.cfg.block_native = true;
                    set_status(app, "Системные сочетания отключены.");
                } else {
                    app.cfg.block_native = false;
                    set_check(hwnd, ID_BLOCK, false);
                    set_status(
                        app,
                        "Не удалось отключить системные сочетания. См. log.txt.",
                    );
                }
            } else {
                crate::app::restore_block(app);
                app.cfg.block_native = false;
                if crate::app::native_hotkey_conflict(app) {
                    set_status(
                        app,
                        "Системные сочетания восстановлены, но ваше сочетание совпадает с системным — возможно двойное переключение.",
                    );
                } else {
                    set_status(app, "Системные сочетания восстановлены.");
                }
            }
            app.save();
            app.sync_hook();
        }
        ID_VISUAL => {
            app.cfg.visual = check_state(hwnd, ID_VISUAL);
            app.save();
            set_status(
                app,
                if app.cfg.visual {
                    "Индикатор включён."
                } else {
                    "Индикатор выключен."
                },
            );
        }
        ID_SOUND => {
            app.cfg.sound = check_state(hwnd, ID_SOUND);
            app.save();
            set_status(
                app,
                if app.cfg.sound {
                    "Звук включён."
                } else {
                    "Звук выключен."
                },
            );
        }
        ID_AUTOSTART => {
            let want = check_state(hwnd, ID_AUTOSTART);
            if system::set_autostart(want) {
                app.cfg.autostart = want;
                app.save();
                set_status(
                    app,
                    if want {
                        "Автозапуск включён."
                    } else {
                        "Автозапуск выключен."
                    },
                );
            } else {
                set_check(hwnd, ID_AUTOSTART, !want);
                set_status(app, "Не удалось изменить автозапуск.");
            }
        }
        ID_VTYPE => {
            let combo = win::GetDlgItem(hwnd, ID_VTYPE);
            let idx = combo_current(combo).clamp(0, 2) as u8;
            app.cfg.visual_type = idx;
            app.save();
            set_status(app, "Вид индикатора сохранён.");
        }
        ID_PACK => {
            let combo = win::GetDlgItem(hwnd, ID_PACK);
            let idx = combo_current(combo);
            let text = combo_text(combo, idx);
            if !text.is_empty() {
                app.cfg.sound_pack = text;
                app.save();
                set_status(
                    app,
                    &format!(
                        "Звуковой пакет: {}. Нажмите «Проверить», чтобы услышать.",
                        app.cfg.sound_pack
                    ),
                );
            }
        }
        ID_TEST => {
            app.play_preview();
            set_status(app, "Проверка индикатора и звука.");
        }
        ID_CLOSE => {
            win::DestroyWindow(hwnd);
        }
        _ => {}
    }
}

pub unsafe extern "system" fn proc_(
    hwnd: win::HWND,
    msg: win::UINT,
    wp: win::WPARAM,
    lp: win::LPARAM,
) -> win::LRESULT {
    match msg {
        win::WM_CREATE => {
            let app = app_mut();
            create_controls(app, hwnd);
            populate(app, hwnd);
            0
        }
        win::WM_COMMAND => {
            let app = app_mut();
            let id = (wp & 0xFFFF) as i32;
            let code = ((wp >> 16) & 0xFFFF) as u16;
            handle_command(app, hwnd, id, code);
            0
        }
        win::WM_CTLCOLORSTATIC => {
            let hdc = wp as win::HDC;
            let bg = win::GetSysColor(win::COLOR_BTNFACE);
            win::SetBkColor(hdc, bg);
            win::GetSysColorBrush(win::COLOR_BTNFACE) as win::LRESULT
        }
        win::WM_ACTIVATE => {
            let app = app_mut();
            if (wp & 0xFFFF) as u32 == win::WA_INACTIVE as u32 && hook::is_capturing() {
                hook::cancel_capture();
                set_status(app, "Захват отменён.");
                show_capture_state(app);
            }
            0
        }
        win::WM_CLOSE => {
            if hook::is_capturing() {
                hook::cancel_capture();
            }
            win::DestroyWindow(hwnd);
            0
        }
        win::WM_DESTROY => {
            let app = app_mut();
            hook::cancel_capture();
            app.hwnd_settings = std::ptr::null_mut();
            for font in [
                app.settings_font,
                app.settings_font_bold,
                app.settings_font_big,
            ] {
                if !font.is_null() {
                    win::DeleteObject(font);
                }
            }
            app.settings_font = std::ptr::null_mut();
            app.settings_font_bold = std::ptr::null_mut();
            app.settings_font_big = std::ptr::null_mut();
            0
        }
        _ => win::DefWindowProcW(hwnd, msg, wp, lp),
    }
}
