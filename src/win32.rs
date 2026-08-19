#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::ffi::c_void;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::{ffi::OsStr, ptr};

pub type BOOL = i32;
pub type HWND = isize;
pub type HINSTANCE = isize;
pub type HICON = isize;
pub type HCURSOR = isize;
pub type HBRUSH = isize;
pub type HDC = isize;
pub type HGDIOBJ = isize;
pub type HBITMAP = isize;
pub type HMENU = isize;
pub type HANDLE = isize;
pub type WPARAM = usize;
pub type LPARAM = isize;
pub type LRESULT = isize;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GUID {
    pub data1: u32,
    pub data2: u16,
    pub data3: u16,
    pub data4: [u8; 8],
}

impl GUID {
    pub const fn new(data1: u32, data2: u16, data3: u16, data4: [u8; 8]) -> Self {
        Self {
            data1,
            data2,
            data3,
            data4,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct POINT {
    pub x: i32,
    pub y: i32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
pub struct MEASUREITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemWidth: u32,
    pub itemHeight: u32,
    pub itemData: usize,
}

#[repr(C)]
pub struct DRAWITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemAction: u32,
    pub itemState: u32,
    pub hwndItem: HWND,
    pub hDC: HDC,
    pub rcItem: RECT,
    pub itemData: usize,
}

#[repr(C)]
pub struct BITMAP {
    pub bmType: i32,
    pub bmWidth: i32,
    pub bmHeight: i32,
    pub bmWidthBytes: i32,
    pub bmPlanes: u16,
    pub bmBitsPixel: u16,
    pub bmBits: *mut c_void,
}

#[repr(C)]
pub struct MENUITEMINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub fType: u32,
    pub fState: u32,
    pub wID: u32,
    pub hSubMenu: HMENU,
    pub hbmpChecked: HBITMAP,
    pub hbmpUnchecked: HBITMAP,
    pub dwItemData: usize,
    pub dwTypeData: *mut u16,
    pub cch: u32,
    pub hbmpItem: HBITMAP,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt: POINT,
    pub lPrivate: u32,
}

pub type WNDPROC = Option<unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT>;

#[repr(C)]
pub struct WNDCLASSW {
    pub style: u32,
    pub lpfnWndProc: WNDPROC,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: HINSTANCE,
    pub hIcon: HICON,
    pub hCursor: HCURSOR,
    pub hbrBackground: HBRUSH,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
}

impl Default for WNDCLASSW {
    fn default() -> Self {
        Self {
            style: 0,
            lpfnWndProc: None,
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: 0,
            hIcon: 0,
            hCursor: 0,
            hbrBackground: 0,
            lpszMenuName: ptr::null(),
            lpszClassName: ptr::null(),
        }
    }
}

#[repr(C)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: u32,
    pub uFlags: u32,
    pub uCallbackMessage: u32,
    pub hIcon: HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uTimeoutOrVersion: u32,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: GUID,
    pub hBalloonIcon: HICON,
}

impl Default for NOTIFYICONDATAW {
    fn default() -> Self {
        Self {
            cbSize: size_of::<Self>() as u32,
            hWnd: 0,
            uID: 0,
            uFlags: 0,
            uCallbackMessage: 0,
            hIcon: 0,
            szTip: [0; 128],
            dwState: 0,
            dwStateMask: 0,
            szInfo: [0; 256],
            uTimeoutOrVersion: 0,
            szInfoTitle: [0; 64],
            dwInfoFlags: 0,
            guidItem: GUID::new(0, 0, 0, [0; 8]),
            hBalloonIcon: 0,
        }
    }
}

pub const WM_DESTROY: u32 = 0x0002;
pub const WM_DRAWITEM: u32 = 0x002B;
pub const WM_MEASUREITEM: u32 = 0x002C;
pub const WM_NULL: u32 = 0x0000;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_HOTKEY: u32 = 0x0312;
pub const WM_APP: u32 = 0x8000;
pub const WM_LBUTTONUP: u32 = 0x0202;
pub const WM_RBUTTONUP: u32 = 0x0205;

pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_WIN: u32 = 0x0008;
pub const MOD_NOREPEAT: u32 = 0x4000;

pub const VK_F1: u32 = 0x70;
pub const VK_F2: u32 = 0x71;
pub const VK_F3: u32 = 0x72;
pub const VK_F4: u32 = 0x73;
pub const VK_F5: u32 = 0x74;
pub const VK_F6: u32 = 0x75;
pub const VK_F7: u32 = 0x76;
pub const VK_F8: u32 = 0x77;
pub const VK_F9: u32 = 0x78;
pub const VK_F10: u32 = 0x79;
pub const VK_F11: u32 = 0x7A;
pub const VK_F12: u32 = 0x7B;

pub const MF_STRING: u32 = 0x0000;
pub const MF_GRAYED: u32 = 0x0001;
pub const MF_DISABLED: u32 = 0x0002;
pub const MF_CHECKED: u32 = 0x0008;
pub const MF_POPUP: u32 = 0x0010;
pub const MF_SEPARATOR: u32 = 0x0800;
pub const MIIM_FTYPE: u32 = 0x00000100;
pub const MIIM_ID: u32 = 0x00000002;
pub const MFT_OWNERDRAW: u32 = 0x00000100;
pub const ODT_MENU: u32 = 1;

pub const TPM_RIGHTBUTTON: u32 = 0x0002;
pub const TPM_NONOTIFY: u32 = 0x0080;
pub const TPM_RETURNCMD: u32 = 0x0100;

pub const NIM_ADD: u32 = 0x00000000;
pub const NIM_MODIFY: u32 = 0x00000001;
pub const NIM_DELETE: u32 = 0x00000002;
pub const NIF_MESSAGE: u32 = 0x00000001;
pub const NIF_ICON: u32 = 0x00000002;
pub const NIF_TIP: u32 = 0x00000004;
pub const NIF_INFO: u32 = 0x00000010;
pub const NIIF_INFO: u32 = 0x00000001;
pub const NIIF_WARNING: u32 = 0x00000002;
pub const NIIF_ERROR: u32 = 0x00000003;

pub const IDI_APPLICATION: u16 = 32512;
pub const IMAGE_ICON: u32 = 1;
pub const IMAGE_BITMAP: u32 = 0;
pub const LR_DEFAULTSIZE: u32 = 0x00000040;
pub const LR_SHARED: u32 = 0x00008000;
pub const LR_CREATEDIBSECTION: u32 = 0x00002000;
pub const SRCCOPY: u32 = 0x00CC0020;
pub const COLOR_MENU: i32 = 4;
pub const APP_ICON_RESOURCE_ID: u16 = 101;
pub const APP_BANNER_RESOURCE_ID: u16 = 102;

pub const MB_OK: u32 = 0x00000000;
pub const MB_ICONERROR: u32 = 0x00000010;
pub const MB_ICONINFORMATION: u32 = 0x00000040;

pub const STD_INPUT_HANDLE: i32 = -10;
pub const ENABLE_QUICK_EDIT_MODE: u32 = 0x0040;
pub const ENABLE_EXTENDED_FLAGS: u32 = 0x0080;

#[link(name = "user32")]
extern "system" {
    pub fn RegisterClassW(lpWndClass: *const WNDCLASSW) -> u16;
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        X: i32,
        Y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: HMENU,
        hInstance: HINSTANCE,
        lpParam: *mut c_void,
    ) -> HWND;
    pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> LRESULT;
    pub fn DestroyWindow(hWnd: HWND) -> BOOL;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32)
        -> BOOL;
    pub fn TranslateMessage(lpMsg: *const MSG) -> BOOL;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> LRESULT;
    pub fn RegisterHotKey(hWnd: HWND, id: i32, fsModifiers: u32, vk: u32) -> BOOL;
    pub fn UnregisterHotKey(hWnd: HWND, id: i32) -> BOOL;
    pub fn CreatePopupMenu() -> HMENU;
    pub fn AppendMenuW(hMenu: HMENU, uFlags: u32, uIDNewItem: usize, lpNewItem: *const u16)
        -> BOOL;
    pub fn InsertMenuItemW(
        hmenu: HMENU,
        item: u32,
        by_position: BOOL,
        info: *const MENUITEMINFOW,
    ) -> BOOL;
    pub fn DestroyMenu(hMenu: HMENU) -> BOOL;
    pub fn TrackPopupMenu(
        hMenu: HMENU,
        uFlags: u32,
        x: i32,
        y: i32,
        nReserved: i32,
        hWnd: HWND,
        prcRect: *const RECT,
    ) -> i32;
    pub fn GetCursorPos(lpPoint: *mut POINT) -> BOOL;
    pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
    pub fn LoadIconW(hInstance: HINSTANCE, lpIconName: *const u16) -> HICON;
    pub fn LoadImageW(
        hInst: HINSTANCE,
        name: *const u16,
        image_type: u32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> HANDLE;
    pub fn GetDpiForWindow(hwnd: HWND) -> u32;
    pub fn FillRect(hDC: HDC, rect: *const RECT, brush: HBRUSH) -> i32;
    pub fn GetSysColorBrush(index: i32) -> HBRUSH;
    pub fn MessageBoxW(hWnd: HWND, lpText: *const u16, lpCaption: *const u16, uType: u32) -> i32;
    pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: u32, lpTimerFunc: *const c_void)
        -> usize;
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> BOOL;
    pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;
}

#[link(name = "gdi32")]
extern "system" {
    pub fn CreateCompatibleDC(hdc: HDC) -> HDC;
    pub fn SelectObject(hdc: HDC, object: HGDIOBJ) -> HGDIOBJ;
    pub fn DeleteDC(hdc: HDC) -> BOOL;
    pub fn GetObjectW(object: HGDIOBJ, size: i32, data: *mut c_void) -> i32;
    pub fn DeleteObject(object: HGDIOBJ) -> BOOL;
    pub fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    pub fn StretchBlt(
        dst: HDC,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        src: HDC,
        sx: i32,
        sy: i32,
        sw: i32,
        sh: i32,
        rop: u32,
    ) -> BOOL;
}

pub const HALFTONE: i32 = 4;

#[link(name = "shell32")]
extern "system" {
    pub fn Shell_NotifyIconW(dwMessage: u32, lpData: *mut NOTIFYICONDATAW) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    pub fn GetModuleHandleW(lpModuleName: *const u16) -> HINSTANCE;
    pub fn GetStdHandle(nStdHandle: i32) -> HANDLE;
    pub fn GetConsoleMode(hConsoleHandle: HANDLE, lpMode: *mut u32) -> BOOL;
    pub fn SetConsoleMode(hConsoleHandle: HANDLE, dwMode: u32) -> BOOL;
}

pub fn to_wide<S: AsRef<OsStr>>(s: S) -> Vec<u16> {
    s.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn copy_wide<const N: usize>(dst: &mut [u16; N], text: &str) {
    dst.fill(0);
    let wide = OsStr::new(text).encode_wide();
    for (i, ch) in wide.take(N.saturating_sub(1)).enumerate() {
        dst[i] = ch;
    }
}

pub fn make_int_resource(id: u16) -> *const u16 {
    id as usize as *const u16
}

pub fn last_bool_error(operation: &str) -> String {
    format!("{operation} failed")
}

pub unsafe fn disable_quickedit() {
    let handle = GetStdHandle(STD_INPUT_HANDLE);
    if handle == 0 || handle == -1 {
        return;
    }

    let mut mode = 0u32;
    if GetConsoleMode(handle, &mut mode) == 0 {
        return;
    }

    let new_mode = (mode | ENABLE_EXTENDED_FLAGS) & !ENABLE_QUICK_EDIT_MODE;
    let _ = SetConsoleMode(handle, new_mode);
}
