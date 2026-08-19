<p align="center">
  <img src="resources/banner.png" width="720" alt="MicShift — microphone switcher">
</p>

<p align="center">
  A lightweight Windows tray utility for switching between microphones with global hotkeys.
</p>

## About

MicShift makes it quick to change the Windows default recording device without opening Sound settings. Assign two microphones, choose a hotkey for each, and switch from anywhere—even while a game or another application is focused.

Applications configured to use the Windows **Default** input device, including Discord, follow the selected microphone.

## Features

- Native Windows system-tray application
- Two configurable microphone slots
- Customizable global hotkeys
- Active-device filtering that hides disabled, disconnected, and unplugged inputs
- Immediate switching from either a hotkey or the tray menu
- Notification after a successful switch
- Persistent settings stored in `%APPDATA%\MicShift\config.json`
- Optional console interface that runs independently from the tray application
## Requirements

- Windows 10 or Windows 11
- Applications must use the Windows default input device to follow MicShift changes

## Build from source

Install the following:

- [Rust](https://www.rust-lang.org/tools/install) with the stable MSVC toolchain
- Visual Studio Build Tools with **Desktop development with C++** and a Windows SDK

Clone and build the project:

```powershell
git clone https://github.com/ehst0/MicShift.git
cd MicShift
cargo build --release
```

Cargo creates both executables in `target\release`:

```text
MicShift.exe
MicShiftConsole.exe
```

Keep the two files in the same directory. `MicShift.exe` is the main tray application; `MicShiftConsole.exe` is launched only when **Open Console Menu** is selected.

## Usage

1. Launch `MicShift.exe`.
2. Click the MicShift icon in the Windows notification area.
3. Open **Mic 1** and choose your first microphone.
4. Open **Mic 2** and choose your second microphone.
5. Choose a hotkey for each slot, or keep the defaults:
   - **Ctrl+F4** — Mic 1
   - **Ctrl+F5** — Mic 2
6. Set the input device in Discord or another application to **Default**.

MicShift continues running in the notification area. Select **Exit** from its menu to unregister the hotkeys and close it cleanly.

## Configuration

Settings are saved to:

```text
%APPDATA%\MicShift\config.json
```


## Technical note

MicShift uses Windows Core Audio to enumerate active capture endpoints and native `RegisterHotKey` registrations for global shortcuts. Switching the system default endpoint relies on the commonly used, undocumented `IPolicyConfig::SetDefaultEndpoint` COM interface because Windows does not provide a documented public setter. A future Windows update could require that isolated implementation to be revised.

## Contributing

Bug reports, feature suggestions, and pull requests are welcome. When reporting a problem, include your Windows version, the microphone devices involved, and steps that reproduce the issue.

## AI disclosure

MicShift was created collaboratively by [ehst0](https://github.com/ehst0) and OpenAI's ChatGPT/Codex. AI assistance was used during product design, Rust implementation, debugging, documentation, artwork generation, and build verification. The project is published openly so its source and behavior can be reviewed, tested, and improved by the community.

## License

MicShift is free software licensed under the [GNU General Public License v3.0](LICENSE) (`GPL-3.0-only`).
