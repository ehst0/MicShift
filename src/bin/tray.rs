#![windows_subsystem = "windows"]

use micshift::audio::{self, AudioDevice, ComGuard};
use micshift::config::{
    config_modified_time, load_config, save_config, AppConfig, HotkeyConfig, MicSelection,
};
use micshift::hotkeys::{self, HOTKEY_MIC1_ID, HOTKEY_MIC2_ID};
use micshift::presets::hotkey_presets;
use micshift::win32::*;
use std::process::Command;
use std::ptr::{null, null_mut};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;

const APP_NAME: &str = "MicShift";
const TRAY_ID: u32 = 1;
const WM_TRAYICON: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;

const CMD_OPEN_CONSOLE: u32 = 100;
const CMD_SWITCH_MIC1: u32 = 110;
const CMD_SWITCH_MIC2: u32 = 120;
const CMD_EXIT: u32 = 199;
const CMD_BANNER: u32 = 90;
const BANNER_WIDTH_DIP: u32 = 360;
const BANNER_HEIGHT_DIP: u32 = 90;

const CMD_MIC1_DEVICE_BASE: u32 = 1000;
const CMD_MIC2_DEVICE_BASE: u32 = 2000;
const CMD_MIC1_HOTKEY_BASE: u32 = 3000;
const CMD_MIC2_HOTKEY_BASE: u32 = 4000;

#[derive(Clone)]
struct AppState {
    hwnd: HWND,
    config: AppConfig,
    config_mtime: Option<SystemTime>,
    tray_added: bool,
}

static STATE: OnceLock<Mutex<AppState>> = OnceLock::new();

fn state() -> &'static Mutex<AppState> {
    STATE.get().expect("application state not initialized")
}

fn current_config() -> AppConfig {
    state().lock().unwrap().config.clone()
}

fn update_state_config(config: AppConfig) {
    let mut st = state().lock().unwrap();
    st.config = config;
    st.config_mtime = config_modified_time();
}

unsafe fn add_tray_icon(hwnd: HWND) -> Result<(), String> {
    let mut nid = NOTIFYICONDATAW::default();
    nid.hWnd = hwnd;
    nid.uID = TRAY_ID;
    nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    nid.uCallbackMessage = WM_TRAYICON;
    nid.hIcon = load_app_icon();
    copy_wide(&mut nid.szTip, APP_NAME);

    if Shell_NotifyIconW(NIM_ADD, &mut nid) == 0 {
        return Err("Could not create the system tray icon".to_string());
    }

    if let Some(lock) = STATE.get() {
        lock.lock().unwrap().tray_added = true;
    }
    Ok(())
}

unsafe fn load_app_icon() -> HICON {
    let instance = GetModuleHandleW(null());
    let icon = LoadImageW(
        instance,
        make_int_resource(APP_ICON_RESOURCE_ID),
        IMAGE_ICON,
        0,
        0,
        LR_DEFAULTSIZE | LR_SHARED,
    ) as HICON;
    if icon != 0 {
        icon
    } else {
        LoadIconW(0, make_int_resource(IDI_APPLICATION))
    }
}

unsafe fn append_banner(menu: HMENU) -> Result<(), String> {
    let info = MENUITEMINFOW {
        cbSize: std::mem::size_of::<MENUITEMINFOW>() as u32,
        fMask: MIIM_FTYPE | MIIM_ID,
        fType: MFT_OWNERDRAW,
        fState: 0,
        wID: CMD_BANNER,
        hSubMenu: 0,
        hbmpChecked: 0,
        hbmpUnchecked: 0,
        dwItemData: 0,
        dwTypeData: null_mut(),
        cch: 0,
        hbmpItem: 0,
    };
    if InsertMenuItemW(menu, 0, 1, &info) == 0 {
        return Err("Could not add the MicShift banner".to_string());
    }
    Ok(())
}

fn dpi_size(hwnd: HWND, dips: u32) -> u32 {
    let dpi = unsafe { GetDpiForWindow(hwnd) };
    dips.saturating_mul(if dpi == 0 { 96 } else { dpi })
        .saturating_add(48)
        / 96
}

unsafe fn measure_banner(hwnd: HWND, measure: *mut MEASUREITEMSTRUCT) -> bool {
    if measure.is_null() || (*measure).CtlType != ODT_MENU || (*measure).itemID != CMD_BANNER {
        return false;
    }
    (*measure).itemWidth = dpi_size(hwnd, BANNER_WIDTH_DIP);
    (*measure).itemHeight = dpi_size(hwnd, BANNER_HEIGHT_DIP);
    true
}

unsafe fn draw_banner(draw: *mut DRAWITEMSTRUCT) -> bool {
    if draw.is_null() || (*draw).CtlType != ODT_MENU || (*draw).itemID != CMD_BANNER {
        return false;
    }

    let d = &*draw;
    FillRect(d.hDC, &d.rcItem, GetSysColorBrush(COLOR_MENU));
    let bitmap = LoadImageW(
        GetModuleHandleW(null()),
        make_int_resource(APP_BANNER_RESOURCE_ID),
        IMAGE_BITMAP,
        0,
        0,
        LR_CREATEDIBSECTION,
    ) as HBITMAP;
    if bitmap == 0 {
        return true;
    }

    let mut details: BITMAP = std::mem::zeroed();
    if GetObjectW(
        bitmap,
        std::mem::size_of::<BITMAP>() as i32,
        &mut details as *mut _ as *mut _,
    ) != 0
    {
        let memory = CreateCompatibleDC(d.hDC);
        if memory != 0 {
            let old = SelectObject(memory, bitmap);
            SetStretchBltMode(d.hDC, HALFTONE);
            let width = d.rcItem.right - d.rcItem.left;
            let height = d.rcItem.bottom - d.rcItem.top;
            StretchBlt(
                d.hDC,
                d.rcItem.left,
                d.rcItem.top,
                width,
                height,
                memory,
                0,
                0,
                details.bmWidth,
                details.bmHeight,
                SRCCOPY,
            );
            SelectObject(memory, old);
            DeleteDC(memory);
        }
    }
    DeleteObject(bitmap);
    true
}

unsafe fn delete_tray_icon(hwnd: HWND) {
    let mut nid = NOTIFYICONDATAW::default();
    nid.hWnd = hwnd;
    nid.uID = TRAY_ID;
    let _ = Shell_NotifyIconW(NIM_DELETE, &mut nid);

    if let Some(lock) = STATE.get() {
        lock.lock().unwrap().tray_added = false;
    }
}

fn show_balloon(title: &str, message: &str, icon: u32) {
    let hwnd = match STATE.get() {
        Some(lock) => lock.lock().unwrap().hwnd,
        None => return,
    };

    unsafe {
        let mut nid = NOTIFYICONDATAW::default();
        nid.hWnd = hwnd;
        nid.uID = TRAY_ID;
        nid.uFlags = NIF_INFO;
        nid.uTimeoutOrVersion = 2000;
        nid.dwInfoFlags = icon;
        copy_wide(&mut nid.szInfoTitle, title);
        copy_wide(&mut nid.szInfo, message);
        let _ = Shell_NotifyIconW(NIM_MODIFY, &mut nid);
    }
}

fn show_error(message: &str) {
    let tray_available = STATE
        .get()
        .map(|s| s.lock().unwrap().tray_added)
        .unwrap_or(false);

    if tray_available {
        show_balloon(APP_NAME, message, NIIF_ERROR);
    } else {
        unsafe {
            let text = to_wide(message);
            let caption = to_wide(APP_NAME);
            let _ = MessageBoxW(0, text.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONERROR);
        }
    }
}

fn selected_name(selection: &Option<MicSelection>) -> &str {
    selection
        .as_ref()
        .map(|m| m.device_name.as_str())
        .unwrap_or("Not configured")
}

unsafe fn append_text(menu: HMENU, flags: u32, id: u32, text: &str) -> Result<(), String> {
    let wide = to_wide(text);
    if AppendMenuW(menu, flags, id as usize, wide.as_ptr()) == 0 {
        return Err("AppendMenuW failed".to_string());
    }
    Ok(())
}

unsafe fn append_separator(menu: HMENU) -> Result<(), String> {
    if AppendMenuW(menu, MF_SEPARATOR, 0, null()) == 0 {
        return Err("AppendMenuW separator failed".to_string());
    }
    Ok(())
}

unsafe fn append_popup(menu: HMENU, submenu: HMENU, text: &str) -> Result<(), String> {
    let wide = to_wide(text);
    if AppendMenuW(menu, MF_POPUP, submenu as usize, wide.as_ptr()) == 0 {
        return Err("AppendMenuW submenu failed".to_string());
    }
    Ok(())
}

unsafe fn build_device_submenu(
    devices: &[AudioDevice],
    selected: &Option<MicSelection>,
    command_base: u32,
) -> Result<HMENU, String> {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return Err("CreatePopupMenu failed".to_string());
    }

    if devices.is_empty() {
        append_text(
            menu,
            MF_STRING | MF_GRAYED | MF_DISABLED,
            0,
            "No active input devices",
        )?;
        return Ok(menu);
    }

    for (index, device) in devices.iter().enumerate() {
        let checked = selected
            .as_ref()
            .map(|s| s.device_id == device.id)
            .unwrap_or(false);
        let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
        append_text(menu, flags, command_base + index as u32, &device.name)?;
    }

    Ok(menu)
}

unsafe fn build_hotkey_submenu(
    presets: &[HotkeyConfig],
    selected: &HotkeyConfig,
    command_base: u32,
) -> Result<HMENU, String> {
    let menu = CreatePopupMenu();
    if menu == 0 {
        return Err("CreatePopupMenu failed".to_string());
    }

    for (index, preset) in presets.iter().enumerate() {
        let checked = preset.modifiers == selected.modifiers && preset.vk == selected.vk;
        let flags = MF_STRING | if checked { MF_CHECKED } else { 0 };
        append_text(menu, flags, command_base + index as u32, &preset.display)?;
    }

    Ok(menu)
}

unsafe fn build_mic_menu(
    slot: usize,
    config: &AppConfig,
    devices: &[AudioDevice],
    presets: &[HotkeyConfig],
) -> Result<(HMENU, String), String> {
    let (selection, hotkey, switch_cmd, device_base, hotkey_base) = if slot == 1 {
        (
            &config.mic1,
            &config.mic1_hotkey,
            CMD_SWITCH_MIC1,
            CMD_MIC1_DEVICE_BASE,
            CMD_MIC1_HOTKEY_BASE,
        )
    } else {
        (
            &config.mic2,
            &config.mic2_hotkey,
            CMD_SWITCH_MIC2,
            CMD_MIC2_DEVICE_BASE,
            CMD_MIC2_HOTKEY_BASE,
        )
    };

    let menu = CreatePopupMenu();
    if menu == 0 {
        return Err("CreatePopupMenu failed".to_string());
    }

    append_text(
        menu,
        MF_STRING,
        switch_cmd,
        &format!("Switch to Mic {slot} now"),
    )?;
    append_separator(menu)?;

    let device_menu = build_device_submenu(devices, selection, device_base)?;
    append_popup(menu, device_menu, "Select microphone")?;

    let hotkey_menu = build_hotkey_submenu(presets, hotkey, hotkey_base)?;
    append_popup(
        menu,
        hotkey_menu,
        &format!("Set hotkey — {}", hotkey.display),
    )?;

    let label = format!(
        "Mic {slot}: {}  [{}]",
        selected_name(selection),
        hotkey.display
    );
    Ok((menu, label))
}

fn launch_console_helper() -> Result<(), String> {
    let current = std::env::current_exe().map_err(|e| format!("Could not locate app: {e}"))?;
    let dir = current
        .parent()
        .ok_or_else(|| "Could not locate application directory".to_string())?;
    let helper = dir.join("MicShiftConsole.exe");

    if !helper.exists() {
        return Err(format!(
            "Console helper was not found next to the tray app: {}",
            helper.display()
        ));
    }

    Command::new(helper)
        .spawn()
        .map_err(|e| format!("Could not open console menu: {e}"))?;
    Ok(())
}

fn switch_slot(slot: usize) {
    let config = current_config();
    let selection = if slot == 1 {
        &config.mic1
    } else {
        &config.mic2
    };
    let Some(selection) = selection.clone() else {
        show_balloon(
            "Microphone not configured",
            &format!("Choose a device for Mic {slot} first."),
            NIIF_WARNING,
        );
        return;
    };

    match audio::find_active_device_by_id(&selection.device_id) {
        Ok(Some(device)) => match audio::set_default_capture_device(&device.id) {
            Ok(()) => show_balloon(
                "Microphone switched",
                &format!("Mic {slot}: {}", device.name),
                NIIF_INFO,
            ),
            Err(e) => show_error(&e),
        },
        Ok(None) => show_balloon(
            "Microphone unavailable",
            &format!("{} is disabled or disconnected.", selection.device_name),
            NIIF_WARNING,
        ),
        Err(e) => show_error(&e),
    }
}

fn set_mic_selection(slot: usize, device: &AudioDevice) {
    let mut config = current_config();
    let selection = MicSelection {
        device_id: device.id.clone(),
        device_name: device.name.clone(),
    };

    if slot == 1 {
        config.mic1 = Some(selection);
    } else {
        config.mic2 = Some(selection);
    }

    match save_config(&config) {
        Ok(()) => {
            update_state_config(config);
            switch_slot(slot);
        }
        Err(e) => show_error(&e),
    }
}

fn set_hotkey(hwnd: HWND, slot: usize, hotkey: &HotkeyConfig) {
    let old = current_config();
    let mut new = old.clone();

    if slot == 1 {
        new.mic1_hotkey = hotkey.clone();
    } else {
        new.mic2_hotkey = hotkey.clone();
    }

    if let Err(e) = hotkeys::register_config(hwnd, &new) {
        let _ = hotkeys::register_config(hwnd, &old);
        show_error(&e);
        return;
    }

    if let Err(e) = save_config(&new) {
        let _ = hotkeys::register_config(hwnd, &old);
        show_error(&e);
        return;
    }

    update_state_config(new);
    show_balloon(
        "Hotkey updated",
        &format!("Mic {slot}: {}", hotkey.display),
        NIIF_INFO,
    );
}

fn reload_config_if_changed(hwnd: HWND) {
    let disk_time = config_modified_time();
    let (known_time, old) = {
        let st = state().lock().unwrap();
        (st.config_mtime, st.config.clone())
    };

    if disk_time == known_time {
        return;
    }

    let new = match load_config() {
        Ok(c) => c,
        Err(e) => {
            show_error(&e);
            return;
        }
    };

    if new == old {
        state().lock().unwrap().config_mtime = disk_time;
        return;
    }

    let hotkeys_changed = new.mic1_hotkey != old.mic1_hotkey || new.mic2_hotkey != old.mic2_hotkey;
    if hotkeys_changed {
        if let Err(e) = hotkeys::register_config(hwnd, &new) {
            let _ = hotkeys::register_config(hwnd, &old);
            show_error(&format!(
                "Settings changed, but hotkeys could not be reloaded: {e}"
            ));
            state().lock().unwrap().config_mtime = disk_time;
            return;
        }
    }

    let mut st = state().lock().unwrap();
    st.config = new;
    st.config_mtime = disk_time;
}

fn dispatch_command(hwnd: HWND, command: u32, devices: &[AudioDevice], presets: &[HotkeyConfig]) {
    match command {
        CMD_OPEN_CONSOLE => {
            if let Err(e) = launch_console_helper() {
                show_error(&e);
            }
        }
        CMD_SWITCH_MIC1 => switch_slot(1),
        CMD_SWITCH_MIC2 => switch_slot(2),
        CMD_EXIT => unsafe {
            let _ = DestroyWindow(hwnd);
        },
        id if (CMD_MIC1_DEVICE_BASE..CMD_MIC2_DEVICE_BASE).contains(&id) => {
            let index = (id - CMD_MIC1_DEVICE_BASE) as usize;
            if let Some(device) = devices.get(index) {
                set_mic_selection(1, device);
            }
        }
        id if (CMD_MIC2_DEVICE_BASE..CMD_MIC1_HOTKEY_BASE).contains(&id) => {
            let index = (id - CMD_MIC2_DEVICE_BASE) as usize;
            if let Some(device) = devices.get(index) {
                set_mic_selection(2, device);
            }
        }
        id if (CMD_MIC1_HOTKEY_BASE..CMD_MIC2_HOTKEY_BASE).contains(&id) => {
            let index = (id - CMD_MIC1_HOTKEY_BASE) as usize;
            if let Some(hotkey) = presets.get(index) {
                set_hotkey(hwnd, 1, hotkey);
            }
        }
        id if id >= CMD_MIC2_HOTKEY_BASE => {
            let index = (id - CMD_MIC2_HOTKEY_BASE) as usize;
            if let Some(hotkey) = presets.get(index) {
                set_hotkey(hwnd, 2, hotkey);
            }
        }
        _ => {}
    }
}

unsafe fn show_context_menu(hwnd: HWND) {
    reload_config_if_changed(hwnd);

    let devices = match audio::enumerate_active_capture_devices() {
        Ok(d) => d,
        Err(e) => {
            show_error(&e);
            Vec::new()
        }
    };
    let presets = hotkey_presets();
    let config = current_config();

    let root = CreatePopupMenu();
    if root == 0 {
        show_error("Could not create tray menu");
        return;
    }

    let build_result = (|| -> Result<(), String> {
        append_banner(root)?;
        append_separator(root)?;
        append_text(root, MF_STRING, CMD_OPEN_CONSOLE, "Open Console Menu")?;
        append_separator(root)?;

        let (mic1_menu, mic1_label) = build_mic_menu(1, &config, &devices, &presets)?;
        append_popup(root, mic1_menu, &mic1_label)?;
        let (mic2_menu, mic2_label) = build_mic_menu(2, &config, &devices, &presets)?;
        append_popup(root, mic2_menu, &mic2_label)?;

        append_separator(root)?;
        append_text(root, MF_STRING, CMD_EXIT, "Exit")?;
        Ok(())
    })();

    if let Err(e) = build_result {
        let _ = DestroyMenu(root);
        show_error(&e);
        return;
    }

    let mut point = POINT::default();
    if GetCursorPos(&mut point) == 0 {
        let _ = DestroyMenu(root);
        return;
    }

    let _ = SetForegroundWindow(hwnd);
    let command = TrackPopupMenu(
        root,
        TPM_RIGHTBUTTON | TPM_NONOTIFY | TPM_RETURNCMD,
        point.x,
        point.y,
        0,
        hwnd,
        null(),
    );
    let _ = PostMessageW(hwnd, WM_NULL, 0, 0);
    let _ = DestroyMenu(root);

    if command > 0 {
        dispatch_command(hwnd, command as u32, &devices, &presets);
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_MEASUREITEM => {
            if measure_banner(hwnd, lparam as *mut MEASUREITEMSTRUCT) {
                1
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_DRAWITEM => {
            if draw_banner(lparam as *mut DRAWITEMSTRUCT) {
                1
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_TRAYICON => {
            let event = lparam as u32;
            if event == WM_LBUTTONUP || event == WM_RBUTTONUP {
                show_context_menu(hwnd);
            }
            0
        }
        WM_HOTKEY => {
            match wparam as i32 {
                HOTKEY_MIC1_ID => switch_slot(1),
                HOTKEY_MIC2_ID => switch_slot(2),
                _ => {}
            }
            0
        }
        WM_TIMER => {
            if wparam == TIMER_ID {
                reload_config_if_changed(hwnd);
            }
            0
        }
        WM_DESTROY => {
            let _ = KillTimer(hwnd, TIMER_ID);
            hotkeys::unregister_all(hwnd);
            delete_tray_icon(hwnd);
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn run() -> Result<(), String> {
    let _com = ComGuard::new()?;
    let config = load_config()?;

    unsafe {
        let hinstance = GetModuleHandleW(null());
        if hinstance == 0 {
            return Err("GetModuleHandleW failed".to_string());
        }

        let class_name = to_wide("MicShiftHiddenWindow");
        let window_name = to_wide(APP_NAME);
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            hIcon: load_app_icon(),
            lpszClassName: class_name.as_ptr(),
            ..WNDCLASSW::default()
        };

        if RegisterClassW(&wc) == 0 {
            return Err("RegisterClassW failed".to_string());
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            window_name.as_ptr(),
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            hinstance,
            null_mut(),
        );
        if hwnd == 0 {
            return Err("CreateWindowExW failed".to_string());
        }

        STATE
            .set(Mutex::new(AppState {
                hwnd,
                config: config.clone(),
                config_mtime: config_modified_time(),
                tray_added: false,
            }))
            .map_err(|_| "Application state was already initialized".to_string())?;

        add_tray_icon(hwnd)?;

        if let Err(e) = hotkeys::register_config(hwnd, &config) {
            show_balloon("Hotkey warning", &e, NIIF_WARNING);
        }

        if SetTimer(hwnd, TIMER_ID, 1000, null()) == 0 {
            show_balloon(
                "Settings monitor warning",
                "Could not start the settings reload timer.",
                NIIF_WARNING,
            );
        }

        if config.mic1.is_none() || config.mic2.is_none() {
            show_balloon(
                "Setup required",
                "Click the tray icon to select Mic 1 and Mic 2, or open the Console Menu.",
                NIIF_INFO,
            );
        }

        let mut msg = MSG::default();
        loop {
            let result = GetMessageW(&mut msg, 0, 0, 0);
            if result == 0 {
                break;
            }
            if result == -1 {
                return Err("GetMessageW failed".to_string());
            }
            let _ = TranslateMessage(&msg);
            let _ = DispatchMessageW(&msg);
        }
    }

    Ok(())
}

fn main() {
    if let Err(error) = run() {
        unsafe {
            let text = to_wide(&error);
            let caption = to_wide(APP_NAME);
            let _ = MessageBoxW(0, text.as_ptr(), caption.as_ptr(), MB_OK | MB_ICONERROR);
        }
    }
}
