use crate::config::HotkeyConfig;
use crate::win32::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, VK_F1};

pub fn hotkey_presets() -> Vec<HotkeyConfig> {
    let mut result = Vec::new();
    let groups = [
        (MOD_CONTROL, "Ctrl"),
        (MOD_CONTROL | MOD_SHIFT, "Ctrl+Shift"),
        (MOD_ALT, "Alt"),
        (MOD_CONTROL | MOD_ALT, "Ctrl+Alt"),
    ];

    for (mods, prefix) in groups {
        for n in 1..=11u32 {
            result.push(HotkeyConfig {
                modifiers: mods,
                vk: VK_F1 + (n - 1),
                display: format!("{prefix}+F{n}"),
            });
        }
    }

    result
}
