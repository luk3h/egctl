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
    import { Button, ComboBox, Switch, SpinBox, LineEdit, TabWidget } from "std-widgets.slint";

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
        Text { text: root.label; width: 120px; color: #a0a0a0; vertical-alignment: center; }
        @children
    }

    // One clickable part of the mouse drawing.
    component Part inherits Path {
        in property <bool> selected;
        in property <bool> hovered;
        width: 280px;
        height: 448px;
        viewbox-width: 200;
        viewbox-height: 320;
        fill: selected ? #2e5c38 : (hovered ? #3a414a : #262b31);
        stroke: selected ? #5ad16a : #8a93a0;
        stroke-width: selected ? 2.5px : 1.5px;
    }

    // Top-down outline of the mouse. Drawn in a 200x320 grid, shown 1.4x larger.
    component MouseView inherits Rectangle {
        in-out property <int> selected;
        property <float> k: 1.4;
        width: 280px;
        height: 448px;

        // Body
        Path {
            width: 280px;
            height: 448px;
            viewbox-width: 200;
            viewbox-height: 320;
            commands: "M100 8 C150 8 172 40 174 100 L178 220 C180 280 145 312 100 312 C55 312 20 280 22 220 L26 100 C28 40 50 8 100 8 Z";
            fill: #1d2126;
            stroke: #8a93a0;
            stroke-width: 1.5px;
        }

        // 0: Left
        Part {
            commands: "M98 10 C55 10 30 42 28 100 L27 128 Q60 138 98 141 Z";
            selected: root.selected == 0;
            hovered: t-left.has-hover;
        }
        // 1: Right
        Part {
            commands: "M102 10 C145 10 170 42 172 100 L173 128 Q140 138 102 141 Z";
            selected: root.selected == 1;
            hovered: t-right.has-hover;
        }
        // 2: Middle (scroll wheel)
        Part {
            commands: "M92 42 Q92 36 100 36 Q108 36 108 42 L108 86 Q108 92 100 92 Q92 92 92 86 Z";
            selected: root.selected == 2;
            hovered: t-middle.has-hover;
        }
        // 6: Wheel up
        Part {
            commands: "M118 38 L134 38 Q137 38 137 41 L137 59 Q137 62 134 62 L118 62 Q115 62 115 59 L115 41 Q115 38 118 38 Z";
            selected: root.selected == 6;
            hovered: t-up.has-hover;
        }
        // 7: Wheel down
        Part {
            commands: "M118 66 L134 66 Q137 66 137 69 L137 87 Q137 90 134 90 L118 90 Q115 90 115 87 L115 69 Q115 66 118 66 Z";
            selected: root.selected == 7;
            hovered: t-down.has-hover;
        }
        // 3: Forward (front side button)
        Part {
            commands: "M14 116 Q8 116 8 122 L8 150 Q8 156 14 156 L24 156 L24 116 Z";
            selected: root.selected == 3;
            hovered: t-forward.has-hover;
        }
        // 4: Back (rear side button)
        Part {
            commands: "M14 160 Q8 160 8 166 L8 194 Q8 200 14 200 L24 200 L24 160 Z";
            selected: root.selected == 4;
            hovered: t-back.has-hover;
        }

        Text {
            x: 115px * root.k; y: 38px * root.k; width: 22px * root.k; height: 24px * root.k;
            text: "▲"; font-size: 11px; color: #c8cdd4;
            horizontal-alignment: center; vertical-alignment: center;
        }
        Text {
            x: 115px * root.k; y: 66px * root.k; width: 22px * root.k; height: 24px * root.k;
            text: "▼"; font-size: 11px; color: #c8cdd4;
            horizontal-alignment: center; vertical-alignment: center;
        }

        // Click areas. Later ones sit on top of earlier ones.
        t-left := TouchArea {
            x: 28px * root.k; y: 10px * root.k; width: 70px * root.k; height: 130px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 0; }
        }
        t-right := TouchArea {
            x: 102px * root.k; y: 10px * root.k; width: 70px * root.k; height: 130px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 1; }
        }
        t-middle := TouchArea {
            x: 90px * root.k; y: 34px * root.k; width: 20px * root.k; height: 60px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 2; }
        }
        t-up := TouchArea {
            x: 114px * root.k; y: 36px * root.k; width: 24px * root.k; height: 28px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 6; }
        }
        t-down := TouchArea {
            x: 114px * root.k; y: 64px * root.k; width: 24px * root.k; height: 28px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 7; }
        }
        t-forward := TouchArea {
            x: 4px * root.k; y: 114px * root.k; width: 22px * root.k; height: 44px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 3; }
        }
        t-back := TouchArea {
            x: 4px * root.k; y: 158px * root.k; width: 22px * root.k; height: 44px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 4; }
        }
    }

    export component MainWindow inherits Window {
        title: "egctl";
        preferred-width: 920px;
        preferred-height: 660px;

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
        in-out property <int> selected: 0;

        callback refresh();
        callback apply();

        VerticalLayout {
            padding: 16px;
            spacing: 12px;

            VerticalLayout {
                Text { text: "Endgame Gear OP1 8k v2"; font-size: 20px; font-weight: 700; }
                Text { text: root.firmware; color: #a0a0a0; }
            }

            HorizontalLayout {
                spacing: 24px;
                vertical-stretch: 1;

                // Left: the mouse
                VerticalLayout {
                    alignment: start;
                    spacing: 10px;
                    MouseView { selected <=> root.selected; }
                    Button {
                        text: "DPI button (underside)";
                        primary: root.selected == 5;
                        clicked => { root.selected = 5; }
                    }
                    Text {
                        text: "Click a button on the mouse to edit it";
                        color: #a0a0a0;
                        horizontal-alignment: center;
                    }
                }

                // Right: settings
                TabWidget {
                    horizontal-stretch: 1;

                    Tab {
                        title: "Buttons";
                        VerticalLayout {
                            padding: 16px;
                            spacing: 12px;
                            alignment: start;

                            // Only the selected button's editor is shown.
                            VerticalLayout {
                                spacing: 0px;
                                for btn[i] in root.buttons : VerticalLayout {
                                    if i == root.selected : VerticalLayout {
                                        spacing: 10px;
                                        Text { text: btn.name; font-size: 18px; font-weight: 700; }
                                        SettingRow {
                                            label: "Action";
                                            ComboBox {
                                                model: ["Mouse", "Scroll", "Key", "CPI cycle", "CPI shift", "Media", "Disabled"];
                                                current-index <=> btn.kind;
                                            }
                                        }
                                        if btn.kind == 0 : SettingRow {
                                            label: "Mouse button";
                                            ComboBox {
                                                model: ["left", "right", "middle", "back", "forward"];
                                                current-index <=> btn.mouse;
                                            }
                                        }
                                        if btn.kind == 1 : SettingRow {
                                            label: "Direction";
                                            ComboBox { model: ["up", "down"]; current-index <=> btn.scroll; }
                                        }
                                        if btn.kind == 2 : SettingRow {
                                            label: "Key";
                                            LineEdit { placeholder-text: "e.g. f5 or ctrl+c"; text <=> btn.key; }
                                        }
                                        if btn.kind == 4 : SettingRow {
                                            label: "DPI while held";
                                            SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> btn.cpi; }
                                        }
                                        if btn.kind == 5 : SettingRow {
                                            label: "Media key";
                                            ComboBox {
                                                model: ["play-pause", "next", "previous", "mute", "volume-up", "volume-down", "browser", "explorer"];
                                                current-index <=> btn.media;
                                            }
                                        }
                                        SettingRow {
                                            label: "Debounce";
                                            SpinBox {
                                                minimum: 0; maximum: 25;
                                                enabled: btn.spdt == "off";
                                                value <=> btn.debounce;
                                            }
                                        }
                                        if btn.spdt != "off" : Text {
                                            text: "Uses SPDT mode (" + btn.spdt + "), so debounce doesn't apply.";
                                            color: #a0a0a0;
                                            wrap: word-wrap;
                                        }
                                    }
                                }
                            }

                            if root.selected == 0 : Switch {
                                text: "Allow remapping the left button";
                                checked <=> root.allow-left-remap;
                            }
                        }
                    }

                    Tab {
                        title: "Performance";
                        VerticalLayout {
                            padding: 16px;
                            spacing: 10px;
                            alignment: start;
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

                    Tab {
                        title: "CPI";
                        VerticalLayout {
                            padding: 16px;
                            spacing: 10px;
                            alignment: start;
                            SettingRow { label: "Active levels"; SpinBox { minimum: 1; maximum: 4; value <=> root.cpi-levels; } }
                            SettingRow { label: "CPI 1"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi1; } }
                            SettingRow { label: "CPI 2"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi2; } }
                            SettingRow { label: "CPI 3"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi3; } }
                            SettingRow { label: "CPI 4"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi4; } }
                        }
                    }
                }
            }

            // Bottom bar
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
    slint::set_xdg_app_id("egctl")?;
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
