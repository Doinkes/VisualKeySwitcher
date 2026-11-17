//! Application object, window procedures, lifecycle and message routing.

use std::path::PathBuf;
use std::sync::atomic::{AtomicIsize, AtomicPtr, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::OnceLock;

use crate::config::Config;
use crate::hook;
use crate::layout::{self, SwitchOutcome, SwitchRequest};
use crate::overlay;
use crate::settings;
use crate::sound;
use crate::system;
use crate::tray;
use crate::util::{self, MOD_CTRL, MOD_SHIFT};
use crate::win;

pub const WM_APP_SWITCH: win::UINT = win::WM_APP + 1;
pub const WM_APP_SWITCH_DONE: win::UINT = win::WM_APP + 2;
pub const WM_TRAY: win::UINT = win::WM_APP + 3;

static APP_PTR: AtomicPtr<App> = AtomicPtr::new(std::ptr::null_mut());
static SWITCH_TX: OnceLock<Sender<SwitchRequest>> = OnceLock::new();
static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn app_mut() -> &'static mut App {
    unsafe { &mut *APP_PTR.load(Ordering::SeqCst) }
}

pub struct App {
    pub hinst: win::HINSTANCE,
    pub hwnd_main: win::HWND,
    pub hwnd_overlay: win::HWND,
    pub hwnd_settings: win::HWND,
    pub icon_small: win::HICON,
    pub icon_big: win::HICON,
    pub cfg: Config,
    pub cfg_path: PathBuf,
    pub sound: sound::SoundLibrary,
    pub overlay: overlay::OverlayState,
    pub status: String,
    pub settings_font: win::HFONT,
    pub settings_font_bold: win::HFONT,
    pub settings_font_big: win::HFONT,
    pub settings_scale: u32,
    pub tray: Option<tray::Tray>,
    pub taskbar_created_msg: win::UINT,
    pub open_settings_msg: win::UINT,
    pub quit_msg: win::UINT,
    pub restore_msg: win::UINT,
    pub test_switch_msg: win::UINT,
    pub mutex: win::HANDLE,
}

impl App {
    pub fn save(&self) {
        self.cfg.save(&self.cfg_path);
        util::set_debug(self.cfg.debug);
    }

    pub fn sync_hook(&self) {
        hook::set_config(hook::HookConfig {
            mods: self.cfg.hotkey_mods,
            vk: self.cfg.hotkey_vk,
            block_native: self.cfg.block_native,
            trigger_on_press: self.cfg.trigger_on_press,
        });
    }

    pub fn play_preview(&mut self) {
        let (hwnd, _tid, hkl) = unsafe {
            win::foreground_layout().unwrap_or((std::ptr::null_mut(), 0, std::ptr::null_mut()))
        };
        let langid = win::language_id(hkl);
        let iso = unsafe { win::lang_iso(langid) };
        let outcome = SwitchOutcome {
            success: true,
            no_op: false,
            initial_hkl: hkl,
            target_hkl: hkl,
            hwnd,
            attempts: 0,
            elapsed_ms: 0,
            class: unsafe { win::window_class(hwnd) },
            langid,
            tag: util::iso_tag(&iso),
            preview_langid: 0,
        };
        overlay::show_preview(self, &outcome);
        self.sound.play(&self.cfg.sound_pack, &iso);
    }
}

// ------------------------------------------------------------ block control

/// True when the configured hotkey is the same modifier-only chord as the
/// restored Windows hotkey, so pressing it would switch twice.
pub fn native_hotkey_conflict(app: &App) -> bool {
    if app.cfg.hotkey_vk != 0 {
        return false;
    }
    let Some(backup) = &app.cfg.toggle_backup else {
        return false;
    };
    let native = match backup.language.as_deref() {
        Some("1") => util::MOD_ALT | util::MOD_SHIFT,
        Some("2") => util::MOD_CTRL | util::MOD_SHIFT,
        _ => 0,
    };
    native != 0 && app.cfg.hotkey_mods == native
}

pub fn apply_block(app: &mut App) -> bool {
    if app.cfg.toggle_backup.is_none() {
        app.cfg.toggle_backup = Some(system::read_toggle());
        app.save();
    }
    let ok = system::disable_native_toggle();
    if ok {
        app.cfg.block_applied = true;
        app.cfg.block_native = true;
        app.save();
        util::log("Системные сочетания переключения отключены");
    } else {
        app.cfg.block_applied = false;
        app.cfg.block_native = false;
        app.save();
        util::log("Не удалось отключить системные сочетания переключения");
    }
    ok
}

pub fn restore_block(app: &mut App) -> bool {
    let Some(backup) = app.cfg.toggle_backup.clone() else {
        app.cfg.block_applied = false;
        app.save();
        return false;
    };
    let ok = system::restore_native_toggle(&backup);
    // NOTE: the user's preference (block_native) is intentionally kept.
    // Restoring happens when the program exits or is killed; on the next
    // start the native hotkeys are disabled again.
    app.cfg.block_applied = false;
    app.save();
    util::log(if ok {
        "Системные сочетания переключения восстановлены"
    } else {
        "Не удалось восстановить системные сочетания переключения"
    });
    ok
}

// ------------------------------------------------------------ hook callbacks

pub fn request_switch() {
    let hwnd = MAIN_HWND.load(Ordering::SeqCst);
    if hwnd == 0 {
        return;
    }
    unsafe {
        win::PostMessageW(hwnd as win::HWND, WM_APP_SWITCH, 0, 0);
    }
}

fn settings_window() -> win::HWND {
    let ptr = APP_PTR.load(Ordering::SeqCst);
    if ptr.is_null() {
        return std::ptr::null_mut();
    }
    unsafe { (*ptr).hwnd_settings }
}

pub fn capture_preview(text: &str) {
    if settings_window().is_null() {
        return;
    }
    let app = app_mut();
    unsafe {
        settings::preview_value(app, text);
    }
}

pub fn capture_complete(mods: u8, vk: u16) {
    let app = app_mut();
    app.cfg.hotkey_mods = mods;
    app.cfg.hotkey_vk = vk;
    app.save();
    app.sync_hook();
    let chord = util::format_hotkey(mods, vk);
    let status = if !app.cfg.block_native && native_hotkey_conflict(app) {
        format!(
            "Сохранено: {}. Внимание: совпадает с системным сочетанием!",
            chord
        )
    } else {
        format!("Сочетание сохранено: {}", chord)
    };
    util::log(&status);
    unsafe {
        settings::show_capture_state(app);
        settings::set_status(app, &status);
    }
}

pub fn capture_cancel() {
    let app = app_mut();
    unsafe {
        settings::show_capture_state(app);
        settings::set_status(app, "Захват отменён.");
    }
}

pub fn capture_invalid(reason: &str) {
    let app = app_mut();
    unsafe {
        settings::show_capture_state(app);
        settings::set_status(app, reason);
    }
    util::log(&format!("Недопустимое сочетание: {}", reason));
}

// ------------------------------------------------------------ switching

fn on_switch_request(app: &mut App) {
    let Some((hwnd, tid, hkl)) = (unsafe { win::foreground_layout() }) else {
        util::log("Нет активного окна для переключения");
        return;
    };
    if hwnd == app.hwnd_overlay {
        return;
    }

    let mut request = SwitchRequest {
        hwnd,
        tid,
        initial_hkl: hkl,
        action: app.cfg.action.clone(),
        block_native: app.cfg.block_native,
        preview_langid: 0,
    };

    // Instant feedback: never wait for the switch to be confirmed.
    // The indicator and the sound fire right now, using the predicted target
    // layout. If the real result differs, the picture is corrected as soon
    // as the worker reports back.
    let (shown_hkl, preview_ok) = match layout::preview_target(&request) {
        layout::Preview::Target(h) => (h, true),
        layout::Preview::Missing => (hkl, false),
        layout::Preview::Nothing => {
            util::debug("Переключение не требуется: нет другой раскладки");
            return;
        }
    };
    let preview_langid = win::language_id(shown_hkl);
    let iso = unsafe { win::lang_iso(preview_langid) };
    let preview = SwitchOutcome {
        success: preview_ok,
        no_op: false,
        initial_hkl: hkl,
        target_hkl: shown_hkl,
        hwnd,
        attempts: 0,
        elapsed_ms: 0,
        class: unsafe { win::window_class(hwnd) },
        langid: preview_langid,
        tag: util::iso_tag(&iso),
        preview_langid,
    };
    request.preview_langid = if app.cfg.visual { preview_langid } else { 0 };
    if app.cfg.visual {
        overlay::show(app, &preview);
    }
    if app.cfg.sound {
        app.sound.play(&app.cfg.sound_pack, &iso);
    }

    util::debug(&format!(
        "Запрос переключения: hwnd={:?} class={} tid={} layout={:04X}",
        hwnd,
        unsafe { win::window_class(hwnd) },
        tid,
        win::language_id(hkl)
    ));
    if let Some(tx) = SWITCH_TX.get() {
        let _ = tx.send(request);
    }
}

fn on_switch_done(app: &mut App, outcome: SwitchOutcome) {
    if outcome.no_op {
        util::debug("Переключение не требуется: установлена только одна раскладка");
        return;
    }

    // The indicator and the sound were already played instantly in
    // on_switch_request. Only correct the picture if reality differs.
    if app.cfg.visual
        && outcome.preview_langid != 0
        && (outcome.langid != outcome.preview_langid || !outcome.success)
    {
        overlay::show(app, &outcome);
    }
    if let Some(t) = app.tray.as_mut() {
        t.set_tip(&format!("VisualKeySwitcher — {}", outcome.tag));
    }

    if outcome.success {
        util::log(&format!(
            "Переключено: {:04X} -> {:04X} ({}), попыток {}, {} мс, class={}",
            win::language_id(outcome.initial_hkl),
            outcome.langid,
            outcome.tag,
            outcome.attempts,
            outcome.elapsed_ms,
            outcome.class
        ));
    } else {
        util::log(&format!(
            "Не удалось переключить раскладку: class={} target={:04X} итог={:04X} попыток={} {} мс",
            outcome.class,
            win::language_id(outcome.target_hkl),
            outcome.langid,
            outcome.attempts,
            outcome.elapsed_ms
        ));
    }
}

fn start_worker(main_hwnd: win::HWND) {
    let main_hwnd = main_hwnd as isize;
    let (tx, rx) = mpsc::channel::<SwitchRequest>();
    let _ = SWITCH_TX.set(tx);
    std::thread::spawn(move || {
        while let Ok(request) = rx.recv() {
            let outcome = layout::perform_switch(&request);
            let raw = Box::into_raw(Box::new(outcome));
            let posted = unsafe {
                win::PostMessageW(
                    main_hwnd as win::HWND,
                    WM_APP_SWITCH_DONE,
                    0,
                    raw as win::LPARAM,
                )
            };
            if posted == 0 {
                unsafe {
                    drop(Box::from_raw(raw));
                }
            }
        }
    });
}

// ------------------------------------------------------------ tray menu

unsafe fn handle_tray_menu(app: &mut App) {
    let cmd = tray::show_menu(app.hwnd_main, &app.cfg);
    match cmd {
        tray::CMD_SETTINGS => settings::open(app),
        tray::CMD_TOGGLE_VISUAL => {
            app.cfg.visual = !app.cfg.visual;
            app.save();
            app.status = if app.cfg.visual {
                "Индикатор включён.".into()
            } else {
                "Индикатор выключен.".into()
            };
            settings::refresh(app);
        }
        tray::CMD_TOGGLE_SOUND => {
            app.cfg.sound = !app.cfg.sound;
            app.save();
            app.status = if app.cfg.sound {
                "Звук включён.".into()
            } else {
                "Звук выключен.".into()
            };
            settings::refresh(app);
        }
        tray::CMD_TOGGLE_BLOCK => {
            if app.cfg.block_native {
                restore_block(app);
                app.cfg.block_native = false;
                if native_hotkey_conflict(app) {
                    app.status =
                        "Системные сочетания восстановлены; ваше сочетание совпадает с системным."
                            .into();
                    if let Some(t) = app.tray.as_mut() {
                        t.balloon(
                            "VisualKeySwitcher",
                            "Системное сочетание совпадает с вашим. Возможно двойное переключение: выберите другое сочетание или включите блокировку.",
                            win::NIIF_WARNING,
                        );
                    }
                } else {
                    app.status = "Системные сочетания восстановлены.".into();
                }
            } else if apply_block(app) {
                app.status = "Системные сочетания отключены.".into();
            } else {
                app.status = "Не удалось отключить системные сочетания. См. log.txt.".into();
                if let Some(t) = app.tray.as_mut() {
                    t.balloon(
                        "VisualKeySwitcher",
                        "Не удалось отключить системные сочетания. Подробности в log.txt.",
                        win::NIIF_ERROR,
                    );
                }
            }
            app.save();
            app.sync_hook();
            settings::refresh(app);
        }
        tray::CMD_EXIT => {
            win::DestroyWindow(app.hwnd_main);
        }
        _ => {}
    }
}

// ------------------------------------------------------------ main window

unsafe extern "system" fn main_proc(
    hwnd: win::HWND,
    msg: win::UINT,
    wp: win::WPARAM,
    lp: win::LPARAM,
) -> win::LRESULT {
    if APP_PTR.load(Ordering::SeqCst).is_null() {
        return win::DefWindowProcW(hwnd, msg, wp, lp);
    }
    let app = app_mut();

    if msg == app.taskbar_created_msg && msg != 0 {
        if let Some(t) = app.tray.as_mut() {
            t.readd();
        }
        return 0;
    }
    if msg == app.open_settings_msg && msg != 0 {
        settings::open(app);
        return 0;
    }
    if msg == app.quit_msg && msg != 0 {
        win::DestroyWindow(hwnd);
        return 0;
    }
    if msg == app.restore_msg && msg != 0 {
        restore_block(app);
        app.cfg.block_native = false;
        app.save();
        app.sync_hook();
        settings::refresh(app);
        return 0;
    }
    if msg == app.test_switch_msg && msg != 0 {
        on_switch_request(app);
        return 0;
    }

    match msg {
        WM_APP_SWITCH => {
            on_switch_request(app);
            0
        }
        WM_APP_SWITCH_DONE => {
            let outcome = *Box::from_raw(lp as *mut SwitchOutcome);
            on_switch_done(app, outcome);
            0
        }
        WM_TRAY => {
            match lp as u32 {
                win::WM_RBUTTONUP | win::WM_CONTEXTMENU => handle_tray_menu(app),
                win::WM_LBUTTONDBLCLK => settings::open(app),
                _ => {}
            }
            0
        }
        win::WM_QUERYENDSESSION => {
            if app.cfg.block_applied {
                restore_block(app);
            }
            1
        }
        win::WM_ENDSESSION => {
            if wp != 0 && app.cfg.block_applied {
                restore_block(app);
            }
            0
        }
        win::WM_CLOSE => {
            win::DestroyWindow(hwnd);
            0
        }
        win::WM_DESTROY => {
            win::PostQuitMessage(0);
            0
        }
        _ => win::DefWindowProcW(hwnd, msg, wp, lp),
    }
}

unsafe fn load_icons() -> (win::HICON, win::HICON) {
    let hinst = win::GetModuleHandleW(std::ptr::null());
    let small_size = win::GetSystemMetrics(win::SM_CXSMICON).max(16);
    let big_size = 32;
    let id = 101usize as *const u16;

    let mut small = win::LoadImageW(
        hinst,
        id,
        win::IMAGE_ICON,
        small_size,
        small_size,
        win::LR_DEFAULTCOLOR,
    ) as win::HICON;
    let mut big = win::LoadImageW(
        hinst,
        id,
        win::IMAGE_ICON,
        big_size,
        big_size,
        win::LR_DEFAULTCOLOR,
    ) as win::HICON;

    if small.is_null() || big.is_null() {
        let path = util::exe_dir().join("app.ico");
        let pw = win::wide_os(path.as_os_str());
        if small.is_null() {
            small = win::LoadImageW(
                std::ptr::null_mut(),
                pw.as_ptr(),
                win::IMAGE_ICON,
                small_size,
                small_size,
                win::LR_LOADFROMFILE,
            ) as win::HICON;
        }
        if big.is_null() {
            big = win::LoadImageW(
                std::ptr::null_mut(),
                pw.as_ptr(),
                win::IMAGE_ICON,
                big_size,
                big_size,
                win::LR_LOADFROMFILE,
            ) as win::HICON;
        }
    }
    if small.is_null() {
        small = win::LoadIconW(std::ptr::null_mut(), win::IDI_APPLICATION as *const u16);
    }
    if big.is_null() {
        big = win::LoadIconW(std::ptr::null_mut(), win::IDI_APPLICATION as *const u16);
    }
    (small, big)
}

unsafe fn create_main_window(hinst: win::HINSTANCE) -> win::HWND {
    let class = win::wide("VKS_MainWnd");
    let wc = win::WNDCLASSEXW {
        cbSize: std::mem::size_of::<win::WNDCLASSEXW>() as win::UINT,
        style: 0,
        lpfnWndProc: Some(main_proc),
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
    win::RegisterClassExW(&wc);
    let title = win::wide("VisualKeySwitcher");
    win::CreateWindowExW(
        0,
        class.as_ptr(),
        title.as_ptr(),
        win::WS_OVERLAPPED,
        0,
        0,
        0,
        0,
        std::ptr::null_mut(),
        std::ptr::null_mut(),
        hinst,
        std::ptr::null_mut(),
    )
}

// ------------------------------------------------------------ startup / exit

fn validate_hotkey(cfg: &mut Config) -> bool {
    let mods = cfg.hotkey_mods;
    let vk = cfg.hotkey_vk;
    let invalid = if vk == 0 {
        mods.count_ones() < 2 || (mods & util::MOD_WIN) != 0
    } else {
        util::is_modifier(vk) || (mods == 0 && !util::lone_key_allowed(vk))
    };
    if invalid {
        util::log(&format!(
            "Некорректное сочетание в конфиге ({:?} + {}), сброшено на Ctrl+Shift",
            mods,
            util::vk_name(vk)
        ));
        cfg.hotkey_mods = MOD_CTRL | MOD_SHIFT;
        cfg.hotkey_vk = 0;
    }
    invalid
}

unsafe fn message_loop() {
    let mut msg: win::MSG = std::mem::zeroed();
    while win::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
        win::TranslateMessage(&msg);
        win::DispatchMessageW(&msg);
    }
}

unsafe fn cleanup(app: &mut App) {
    hook::uninstall();
    if app.cfg.block_applied {
        restore_block(app);
    }
    if let Some(t) = app.tray.as_mut() {
        t.remove();
    }
    if !app.hwnd_settings.is_null() {
        win::DestroyWindow(app.hwnd_settings);
        app.hwnd_settings = std::ptr::null_mut();
    }
    if !app.hwnd_overlay.is_null() {
        win::DestroyWindow(app.hwnd_overlay);
        app.hwnd_overlay = std::ptr::null_mut();
    }
    if !app.hwnd_main.is_null() && win::IsWindow(app.hwnd_main) != 0 {
        win::DestroyWindow(app.hwnd_main);
        app.hwnd_main = std::ptr::null_mut();
    }
    if !app.mutex.is_null() {
        win::CloseHandle(app.mutex);
        app.mutex = std::ptr::null_mut();
    }
}

fn broadcast(msg_name: &str) {
    unsafe {
        let msg = win::RegisterWindowMessageW(win::wide(msg_name).as_ptr());
        win::PostMessageW(win::HWND_BROADCAST, msg, 0, 0);
    }
}

fn restore_from_config(cfg_path: &PathBuf) -> bool {
    let mut cfg = Config::load_or_migrate(cfg_path);
    let ok = match cfg.toggle_backup.clone() {
        Some(b) => system::restore_native_toggle(&b),
        None => false,
    };
    cfg.block_applied = false;
    cfg.block_native = false;
    cfg.save(cfg_path);
    ok
}

pub fn run() -> i32 {
    unsafe {
        // Per-monitor DPI awareness (the manifest normally does this already).
        win::SetProcessDpiAwarenessContext(-4isize as win::HANDLE);
    }

    let args: Vec<String> = std::env::args().skip(1).collect();
    let want_settings = args.iter().any(|a| a == "--settings" || a == "-s");
    let want_quit = args.iter().any(|a| a == "--quit" || a == "-q");
    let want_restore = args.iter().any(|a| a == "--restore-system-hotkeys");
    let want_test = args.iter().any(|a| a == "--test-switch");
    let want_help = args.iter().any(|a| a == "--help" || a == "-h");

    if want_help {
        let text = "VisualKeySwitcher 2.0\r\n\r\n\
            --settings                 открыть настройки\r\n\
            --quit                     закрыть программу\r\n\
            --test-switch              проверить переключение на активном окне\r\n\
            --restore-system-hotkeys   вернуть системные сочетания Windows\r\n\r\n\
            Настройки: %APPDATA%\\VisualKeySwitcher\\config.ini\r\n\
            Журнал:    %APPDATA%\\VisualKeySwitcher\\log.txt";
        unsafe {
            win::MessageBoxW(
                std::ptr::null_mut(),
                win::wide(text).as_ptr(),
                win::wide("VisualKeySwitcher").as_ptr(),
                win::MB_OK | win::MB_ICONINFORMATION | win::MB_SETFOREGROUND,
            );
        }
        return 0;
    }

    util::init_logging(util::log_path(), false);
    let cfg_path = util::config_path();
    util::log("---- запуск ----");

    // Single instance check.
    let mutex_name = win::wide("Local\\VisualKeySwitcher.Instance");
    unsafe {
        win::SetLastError(0);
    }
    let mutex = unsafe { win::CreateMutexW(std::ptr::null(), 0, mutex_name.as_ptr()) };
    let already = !mutex.is_null() && unsafe { win::GetLastError() } == win::ERROR_ALREADY_EXISTS;

    if want_restore {
        if already {
            broadcast("VisualKeySwitcher.RestoreSystemHotkeys");
            unsafe {
                win::CloseHandle(mutex);
            }
        } else {
            let ok = restore_from_config(&cfg_path);
            util::log(&format!(
                "Ручное восстановление системных сочетаний: {}",
                if ok {
                    "успешно"
                } else {
                    "нечего восстанавливать"
                }
            ));
            unsafe {
                win::CloseHandle(mutex);
            }
        }
        return 0;
    }

    if already {
        if want_quit {
            broadcast("VisualKeySwitcher.Quit");
        } else if want_test {
            broadcast("VisualKeySwitcher.TestSwitch");
        } else {
            broadcast("VisualKeySwitcher.OpenSettings");
        }
        unsafe {
            win::CloseHandle(mutex);
        }
        return 0;
    }

    let first_run = !cfg_path.is_file();
    let mut cfg = Config::load_or_migrate(&cfg_path);
    util::set_debug(cfg.debug);
    if validate_hotkey(&mut cfg) {
        cfg.save(&cfg_path);
    }
    cfg.autostart = system::autostart_enabled();
    system::refresh_autostart_path();

    util::log(&format!(
        "Конфиг: {} | сочетание: {} | блокировка системных: {}",
        cfg_path.display(),
        util::format_hotkey(cfg.hotkey_mods, cfg.hotkey_vk),
        cfg.block_native
    ));

    let hinst = unsafe { win::GetModuleHandleW(std::ptr::null()) };
    let (icon_small, icon_big) = unsafe { load_icons() };

    let mut app = Box::new(App {
        hinst,
        hwnd_main: std::ptr::null_mut(),
        hwnd_overlay: std::ptr::null_mut(),
        hwnd_settings: std::ptr::null_mut(),
        icon_small,
        icon_big,
        cfg,
        cfg_path: cfg_path.clone(),
        sound: sound::SoundLibrary::new(),
        overlay: overlay::OverlayState::default(),
        status: String::new(),
        settings_font: std::ptr::null_mut(),
        settings_font_bold: std::ptr::null_mut(),
        settings_font_big: std::ptr::null_mut(),
        settings_scale: 96,
        tray: None,
        taskbar_created_msg: unsafe {
            win::RegisterWindowMessageW(win::wide("TaskbarCreated").as_ptr())
        },
        open_settings_msg: unsafe {
            win::RegisterWindowMessageW(win::wide("VisualKeySwitcher.OpenSettings").as_ptr())
        },
        quit_msg: unsafe {
            win::RegisterWindowMessageW(win::wide("VisualKeySwitcher.Quit").as_ptr())
        },
        restore_msg: unsafe {
            win::RegisterWindowMessageW(
                win::wide("VisualKeySwitcher.RestoreSystemHotkeys").as_ptr(),
            )
        },
        test_switch_msg: unsafe {
            win::RegisterWindowMessageW(win::wide("VisualKeySwitcher.TestSwitch").as_ptr())
        },
        mutex,
    });

    let chosen = app.sound.ensure_pack(&app.cfg.sound_pack);
    if !chosen.is_empty() && chosen != app.cfg.sound_pack {
        app.cfg.sound_pack = chosen;
        app.save();
    } else if app.cfg.sound_pack.is_empty() {
        app.cfg.sound_pack = chosen;
        app.save();
    }

    let app_ptr = Box::into_raw(app);
    APP_PTR.store(app_ptr, Ordering::SeqCst);
    let app = unsafe { &mut *app_ptr };

    unsafe {
        app.hwnd_main = create_main_window(app.hinst);
        if app.hwnd_main.is_null() {
            util::log("Не удалось создать главное окно");
            win::CloseHandle(app.mutex);
            return 1;
        }
        MAIN_HWND.store(app.hwnd_main as isize, Ordering::SeqCst);

        if !overlay::register(app.hinst) {
            util::log("Не удалось зарегистрировать класс индикатора");
        }
        app.hwnd_overlay = overlay::create(app.hinst);
        if app.hwnd_overlay.is_null() {
            util::log("Не удалось создать окно индикатора");
        }

        let mut tray = tray::Tray::new(app.hwnd_main, app.icon_small, "VisualKeySwitcher");
        tray.add();
        app.tray = Some(tray);

        // Recover from a crash that left the Windows toggle disabled.
        if !app.cfg.block_native && app.cfg.block_applied {
            util::log("Обнаружено незавершённое отключение системных сочетаний, восстанавливаю");
            restore_block(app);
        }
        if app.cfg.block_native && !apply_block(app) {
            if let Some(t) = app.tray.as_mut() {
                t.balloon(
                    "VisualKeySwitcher",
                    "Не удалось отключить системные сочетания. Подробности в log.txt.",
                    win::NIIF_ERROR,
                );
            }
        }

        if !app.cfg.block_native && native_hotkey_conflict(app) {
            let text = "Сочетание совпадает с системным: возможно двойное переключение. Включите блокировку или выберите другое сочетание.";
            util::log(text);
            app.status = text.to_string();
            if let Some(t) = app.tray.as_mut() {
                t.balloon("VisualKeySwitcher", text, win::NIIF_WARNING);
            }
        }

        if first_run && !want_settings {
            let hint = format!(
                "Работает в трее. Нажмите {} для смены языка. Настройки — правый клик по иконке.",
                util::format_hotkey(app.cfg.hotkey_mods, app.cfg.hotkey_vk)
            );
            if let Some(t) = app.tray.as_mut() {
                t.balloon("VisualKeySwitcher", &hint, win::NIIF_INFO);
            }
            util::log(&format!(
                "Первый запуск: показана подсказка в трее ({})",
                hint
            ));
        }

        if !hook::install() {
            let err = win::last_error();
            util::log(&format!(
                "Не удалось установить клавиатурный хук: {}",
                win::error_text(err)
            ));
            let text = format!(
                "Не удалось установить клавиатурный хук.\r\n\r\n{}\r\n\r\n\
                 Попробуйте запустить программу от имени администратора.",
                win::error_text(err)
            );
            win::MessageBoxW(
                std::ptr::null_mut(),
                win::wide(&text).as_ptr(),
                win::wide("VisualKeySwitcher").as_ptr(),
                win::MB_OK | win::MB_ICONERROR | win::MB_SETFOREGROUND,
            );
            cleanup(app);
            return 1;
        }

        app.sync_hook();
        start_worker(app.hwnd_main);
        util::log(&format!(
            "Готово. Сочетание: {}. Индикатор: {}. Звук: {}.",
            util::format_hotkey(app.cfg.hotkey_mods, app.cfg.hotkey_vk),
            app.cfg.visual,
            app.cfg.sound
        ));

        if want_settings {
            settings::open(app);
        }

        message_loop();

        util::log("---- выход ----");
        cleanup(app);
    }

    0
}
