use micshift::audio::{self, AudioDevice, ComGuard};
use micshift::config::{load_config, save_config, AppConfig, HotkeyConfig, MicSelection};
use micshift::presets::hotkey_presets;
use micshift::win32::disable_quickedit;
use std::io::{self, Write};

const APP_NAME: &str = "MicShift";

fn clear_screen() {
    // Keep this helper compatible with classic conhost as well as Windows Terminal.
    // A few blank lines are preferable to relying on ANSI mode being enabled.
    println!("\n\n");
}

fn read_line(prompt: &str) -> String {
    print!("{prompt}");
    let _ = io::stdout().flush();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    line.trim().to_string()
}

fn current_name(selection: &Option<MicSelection>) -> &str {
    selection
        .as_ref()
        .map(|m| m.device_name.as_str())
        .unwrap_or("Not configured")
}

fn pause() {
    let _ = read_line("\nPress Enter to continue...");
}

fn choose_device(devices: &[AudioDevice], current: &Option<MicSelection>) -> Option<AudioDevice> {
    if devices.is_empty() {
        println!("No active/connected input devices are available.");
        pause();
        return None;
    }

    println!("\nActive microphone/input devices:\n");
    for (index, device) in devices.iter().enumerate() {
        let mark = current
            .as_ref()
            .map(|c| c.device_id == device.id)
            .unwrap_or(false);
        println!(
            "{:>2}. {}{}",
            index + 1,
            device.name,
            if mark { "  [current]" } else { "" }
        );
    }

    loop {
        let input = read_line("\nChoose a device number, or Enter to cancel: ");
        if input.is_empty() {
            return None;
        }
        if let Ok(n) = input.parse::<usize>() {
            if (1..=devices.len()).contains(&n) {
                return Some(devices[n - 1].clone());
            }
        }
        println!("Please enter a number from 1 to {}.", devices.len());
    }
}

fn choose_hotkey(current: &HotkeyConfig) -> Option<HotkeyConfig> {
    let presets = hotkey_presets();
    println!("\nAvailable hotkeys:\n");

    for (index, hotkey) in presets.iter().enumerate() {
        let mark = hotkey.modifiers == current.modifiers && hotkey.vk == current.vk;
        println!(
            "{:>2}. {:<16}{}",
            index + 1,
            hotkey.display,
            if mark { " [current]" } else { "" }
        );
    }

    loop {
        let input = read_line("\nChoose a hotkey number, or Enter to cancel: ");
        if input.is_empty() {
            return None;
        }
        if let Ok(n) = input.parse::<usize>() {
            if (1..=presets.len()).contains(&n) {
                return Some(presets[n - 1].clone());
            }
        }
        println!("Please enter a number from 1 to {}.", presets.len());
    }
}

fn save_or_report(config: &AppConfig) -> bool {
    match save_config(config) {
        Ok(()) => {
            println!("\nSettings saved. The tray app will reload them automatically.");
            true
        }
        Err(e) => {
            println!("\nERROR: {e}");
            false
        }
    }
}

fn switch_slot(config: &AppConfig, slot: usize) {
    let selection = if slot == 1 {
        &config.mic1
    } else {
        &config.mic2
    };
    let Some(selection) = selection else {
        println!("\nMic {slot} is not configured.");
        pause();
        return;
    };

    match audio::find_active_device_by_id(&selection.device_id) {
        Ok(Some(device)) => match audio::set_default_capture_device(&device.id) {
            Ok(()) => println!("\nSwitched to Mic {slot}: {}", device.name),
            Err(e) => println!("\nERROR: {e}"),
        },
        Ok(None) => println!(
            "\n{} is currently disabled or disconnected.",
            selection.device_name
        ),
        Err(e) => println!("\nERROR: {e}"),
    }
    pause();
}

fn main() {
    unsafe { disable_quickedit() };

    let _com = match ComGuard::new() {
        Ok(g) => g,
        Err(e) => {
            eprintln!("{e}");
            pause();
            return;
        }
    };

    let mut config = match load_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            AppConfig::default()
        }
    };

    loop {
        clear_screen();
        println!("============================================================");
        println!(" {APP_NAME} - Console Menu");
        println!("============================================================\n");
        println!("Mic 1: {}", current_name(&config.mic1));
        println!("      Hotkey: {}", config.mic1_hotkey.display);
        println!();
        println!("Mic 2: {}", current_name(&config.mic2));
        println!("      Hotkey: {}", config.mic2_hotkey.display);
        println!("\n------------------------------------------------------------");
        println!("1. Select Mic 1");
        println!("2. Set Mic 1 hotkey");
        println!("3. Select Mic 2");
        println!("4. Set Mic 2 hotkey");
        println!("5. Switch to Mic 1 now");
        println!("6. Switch to Mic 2 now");
        println!("0. Close Console Menu");
        println!("------------------------------------------------------------");
        println!("Ctrl+C also closes this helper without stopping the tray app.\n");

        match read_line("Select: ").as_str() {
            "1" => match audio::enumerate_active_capture_devices() {
                Ok(devices) => {
                    if let Some(device) = choose_device(&devices, &config.mic1) {
                        config.mic1 = Some(MicSelection {
                            device_id: device.id,
                            device_name: device.name,
                        });
                        let _ = save_or_report(&config);
                        pause();
                    }
                }
                Err(e) => {
                    println!("\nERROR: {e}");
                    pause();
                }
            },
            "2" => {
                if let Some(hotkey) = choose_hotkey(&config.mic1_hotkey) {
                    config.mic1_hotkey = hotkey;
                    let _ = save_or_report(&config);
                    pause();
                }
            }
            "3" => match audio::enumerate_active_capture_devices() {
                Ok(devices) => {
                    if let Some(device) = choose_device(&devices, &config.mic2) {
                        config.mic2 = Some(MicSelection {
                            device_id: device.id,
                            device_name: device.name,
                        });
                        let _ = save_or_report(&config);
                        pause();
                    }
                }
                Err(e) => {
                    println!("\nERROR: {e}");
                    pause();
                }
            },
            "4" => {
                if let Some(hotkey) = choose_hotkey(&config.mic2_hotkey) {
                    config.mic2_hotkey = hotkey;
                    let _ = save_or_report(&config);
                    pause();
                }
            }
            "5" => switch_slot(&config, 1),
            "6" => switch_slot(&config, 2),
            "0" => break,
            _ => {}
        }

        if let Ok(new_config) = load_config() {
            config = new_config;
        }
    }

    println!("\nConsole menu closed. The tray app remains running.");
}
