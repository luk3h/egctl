use anyhow::Context;
use slint::Model;

use egctl::device::Device;
use egctl::types::config::{ButtonConfig, BUTTON_NAMES};
use egctl::types::keys::format_key;
use egctl::types::{
    key_code, modifier_bit, validate_dpi, MappingType, MediaKey, MouseConfig, MouseKey,
    PollingRate, ScrollDirection, SpdtMode,
};

slint::slint! {
    import { Button, GroupBox, VerticalBox, ComboBox, Switch, SpinBox, LineEdit } from "std-widgets.slint";

    export struct ButtonRow {
        name: string,
        kind: int,
        mouse: int,
        scroll: int,
        key: string,
        cpi: int,
        media: int,
        debounce: int,
        spdt: string,
    }

    component SettingRow inherits HorizontalLayout {
        in property <string> label;
        spacing: 8px;
        Text { text: root.label; width: 140px; color: #a0a0a0; vertical-alignment: center; }
        @children
    }

    export component MainWindow inherits Window {
        title: "egctl";
        preferred-width: 640px;
        preferred-height: 1000px;

        in property <string> firmware;
        in property <string> status;
        in property <bool> status-error;
        in property <string> lod-text;

        in-out property <int> polling-index;
        in-out property <bool> angle-snapping;
        in-out property <bool> ripple-control;
        in-out property <bool> motion-sync;
        in-out property <bool> glass-mode;
        in-out property <bool> force-max-fps;
        in-out property <bool> slamclick-filter;
        in-out property <bool> jitter-filter;

        in-out property <int> cpi-levels;
        in-out property <int> cpi1;
        in-out property <int> cpi2;
        in-out property <int> cpi3;
        in-out property <int> cpi4;

        in-out property <[ButtonRow]> buttons;
        in-out property <bool> allow-left-remap;

        callback refresh();
        callback apply();

        VerticalBox {
            Text { text: "Endgame Gear OP1 8k v2"; font-size: 18px; font-weight: 700; }
            Text { text: root.firmware; color: #a0a0a0; }

            GroupBox {
                title: "General";
                VerticalLayout {
                    spacing: 6px;
                    SettingRow {
                        label: "Polling rate";
                        ComboBox {
                            model: ["1000 Hz", "2000 Hz", "4000 Hz", "8000 Hz"];
                            current-index <=> root.polling-index;
                        }
                    }
                    SettingRow {
                        label: "Lift-off distance";
                        Text { text: root.lod-text + "  (read-only for now)"; vertical-alignment: center; }
                    }
                    Switch { text: "Angle snapping"; checked <=> root.angle-snapping; }
                    Switch { text: "Ripple control"; checked <=> root.ripple-control; }
                    Switch { text: "Motion sync"; checked <=> root.motion-sync; }
                    Switch { text: "Glass mode"; checked <=> root.glass-mode; }
                    Switch { text: "Force max fps"; checked <=> root.force-max-fps; }
                    Switch { text: "Slamclick filter"; checked <=> root.slamclick-filter; }
                    Switch { text: "Jitter filter"; checked <=> root.jitter-filter; }
                }
            }

            GroupBox {
                title: "CPI levels";
                VerticalLayout {
                    spacing: 6px;
                    SettingRow { label: "Active levels"; SpinBox { minimum: 1; maximum: 4; value <=> root.cpi-levels; } }
                    SettingRow { label: "CPI 1"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi1; } }
                    SettingRow { label: "CPI 2"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi2; } }
                    SettingRow { label: "CPI 3"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi3; } }
                    SettingRow { label: "CPI 4"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi4; } }
                }
            }

            GroupBox {
                title: "Buttons";
                VerticalLayout {
                    spacing: 6px;
                    HorizontalLayout {
                        spacing: 8px;
                        Text { text: "Button"; width: 90px; color: #a0a0a0; }
                        Text { text: "Action"; width: 130px; color: #a0a0a0; }
                        Text { text: "Setting"; horizontal-stretch: 1; color: #a0a0a0; }
                        Text { text: "Debounce"; width: 90px; color: #a0a0a0; }
                    }
                    for btn in root.buttons : HorizontalLayout {
                        spacing: 8px;
                        Text { text: btn.name; width: 90px; vertical-alignment: center; }
                        ComboBox {
                            width: 130px;
                            model: ["Mouse", "Scroll", "Key", "CPI cycle", "CPI shift", "Media", "Disabled"];
                            current-index <=> btn.kind;
                        }
                        HorizontalLayout {
                            horizontal-stretch: 1;
                            if btn.kind == 0 : ComboBox {
                                model: ["left", "right", "middle", "back", "forward"];
                                current-index <=> btn.mouse;
                            }
                            if btn.kind == 1 : ComboBox {
                                model: ["up", "down"];
                                current-index <=> btn.scroll;
                            }
                            if btn.kind == 2 : LineEdit {
                                placeholder-text: "e.g. f5 or ctrl+c";
                                text <=> btn.key;
                            }
                            if btn.kind == 4 : SpinBox {
                                minimum: 50; maximum: 26000; step-size: 50;
                                value <=> btn.cpi;
                            }
                            if btn.kind == 5 : ComboBox {
                                model: ["play-pause", "next", "previous", "mute", "volume-up", "volume-down", "browser", "explorer"];
                                current-index <=> btn.media;
                            }
                            if btn.kind == 3 || btn.kind == 6 : Text { text: "—"; vertical-alignment: center; }
                        }
                        SpinBox {
                            width: 90px;
                            minimum: 0; maximum: 25;
                            enabled: btn.spdt == "off";
                            value <=> btn.debounce;
                        }
                    }
                    Switch { text: "Allow remapping the left button"; checked <=> root.allow-left-remap; }
                }
            }

            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: root.status;
                    color: root.status-error ? #e05050 : #50c060;
                    wrap: word-wrap;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                }
                Button { text: "Revert"; clicked => { root.refresh(); } }
                Button { text: "Apply"; primary: true; clicked => { root.apply(); } }
            }
        }
    }
}

// Order must match the dropdowns in the UI above.
const MOUSE_KEYS: [MouseKey; 5] = [
    MouseKey::Left,
MouseKey::Right,
MouseKey::Middle,
MouseKey::Back,
MouseKey::Forward,
];
const MEDIA_KEYS: [MediaKey; 8] = [
    MediaKey::PlayPause,
MediaKey::Next,
MediaKey::Previous,
MediaKey::Mute,
MediaKey::VolumeUp,
MediaKey::VolumeDown,
MediaKey::Browser,
MediaKey::Explorer,
];

/// Read the mouse. Read-only.
fn read_mouse() -> anyhow::Result<(String, MouseConfig)> {
    let dev = Device::open()?;
    let (major, minor) = dev.get_firmware_version()?;
    let config = dev.read_config()?;
    Ok((format!("Firmware {minor:x}.{major:x}"), config))
}

/// Turn one button's config into a row for the window.
fn to_row(name: &str, b: &ButtonConfig) -> ButtonRow {
    let mut row = ButtonRow {
        name: name.into(),
        kind: 6,
        mouse: 0,
        scroll: 0,
        key: "".into(),
        cpi: 800,
        media: 0,
        debounce: b.multiclick as i32,
        spdt: b.spdt.to_string().into(),
    };
    match b.mapping {
        MappingType::Mouse(k) => {
            row.kind = 0;
            row.mouse = MOUSE_KEYS.iter().position(|m| *m == k).unwrap_or(0) as i32;
        }
        MappingType::Scroll(d) => {
            row.kind = 1;
            row.scroll = if d == ScrollDirection::Down { 1 } else { 0 };
        }
        MappingType::Keyboard { modifiers, code } => {
            row.kind = 2;
            row.key = format_key(modifiers, code).into();
        }
        MappingType::CpiLoop => row.kind = 3,
        MappingType::Cpi { x, .. } => {
            row.kind = 4;
            row.cpi = x as i32;
        }
        MappingType::Media(k) => {
            row.kind = 5;
            row.media = MEDIA_KEYS.iter().position(|m| *m == k).unwrap_or(0) as i32;
        }
        MappingType::Disable => row.kind = 6,
    }
    row
}

/// Parse text like "f5", "ctrl+c" or "ctrl+shift+esc" into (modifiers, key code).
fn parse_key(s: &str) -> anyhow::Result<(u8, u8)> {
    let mut modifiers = 0u8;
    let mut code = 0u8;
    for part in s.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        if let Some(bit) = modifier_bit(part) {
            modifiers |= bit;
        } else if let Some(c) = key_code(part) {
            if code != 0 {
                anyhow::bail!("only one non-modifier key allowed");
            }
            code = c;
        } else {
            anyhow::bail!("unknown key '{part}' (run `egctl keys` for the list)");
        }
    }
    if modifiers == 0 && code == 0 {
        anyhow::bail!("no key entered");
    }
    Ok((modifiers, code))
}

/// Turn a row from the window back into a button mapping.
fn mapping_from_row(r: &ButtonRow) -> anyhow::Result<MappingType> {
    Ok(match r.kind {
        0 => MappingType::Mouse(MOUSE_KEYS[r.mouse.clamp(0, 4) as usize]),
       1 => MappingType::Scroll(if r.scroll == 1 {
           ScrollDirection::Down
       } else {
           ScrollDirection::Up
       }),
       2 => {
           let (modifiers, code) = parse_key(&r.key)?;
           MappingType::Keyboard { modifiers, code }
       }
       3 => MappingType::CpiLoop,
       4 => {
           let dpi = validate_dpi(r.cpi.clamp(50, 26000) as u16)?;
           MappingType::Cpi {
               xy_split: false,
               x: dpi,
               y: dpi,
           }
       }
       5 => MappingType::Media(MEDIA_KEYS[r.media.clamp(0, 7) as usize]),
       _ => MappingType::Disable,
    })
}

/// Copy the mouse's settings into the window.
fn fill(ui: &MainWindow, firmware: String, c: &MouseConfig) {
    ui.set_firmware(firmware.into());
    ui.set_polling_index(match c.polling_rate {
        PollingRate::Hz1000 => 0,
        PollingRate::Hz2000 => 1,
        PollingRate::Hz4000 => 2,
        PollingRate::Hz8000 => 3,
    });
    ui.set_lod_text(c.lod.to_string().into());
    ui.set_angle_snapping(c.angle_snapping);
    ui.set_ripple_control(c.ripple_control);
    ui.set_motion_sync(c.motion_sync);
    ui.set_glass_mode(c.glass_mode);
    ui.set_force_max_fps(c.force_max_fps);
    ui.set_slamclick_filter(c.slamclick_filter);
    ui.set_jitter_filter(c.jitter_filter);
    ui.set_cpi_levels(c.cpi_levels as i32);
    ui.set_cpi1(c.cpis[0].x as i32);
    ui.set_cpi2(c.cpis[1].x as i32);
    ui.set_cpi3(c.cpis[2].x as i32);
    ui.set_cpi4(c.cpis[3].x as i32);

    let rows: Vec<ButtonRow> = c
    .buttons
    .iter()
    .zip(BUTTON_NAMES.iter())
    .map(|(b, name)| to_row(name, b))
    .collect();
    ui.set_buttons(slint::ModelRc::new(slint::VecModel::from(rows)));
}

/// Reload from the mouse. Returns true if it worked.
fn load(ui: &MainWindow) -> bool {
    match read_mouse() {
        Ok((firmware, config)) => {
            fill(ui, firmware, &config);
            ui.set_status_error(false);
            ui.set_status("Loaded from mouse".into());
            true
        }
        Err(e) => {
            ui.set_status_error(true);
            ui.set_status(format!("{e:#}").into());
            false
        }
    }
}

/// Validate everything, then read fresh, change only what the window shows, write back.
fn apply(ui: &MainWindow) -> anyhow::Result<()> {
    // 1. Check every button row BEFORE touching the mouse.
    let model = ui.get_buttons();
    let mut new_buttons = Vec::new();
    for i in 0..model.row_count() {
        let r = model.row_data(i).context("missing button row")?;
        let mapping = mapping_from_row(&r).with_context(|| format!("{} button", r.name))?;
        new_buttons.push((mapping, r.debounce));
    }

    // 2. Safety lock: don't let the left button stop being left-click by accident.
    let left = new_buttons.first().map(|(m, _)| *m);
    if !ui.get_allow_left_remap() && left != Some(MappingType::Mouse(MouseKey::Left)) {
        anyhow::bail!(
            "Left button is no longer left-click. Tick \"Allow remapping the left button\" if you really mean it."
        );
    }

    // 3. Read fresh and apply.
    let dev = Device::open()?;
    let mut c = dev.read_config()?;

    c.polling_rate = match ui.get_polling_index() {
        1 => PollingRate::Hz2000,
        2 => PollingRate::Hz4000,
        3 => PollingRate::Hz8000,
        _ => PollingRate::Hz1000,
    };
    c.angle_snapping = ui.get_angle_snapping();
    c.ripple_control = ui.get_ripple_control();
    c.motion_sync = ui.get_motion_sync();
    c.glass_mode = ui.get_glass_mode();
    c.force_max_fps = ui.get_force_max_fps();
    c.slamclick_filter = ui.get_slamclick_filter();
    c.jitter_filter = ui.get_jitter_filter();

    c.cpi_levels = ui.get_cpi_levels().clamp(1, 4) as u8;
    let values = [ui.get_cpi1(), ui.get_cpi2(), ui.get_cpi3(), ui.get_cpi4()];
    for (i, v) in values.iter().enumerate() {
        // Levels with separate X/Y values aren't editable here yet, so leave them alone.
        if c.cpis[i].xy_split {
            continue;
        }
        let dpi = validate_dpi((*v).clamp(50, 26000) as u16)?;
        c.cpis[i].x = dpi;
        c.cpis[i].y = dpi;
    }

    for (i, (mapping, debounce)) in new_buttons.into_iter().enumerate().take(c.buttons.len()) {
        c.buttons[i].mapping = mapping;
        if c.buttons[i].spdt == SpdtMode::Off {
            c.buttons[i].multiclick = debounce.clamp(0, 25) as u8;
        }
    }

    dev.write_config(&c)?;
    Ok(())
}

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    load(&ui);

    let weak = ui.as_weak();
    ui.on_refresh(move || {
        if let Some(ui) = weak.upgrade() {
            load(&ui);
        }
    });

    let weak = ui.as_weak();
    ui.on_apply(move || {
        if let Some(ui) = weak.upgrade() {
            match apply(&ui) {
                Ok(()) => {
                    if load(&ui) {
                        ui.set_status("Saved to mouse".into());
                    }
                }
                Err(e) => {
                    ui.set_status_error(true);
                    ui.set_status(format!("Not saved: {e:#}").into());
                }
            }
        }
    });

    ui.run()
}
