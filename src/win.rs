//! Minimal hand-written Win32 bindings (no external crates).
#![allow(
    non_snake_case,
    non_upper_case_globals,
    dead_code,
    clippy::upper_case_acronyms
)]

use std::ffi::c_void;

pub type HWND = *mut c_void;
pub type HINSTANCE = *mut c_void;
pub type HICON = *mut c_void;
pub type HCURSOR = *mut c_void;
pub type HBRUSH = *mut c_void;
pub type HFONT = *mut c_void;
pub type HMENU = *mut c_void;
pub type HHOOK = *mut c_void;
pub type HANDLE = *mut c_void;
pub type HKEY = *mut c_void;
pub type HKL = *mut c_void;
pub type HDC = *mut c_void;
pub type HMONITOR = *mut c_void;
pub type LRESULT = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type UINT = u32;
pub type DWORD = u32;
pub type BOOL = i32;
pub type ATOM = u16;

// ---------------------------------------------------------------- styles

pub const WS_OVERLAPPED: DWORD = 0x0000_0000;
pub const WS_POPUP: DWORD = 0x8000_0000;
pub const WS_CHILD: DWORD = 0x4000_0000;
pub const WS_MINIMIZE: DWORD = 0x2000_0000;
pub const WS_VISIBLE: DWORD = 0x1000_0000;
pub const WS_DISABLED: DWORD = 0x0800_0000;
pub const WS_CLIPSIBLINGS: DWORD = 0x0400_0000;
pub const WS_CLIPCHILDREN: DWORD = 0x0200_0000;
pub const WS_CAPTION: DWORD = 0x00C0_0000;
pub const WS_SYSMENU: DWORD = 0x0008_0000;
pub const WS_THICKFRAME: DWORD = 0x0004_0000;
pub const WS_MINIMIZEBOX: DWORD = 0x0002_0000;
pub const WS_MAXIMIZEBOX: DWORD = 0x0001_0000;
pub const WS_OVERLAPPEDWINDOW: DWORD = 0x00CF_0000;
pub const WS_EX_TOPMOST: DWORD = 0x0000_0008;
pub const WS_EX_TRANSPARENT: DWORD = 0x0000_0020;
pub const WS_EX_TOOLWINDOW: DWORD = 0x0000_0080;
pub const WS_EX_APPWINDOW: DWORD = 0x0004_0000;
pub const WS_EX_LAYERED: DWORD = 0x0008_0000;
pub const WS_EX_NOACTIVATE: DWORD = 0x0800_0000;
pub const WS_EX_WINDOWEDGE: DWORD = 0x0000_0100;
pub const WS_EX_CLIENTEDGE: DWORD = 0x0000_0200;
pub const WS_TABSTOP: DWORD = 0x0001_0000;
pub const WS_VSCROLL: DWORD = 0x0020_0000;
pub const WS_BORDER: DWORD = 0x0080_0000;

// ---------------------------------------------------------------- messages

pub const WM_NULL: UINT = 0x0000;
pub const WM_CREATE: UINT = 0x0001;
pub const WM_DESTROY: UINT = 0x0002;
pub const WM_MOVE: UINT = 0x0003;
pub const WM_SIZE: UINT = 0x0005;
pub const WM_ACTIVATE: UINT = 0x0006;
pub const WM_SETFOCUS: UINT = 0x0007;
pub const WM_KILLFOCUS: UINT = 0x0008;
pub const WM_PAINT: UINT = 0x000F;
pub const WM_CLOSE: UINT = 0x0010;
pub const WM_QUERYENDSESSION: UINT = 0x0011;
pub const WM_ERASEBKGND: UINT = 0x0014;
pub const WM_ENDSESSION: UINT = 0x0016;
pub const WM_SHOWWINDOW: UINT = 0x0018;
pub const WM_SETTINGCHANGE: UINT = 0x001A;
pub const WM_MOUSEACTIVATE: UINT = 0x0021;
pub const WM_GETMINMAXINFO: UINT = 0x0024;
pub const WM_SETFONT: UINT = 0x0030;
pub const WM_GETFONT: UINT = 0x0031;
pub const WM_INPUTLANGCHANGEREQUEST: UINT = 0x0050;
pub const WM_CONTEXTMENU: UINT = 0x007B;
pub const WM_DISPLAYCHANGE: UINT = 0x007E;
pub const WM_SETICON: UINT = 0x0080;
pub const WM_NCHITTEST: UINT = 0x0084;
pub const WM_COMMAND: UINT = 0x0111;
pub const WM_SYSCOMMAND: UINT = 0x0112;
pub const WM_TIMER: UINT = 0x0113;
pub const WM_CTLCOLORBTN: UINT = 0x0135;
pub const WM_CTLCOLORSTATIC: UINT = 0x0138;
pub const WM_USER: UINT = 0x0400;
pub const WM_APP: UINT = 0x8000;
pub const WM_KEYDOWN: UINT = 0x0100;
pub const WM_KEYUP: UINT = 0x0101;
pub const WM_SYSKEYDOWN: UINT = 0x0104;
pub const WM_SYSKEYUP: UINT = 0x0105;
pub const WM_LBUTTONUP: UINT = 0x0202;
pub const WM_RBUTTONUP: UINT = 0x0205;
pub const WM_LBUTTONDBLCLK: UINT = 0x0203;

// ---------------------------------------------------------------- controls

pub const BS_PUSHBUTTON: DWORD = 0x0000_0000;
pub const BS_DEFPUSHBUTTON: DWORD = 0x0000_0001;
pub const BS_AUTOCHECKBOX: DWORD = 0x0000_0003;
pub const BS_GROUPBOX: DWORD = 0x0000_0007;
pub const SS_LEFT: DWORD = 0x0000_0000;
pub const SS_LEFTNOWORDWRAP: DWORD = 0x0000_000C;
pub const SS_NOPREFIX: DWORD = 0x0000_0080;
pub const CBS_DROPDOWNLIST: DWORD = 0x0000_0003;
pub const CBS_HASSTRINGS: DWORD = 0x0000_0200;
pub const ES_LEFT: DWORD = 0x0000;
pub const ES_AUTOHSCROLL: DWORD = 0x0080;
pub const ES_READONLY: DWORD = 0x0800;
pub const EM_SETREADONLY: UINT = 0x00CF;
pub const BM_SETCHECK: UINT = 0x00F1;
pub const BM_GETCHECK: UINT = 0x00F0;
pub const BST_CHECKED: usize = 1;
pub const BST_UNCHECKED: usize = 0;
pub const CB_ADDSTRING: UINT = 0x0143;
pub const CB_RESETCONTENT: UINT = 0x014B;
pub const CB_SETCURSEL: UINT = 0x014E;
pub const CB_GETCURSEL: UINT = 0x0147;
pub const CB_GETLBTEXT: UINT = 0x0148;
pub const CB_GETCOUNT: UINT = 0x0146;

// ---------------------------------------------------------------- window positioning

pub const SW_HIDE: i32 = 0;
pub const SW_SHOWNORMAL: i32 = 1;
pub const SW_SHOWNOACTIVATE: i32 = 4;
pub const SW_SHOW: i32 = 5;
pub const SW_MINIMIZE: i32 = 6;
pub const SW_RESTORE: i32 = 9;

pub const SWP_NOSIZE: UINT = 0x0001;
pub const SWP_NOMOVE: UINT = 0x0002;
pub const SWP_NOZORDER: UINT = 0x0004;
pub const SWP_NOACTIVATE: UINT = 0x0010;
pub const SWP_FRAMECHANGED: UINT = 0x0020;
pub const SWP_SHOWWINDOW: UINT = 0x0040;
pub const SWP_HIDEWINDOW: UINT = 0x0080;
pub const SWP_NOOWNERZORDER: UINT = 0x0200;

pub const HWND_TOP: HWND = 0isize as HWND;
pub const HWND_TOPMOST: HWND = -1isize as HWND;
pub const HWND_NOTOPMOST: HWND = -2isize as HWND;
pub const HWND_BROADCAST: HWND = 0xFFFF as isize as HWND;

pub const GWL_STYLE: i32 = -16;
pub const GWL_EXSTYLE: i32 = -20;
pub const GWLP_USERDATA: i32 = -21;
pub const GWLP_WNDPROC: i32 = -4;

pub const MAPVK_VK_TO_VSC: UINT = 0;

// ---------------------------------------------------------------- GDI

pub const TRANSPARENT: i32 = 1;
pub const FW_NORMAL: i32 = 400;
pub const FW_BOLD: i32 = 700;
pub const DEFAULT_CHARSET: DWORD = 1;
pub const OUT_DEFAULT_PRECIS: DWORD = 0;
pub const CLIP_DEFAULT_PRECIS: DWORD = 0;
pub const CLEARTYPE_QUALITY: DWORD = 5;
pub const DEFAULT_PITCH: DWORD = 0;
pub const FF_DONTCARE: DWORD = 0;
pub const DT_LEFT: UINT = 0x0000;
pub const DT_CENTER: UINT = 0x0001;
pub const DT_RIGHT: UINT = 0x0002;
pub const DT_VCENTER: UINT = 0x0004;
pub const DT_WORDBREAK: UINT = 0x0010;
pub const DT_SINGLELINE: UINT = 0x0020;
pub const DT_NOPREFIX: UINT = 0x0800;
pub const NULL_BRUSH: i32 = 5;
pub const LWA_ALPHA: UINT = 0x0002;

pub const COLOR_WINDOW: i32 = 5;
pub const COLOR_BTNFACE: i32 = 15;
pub const COLOR_BTNTEXT: i32 = 18;
pub const COLOR_GRAYTEXT: i32 = 17;

// ---------------------------------------------------------------- keyboard

pub const WH_KEYBOARD_LL: i32 = 13;
pub const HC_ACTION: i32 = 0;
pub const LLKHF_EXTENDED: DWORD = 0x01;
pub const LLKHF_INJECTED: DWORD = 0x10;
pub const LLKHF_UP: DWORD = 0x80;

pub const VK_LBUTTON: u16 = 0x01;
pub const VK_RBUTTON: u16 = 0x02;
pub const VK_CANCEL: u16 = 0x03;
pub const VK_MBUTTON: u16 = 0x04;
pub const VK_XBUTTON1: u16 = 0x05;
pub const VK_XBUTTON2: u16 = 0x06;
pub const VK_BACK: u16 = 0x08;
pub const VK_TAB: u16 = 0x09;
pub const VK_CLEAR: u16 = 0x0C;
pub const VK_RETURN: u16 = 0x0D;
pub const VK_SHIFT: u16 = 0x10;
pub const VK_CONTROL: u16 = 0x11;
pub const VK_MENU: u16 = 0x12;
pub const VK_PAUSE: u16 = 0x13;
pub const VK_CAPITAL: u16 = 0x14;
pub const VK_ESCAPE: u16 = 0x1B;
pub const VK_SPACE: u16 = 0x20;
pub const VK_PRIOR: u16 = 0x21;
pub const VK_NEXT: u16 = 0x22;
pub const VK_END: u16 = 0x23;
pub const VK_HOME: u16 = 0x24;
pub const VK_LEFT: u16 = 0x25;
pub const VK_UP: u16 = 0x26;
pub const VK_RIGHT: u16 = 0x27;
pub const VK_DOWN: u16 = 0x28;
pub const VK_SNAPSHOT: u16 = 0x2C;
pub const VK_INSERT: u16 = 0x2D;
pub const VK_DELETE: u16 = 0x2E;
pub const VK_LWIN: u16 = 0x5B;
pub const VK_RWIN: u16 = 0x5C;
pub const VK_APPS: u16 = 0x5D;
pub const VK_SLEEP: u16 = 0x5F;
pub const VK_NUMPAD0: u16 = 0x60;
pub const VK_NUMPAD9: u16 = 0x69;
pub const VK_MULTIPLY: u16 = 0x6A;
pub const VK_ADD: u16 = 0x6B;
pub const VK_SEPARATOR: u16 = 0x6C;
pub const VK_SUBTRACT: u16 = 0x6D;
pub const VK_DECIMAL: u16 = 0x6E;
pub const VK_DIVIDE: u16 = 0x6F;
pub const VK_F1: u16 = 0x70;
pub const VK_F24: u16 = 0x87;
pub const VK_NUMLOCK: u16 = 0x90;
pub const VK_SCROLL: u16 = 0x91;
pub const VK_OEM_1: u16 = 0xBA;
pub const VK_OEM_PLUS: u16 = 0xBB;
pub const VK_OEM_COMMA: u16 = 0xBC;
pub const VK_OEM_MINUS: u16 = 0xBD;
pub const VK_OEM_PERIOD: u16 = 0xBE;
pub const VK_OEM_2: u16 = 0xBF;
pub const VK_OEM_3: u16 = 0xC0;
pub const VK_OEM_4: u16 = 0xDB;
pub const VK_OEM_5: u16 = 0xDC;
pub const VK_OEM_6: u16 = 0xDD;
pub const VK_OEM_7: u16 = 0xDE;
pub const VK_OEM_8: u16 = 0xDF;
pub const VK_OEM_102: u16 = 0xE2;
pub const VK_LSHIFT: u16 = 0xA0;
pub const VK_RSHIFT: u16 = 0xA1;
pub const VK_LCONTROL: u16 = 0xA2;
pub const VK_RCONTROL: u16 = 0xA3;
pub const VK_LMENU: u16 = 0xA4;
pub const VK_RMENU: u16 = 0xA5;

// ---------------------------------------------------------------- system

pub const SPI_SETLANGTOGGLE: UINT = 0x005B;
pub const SPIF_UPDATEINIFILE: UINT = 0x0001;
pub const SPIF_SENDCHANGE: UINT = 0x0002;

pub const SM_CXSMICON: i32 = 49;
pub const SM_CYSMICON: i32 = 50;
pub const SM_CXSCREEN: i32 = 0;
pub const SM_CYSCREEN: i32 = 1;

pub const MONITOR_DEFAULTTONEAREST: DWORD = 2;
pub const MONITOR_DEFAULTTOPRIMARY: DWORD = 1;

pub const IMAGE_ICON: UINT = 1;
pub const LR_DEFAULTCOLOR: UINT = 0x0000_0000;
pub const LR_LOADFROMFILE: UINT = 0x0000_0010;
pub const LR_SHARED: UINT = 0x0000_8000;
pub const IDI_APPLICATION: usize = 32512;
pub const IDC_ARROW: usize = 32512;
pub const ICON_SMALL: WPARAM = 0;
pub const ICON_BIG: WPARAM = 1;

pub const CS_VREDRAW: UINT = 0x0001;
pub const CS_HREDRAW: UINT = 0x0002;

pub const MB_OK: UINT = 0x0000;
pub const MB_ICONERROR: UINT = 0x0010;
pub const MB_ICONWARNING: UINT = 0x0030;
pub const MB_ICONINFORMATION: UINT = 0x0040;
pub const MB_SETFOREGROUND: UINT = 0x0001_0000;
pub const MB_TOPMOST: UINT = 0x0004_0000;

pub const HKEY_CURRENT_USER: HKEY = 0x8000_0001u64 as HKEY;
pub const KEY_QUERY_VALUE: DWORD = 0x0001;
pub const KEY_SET_VALUE: DWORD = 0x0002;
pub const KEY_READ: DWORD = 0x20019;
pub const KEY_WRITE: DWORD = 0x20006;
pub const REG_SZ: DWORD = 1;
pub const REG_DWORD: DWORD = 4;
pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_ALREADY_EXISTS: u32 = 183;
pub const ERROR_ACCESS_DENIED: u32 = 5;

pub const SND_ASYNC: UINT = 0x0001;
pub const SND_NODEFAULT: UINT = 0x0002;
pub const SND_FILENAME: UINT = 0x0002_0000;

pub const TPM_LEFTALIGN: UINT = 0x0000;
pub const TPM_RIGHTBUTTON: UINT = 0x0002;
pub const TPM_NONOTIFY: UINT = 0x0080;
pub const TPM_RETURNCMD: UINT = 0x0100;
pub const MF_STRING: UINT = 0x0000;
pub const MF_SEPARATOR: UINT = 0x0800;
pub const MF_CHECKED: UINT = 0x0008;
pub const MF_UNCHECKED: UINT = 0x0000;
pub const MF_GRAYED: UINT = 0x0001;
pub const MF_DEFAULT: UINT = 0x1000;
pub const MF_POPUP: UINT = 0x0010;

pub const NIM_ADD: DWORD = 0;
pub const NIM_MODIFY: DWORD = 1;
pub const NIM_DELETE: DWORD = 2;
pub const NIF_MESSAGE: DWORD = 0x0001;
pub const NIF_ICON: DWORD = 0x0002;
pub const NIF_TIP: DWORD = 0x0004;
pub const NIF_INFO: DWORD = 0x0010;
pub const NIIF_INFO: DWORD = 0x0001;
pub const NIIF_WARNING: DWORD = 0x0002;
pub const NIIF_ERROR: DWORD = 0x0003;

pub const WA_INACTIVE: WPARAM = 0;
pub const MA_NOACTIVATE: LRESULT = 3;

// ---------------------------------------------------------------- structures

#[repr(C)]
pub struct WNDCLASSEXW {
    pub cbSize: UINT,
    pub style: UINT,
    pub lpfnWndProc: Option<unsafe extern "system" fn(HWND, UINT, WPARAM, LPARAM) -> LRESULT>,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: HICON,
}

#[repr(C)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: UINT,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: DWORD,
    pub pt_x: i32,
    pub pt_y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl RECT {
    pub fn width(&self) -> i32 {
        self.right - self.left
    }
    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }
}

#[repr(C)]
pub struct PAINTSTRUCT {
    pub hdc: HDC,
    pub fErase: BOOL,
    pub rcPaint: RECT,
    pub fRestore: BOOL,
    pub fIncUpdate: BOOL,
    pub rgbReserved: [u8; 32],
}

#[repr(C)]
pub struct KBDLLHOOKSTRUCT {
    pub vkCode: DWORD,
    pub scanCode: DWORD,
    pub flags: DWORD,
    pub time: DWORD,
    pub dwExtraInfo: usize,
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: DWORD,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: DWORD,
    pub dwStateMask: DWORD,
    pub szInfo: [u16; 256],
    pub uTimeout_or_uVersion: UINT,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: DWORD,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: HICON,
}

#[repr(C)]
pub struct MONITORINFO {
    pub cbSize: DWORD,
    pub rcMonitor: RECT,
    pub rcWork: RECT,
    pub dwFlags: DWORD,
}

#[repr(C)]
pub struct GUITHREADINFO {
    pub cbSize: DWORD,
    pub flags: DWORD,
    pub hwndActive: HWND,
    pub hwndFocus: HWND,
    pub hwndCapture: HWND,
    pub hwndMenuOwner: HWND,
    pub hwndMoveSize: HWND,
    pub hwndCaret: HWND,
    pub rcCaret: RECT,
}

#[repr(C)]
pub struct LOGFONTW {
    pub lfHeight: i32,
    pub lfWidth: i32,
    pub lfEscapement: i32,
    pub lfOrientation: i32,
    pub lfWeight: i32,
    pub lfItalic: u8,
    pub lfUnderline: u8,
    pub lfStrikeOut: u8,
    pub lfCharSet: u8,
    pub lfOutPrecision: u8,
    pub lfClipPrecision: u8,
    pub lfQuality: u8,
    pub lfPitchAndFamily: u8,
    pub lfFaceName: [u16; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct SYSTEMTIME {
    pub wYear: u16,
    pub wMonth: u16,
    pub wDayOfWeek: u16,
    pub wDay: u16,
    pub wHour: u16,
    pub wMinute: u16,
    pub wSecond: u16,
    pub wMilliseconds: u16,
}

// ---------------------------------------------------------------- user32

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> ATOM;
    pub fn CreateWindowExW(
        dwExStyle: DWORD,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: DWORD,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
    pub fn UpdateWindow(hWnd: HWND) -> BOOL;
    pub fn GetMessageW(
        lpMsg: *mut MSG,
        hWnd: HWND,
        wMsgFilterMin: UINT,
        wMsgFilterMax: UINT,
    ) -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn SendMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> BOOL;
    pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
    pub fn GetClassNameW(hWnd: HWND, lpClassName: *mut u16, nMaxCount: i32) -> i32;
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn SetWindowPos(
        hWnd: HWND,
        hWndInsertAfter: HWND,
        X: i32,
        Y: i32,
        cx: i32,
        cy: i32,
        uFlags: UINT,
    ) -> BOOL;
    pub fn InvalidateRect(hWnd: HWND, lpRect: *const RECT, bErase: BOOL) -> BOOL;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn GetWindowRect(hWnd: HWND, lpRect: *mut RECT) -> BOOL;
    pub fn BeginPaint(hWnd: HWND, lpPaint: *mut PAINTSTRUCT) -> HDC;
    pub fn EndPaint(hWnd: HWND, lpPaint: *const PAINTSTRUCT) -> BOOL;
    pub fn GetDC(hWnd: HWND) -> HDC;
    pub fn ReleaseDC(hWnd: HWND, hDC: HDC) -> i32;
    pub fn DrawTextW(
        hdc: HDC,
        lpchText: *const u16,
        cchText: i32,
        lprc: *mut RECT,
        format: UINT,
    ) -> i32;
    pub fn FillRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub fn FrameRect(hdc: HDC, lprc: *const RECT, hbr: HBRUSH) -> i32;
    pub fn SetLayeredWindowAttributes(hWnd: HWND, crKey: u32, bAlpha: u8, dwFlags: UINT) -> BOOL;
    pub fn SetTimer(
        hWnd: HWND,
        nIDEvent: usize,
        uElapse: UINT,
        lpTimerFunc: Option<unsafe extern "system" fn(HWND, UINT, usize, DWORD)>,
    ) -> usize;
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(
        hMenu: HMENU,
        uFlags: UINT,
        uIDNewItem: usize,
        lpNewItem: *const u16,
    ) -> BOOL;
    pub fn CheckMenuItem(hMenu: HMENU, uIDCheckItem: UINT, uCheck: UINT) -> UINT;
    pub fn DestroyMenu(hMenu: HMENU) -> BOOL;
    pub fn TrackPopupMenu(
        hMenu: HMENU,
        uFlags: UINT,
        x: i32,
        y: i32,
        nReserved: i32,
        hWnd: HWND,
        prcRect: *const RECT,
    ) -> i32;
    pub fn LoadCursorW(hInstance: HINSTANCE, lpCursorName: *const u16) -> HCURSOR;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    pub fn LoadImageW(
        hinst: HINSTANCE,
        name: *const u16,
        type_: UINT,
        cx: i32,
        cy: i32,
        fuLoad: UINT,
    ) -> HANDLE;
    pub fn DestroyIcon(hIcon: HICON) -> BOOL;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn GetForegroundWindow() -> HWND;
    pub fn AllowSetForegroundWindow(dwProcessId: DWORD) -> BOOL;
    pub fn AttachThreadInput(idAttach: DWORD, idAttachTo: DWORD, fAttach: BOOL) -> BOOL;
    pub fn GetGUIThreadInfo(idThread: DWORD, pgui: *mut GUITHREADINFO) -> BOOL;
    pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
    pub fn IsWindow(hWnd: HWND) -> BOOL;
    pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
    pub fn GetFocus() -> HWND;
    pub fn SetFocus(hWnd: HWND) -> HWND;
    pub fn EnableWindow(hWnd: HWND, bEnable: BOOL) -> BOOL;
    pub fn GetDlgItem(hDlg: HWND, nIDDlgItem: i32) -> HWND;
    pub fn GetSystemMetrics(nIndex: i32) -> i32;
    pub fn MonitorFromWindow(hWnd: HWND, dwFlags: DWORD) -> HMONITOR;
    pub fn GetMonitorInfoW(hMonitor: HMONITOR, lpmi: *mut MONITORINFO) -> BOOL;
    pub fn GetDpiForWindow(hWnd: HWND) -> UINT;
    pub fn GetDpiForSystem() -> UINT;
    pub fn GetSysColor(nIndex: i32) -> DWORD;
    pub fn GetSysColorBrush(nIndex: i32) -> HBRUSH;
    pub fn SetProcessDpiAwarenessContext(value: HANDLE) -> BOOL;
    pub fn SystemParametersInfoW(
        uiAction: UINT,
        uiParam: UINT,
        pvParam: *mut c_void,
        fWinIni: UINT,
    ) -> BOOL;
    pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: UINT) -> i32;
    pub fn RegisterWindowMessageW(lpString: *const u16) -> UINT;
    pub fn GetKeyboardLayout(idThread: DWORD) -> HKL;
    pub fn GetKeyboardLayoutList(nBuff: i32, lpList: *mut HKL) -> i32;
    pub fn ActivateKeyboardLayout(hkl: HKL, Flags: UINT) -> HKL;
    pub fn GetAsyncKeyState(vKey: i32) -> i16;
    pub fn GetLocaleInfoW(Locale: DWORD, LCType: DWORD, lpLCData: *mut u16, cchData: i32) -> i32;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn MessageBeep(uType: UINT) -> BOOL;
    pub fn SetWindowsHookExW(
        idHook: i32,
        lpfn: Option<unsafe extern "system" fn(i32, WPARAM, LPARAM) -> LRESULT>,
        hmod: HINSTANCE,
        dwThreadId: DWORD,
    ) -> HHOOK;
    pub fn UnhookWindowsHookEx(hhk: HHOOK) -> BOOL;
    pub fn CallNextHookEx(hhk: HHOOK, nCode: i32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
}

// ---------------------------------------------------------------- gdi32

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateSolidBrush(color: u32) -> HBRUSH;
    pub fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: DWORD,
        bUnderline: DWORD,
        bStrikeOut: DWORD,
        iCharSet: DWORD,
        iOutPrecision: DWORD,
        iClipPrecision: DWORD,
        iQuality: DWORD,
        iPitchAndFamily: DWORD,
        pszFaceName: *const u16,
    ) -> HFONT;
    pub fn SelectObject(hdc: HDC, h: HANDLE) -> HANDLE;
    pub fn DeleteObject(ho: HANDLE) -> BOOL;
    pub fn GetStockObject(i: i32) -> HANDLE;
    pub fn SetBkMode(hdc: HDC, mode: i32) -> i32;
    pub fn SetTextColor(hdc: HDC, color: u32) -> u32;
    pub fn SetBkColor(hdc: HDC, color: u32) -> u32;
}

// ---------------------------------------------------------------- kernel32

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn GetModuleFileNameW(hModule: HINSTANCE, lpFilename: *mut u16, nSize: DWORD) -> DWORD;
    pub fn GetCurrentThreadId() -> DWORD;
    pub fn GetCurrentProcessId() -> DWORD;
    pub fn CreateMutexW(
        lpMutexAttributes: *const c_void,
        bInitialOwner: BOOL,
        lpName: *const u16,
    ) -> HANDLE;
    pub fn CloseHandle(hObject: HANDLE) -> BOOL;
    pub fn GetLastError() -> DWORD;
    pub fn SetLastError(dwErrCode: DWORD);
    pub fn FormatMessageW(
        dwFlags: DWORD,
        lpSource: *const c_void,
        dwMessageId: DWORD,
        dwLanguageId: DWORD,
        lpBuffer: *mut u16,
        nSize: DWORD,
        Arguments: *const *const u16,
    ) -> DWORD;
    pub fn Sleep(dwMilliseconds: DWORD);
    pub fn GetTickCount() -> DWORD;
    pub fn GetTickCount64() -> u64;
    pub fn GetLocalTime(lpSystemTime: *mut SYSTEMTIME);
    pub fn GetEnvironmentVariableW(lpName: *const u16, lpBuffer: *mut u16, nSize: DWORD) -> DWORD;
    pub fn MulDiv(nNumber: i32, nNumerator: i32, nDenominator: i32) -> i32;
    pub fn GetACP() -> UINT;
}

// ---------------------------------------------------------------- advapi32

#[link(name = "advapi32")]
extern "system" {
    pub fn RegCreateKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        Reserved: DWORD,
        lpClass: *const u16,
        dwOptions: DWORD,
        samDesired: DWORD,
        lpSecurityAttributes: *const c_void,
        phkResult: *mut HKEY,
        lpdwDisposition: *mut DWORD,
    ) -> i32;
    pub fn RegOpenKeyExW(
        hKey: HKEY,
        lpSubKey: *const u16,
        ulOptions: DWORD,
        samDesired: DWORD,
        phkResult: *mut HKEY,
    ) -> i32;
    pub fn RegQueryValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        lpReserved: *mut DWORD,
        lpType: *mut DWORD,
        lpData: *mut u8,
        lpcbData: *mut DWORD,
    ) -> i32;
    pub fn RegSetValueExW(
        hKey: HKEY,
        lpValueName: *const u16,
        Reserved: DWORD,
        dwType: DWORD,
        lpData: *const u8,
        cbData: DWORD,
    ) -> i32;
    pub fn RegDeleteValueW(hKey: HKEY, lpValueName: *const u16) -> i32;
    pub fn RegCloseKey(hKey: HKEY) -> i32;
}

// ---------------------------------------------------------------- shell32

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(dwMessage: DWORD, lpData: *mut NOTIFYICONDATAW) -> BOOL;
}

// ---------------------------------------------------------------- winmm

#[link(name = "winmm")]
extern "system" {
    pub fn PlaySoundW(pszSound: *const u16, hmod: HINSTANCE, fdwSound: UINT) -> BOOL;
}

// ---------------------------------------------------------------- shcore

#[link(name = "shcore")]
extern "system" {
    pub fn GetDpiForMonitor(
        hmonitor: HMONITOR,
        dpiType: i32,
        dpiX: *mut UINT,
        dpiY: *mut UINT,
    ) -> i32;
}

// ---------------------------------------------------------------- helpers

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub fn wide_os(s: &std::ffi::OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    s.encode_wide().chain(std::iter::once(0)).collect()
}

pub fn from_wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|c| *c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

pub fn last_error() -> u32 {
    unsafe { GetLastError() }
}

pub fn error_text(code: u32) -> String {
    let mut buf = [0u16; 512];
    let n = unsafe {
        FormatMessageW(
            0x0000_1000 | 0x0000_0200, // FROM_SYSTEM | IGNORE_INSERTS
            std::ptr::null(),
            code,
            0,
            buf.as_mut_ptr(),
            buf.len() as DWORD,
            std::ptr::null(),
        )
    };
    if n == 0 {
        format!("код {}", code)
    } else {
        from_wide(&buf).trim().to_string()
    }
}

pub fn rgb(r: u8, g: u8, b: u8) -> u32 {
    (r as u32) | ((g as u32) << 8) | ((b as u32) << 16)
}

/// Thread/process id owner of a window.
pub unsafe fn window_thread(hwnd: HWND) -> DWORD {
    GetWindowThreadProcessId(hwnd, std::ptr::null_mut())
}

/// Safe-ish wrapper: returns the class name of a window.
pub unsafe fn window_class(hwnd: HWND) -> String {
    if hwnd.is_null() {
        return String::new();
    }
    let mut buf = [0u16; 128];
    let n = GetClassNameW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    if n <= 0 {
        String::new()
    } else {
        from_wide(&buf[..n as usize])
    }
}

pub unsafe fn get_window_text(hwnd: HWND) -> String {
    let mut buf = [0u16; 256];
    let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
    if n <= 0 {
        String::new()
    } else {
        from_wide(&buf[..n as usize])
    }
}

pub unsafe fn set_window_text(hwnd: HWND, text: &str) {
    let w = wide(text);
    SetWindowTextW(hwnd, w.as_ptr());
}

pub unsafe fn get_dlg_item(hwnd: HWND, id: i32) -> HWND {
    GetDlgItem(hwnd, id)
}

pub unsafe fn send_msg(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> LRESULT {
    SendMessageW(hwnd, msg, wp, lp)
}

pub unsafe fn post_msg(hwnd: HWND, msg: UINT, wp: WPARAM, lp: LPARAM) -> bool {
    PostMessageW(hwnd, msg, wp, lp) != 0
}

pub unsafe fn timer(hwnd: HWND, id: usize, ms: UINT) {
    SetTimer(hwnd, id, ms, None);
}

pub unsafe fn kill_timer(hwnd: HWND, id: usize) {
    KillTimer(hwnd, id);
}

/// Makes a DPI-aware pixel value from a 96-dpi design value.
pub fn scale(v: i32, dpi: u32) -> i32 {
    unsafe { MulDiv(v, dpi as i32, 96) }
}

pub unsafe fn monitor_info_for(hwnd: HWND) -> Option<(RECT, u32)> {
    let mon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    if mon.is_null() {
        return None;
    }
    let mut mi: MONITORINFO = std::mem::zeroed();
    mi.cbSize = std::mem::size_of::<MONITORINFO>() as DWORD;
    if GetMonitorInfoW(mon, &mut mi) == 0 {
        return None;
    }
    let mut dx: UINT = 96;
    let mut dy: UINT = 96;
    let _ = GetDpiForMonitor(mon, 0 /* MDT_EFFECTIVE_DPI */, &mut dx, &mut dy);
    let dpi = if dx >= 72 { dx } else { 96 };
    Some((mi.rcWork, dpi))
}

pub unsafe fn foreground_layout() -> Option<(HWND, DWORD, HKL)> {
    let hwnd = GetForegroundWindow();
    if hwnd.is_null() {
        return None;
    }
    let tid = window_thread(hwnd);
    if tid == 0 {
        return None;
    }
    Some((hwnd, tid, GetKeyboardLayout(tid)))
}

pub fn language_id(hkl: HKL) -> u16 {
    (hkl as usize & 0xFFFF) as u16
}

pub unsafe fn layouts() -> Vec<HKL> {
    let n = GetKeyboardLayoutList(0, std::ptr::null_mut());
    if n <= 0 {
        return Vec::new();
    }
    let mut list = vec![std::ptr::null_mut(); n as usize];
    let got = GetKeyboardLayoutList(n, list.as_mut_ptr());
    if got <= 0 {
        return Vec::new();
    }
    list.truncate(got as usize);
    list
}

pub unsafe fn lang_iso(langid: u16) -> String {
    let mut buf = [0u16; 32];
    let lcid = langid as DWORD; // SORT_DEFAULT == 0
    let n = GetLocaleInfoW(
        lcid,
        0x0000_0059, /* LOCALE_SISO639LANGNAME */
        buf.as_mut_ptr(),
        buf.len() as i32,
    );
    if n <= 1 {
        return String::new();
    }
    from_wide(&buf).to_lowercase()
}

pub unsafe fn is_key_down(vk: i32) -> bool {
    (GetAsyncKeyState(vk) as u16 & 0x8000) != 0
}

/// Creates a font scaled by DPI. `points` is the point size at 96 dpi.
pub unsafe fn ui_font(dpi: u32, points: i32, bold: bool) -> HFONT {
    let height = -MulDiv(points, dpi as i32, 72);
    let face = wide("Segoe UI");
    CreateFontW(
        height,
        0,
        0,
        0,
        if bold { FW_BOLD } else { FW_NORMAL },
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        DEFAULT_PITCH | FF_DONTCARE,
        face.as_ptr(),
    )
}
