use crate::config::AppConfig;
use crate::win32::{RegisterHotKey, UnregisterHotKey, HWND, MOD_NOREPEAT};

pub const HOTKEY_MIC1_ID: i32 = 1;
pub const HOTKEY_MIC2_ID: i32 = 2;

pub fn unregister_all(hwnd: HWND) {
    unsafe {
        let _ = UnregisterHotKey(hwnd, HOTKEY_MIC1_ID);
        let _ = UnregisterHotKey(hwnd, HOTKEY_MIC2_ID);
    }
}

pub fn register_config(hwnd: HWND, config: &AppConfig) -> Result<(), String> {
    unregister_all(hwnd);

    unsafe {
        if RegisterHotKey(
            hwnd,
            HOTKEY_MIC1_ID,
            config.mic1_hotkey.modifiers | MOD_NOREPEAT,
            config.mic1_hotkey.vk,
        ) == 0
        {
            return Err(format!(
                "Could not register {} for Mic 1. Another program may already use it.",
                config.mic1_hotkey.display
            ));
        }

        if RegisterHotKey(
            hwnd,
            HOTKEY_MIC2_ID,
            config.mic2_hotkey.modifiers | MOD_NOREPEAT,
            config.mic2_hotkey.vk,
        ) == 0
        {
            let _ = UnregisterHotKey(hwnd, HOTKEY_MIC1_ID);
            return Err(format!(
                "Could not register {} for Mic 2. Another program may already use it.",
                config.mic2_hotkey.display
            ));
        }
    }

    Ok(())
}
