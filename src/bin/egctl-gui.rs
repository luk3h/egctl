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
        summary: string,
        edited: bool,
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

    // Amber dot marking a button with unsaved changes.
    component Dot inherits Rectangle {
        width: 9px;
        height: 9px;
        border-radius: 4.5px;
        background: #e0b050;
    }

    // Top-down outline of the mouse. Drawn in a 200x320 grid, shown 1.4x larger.
    component MouseView inherits Rectangle {
        in-out property <int> selected;
        in property <[ButtonRow]> buttons;
        property <float> k: 1.4;

        // Which button the pointer is over (-1 = none). Smaller parts are checked first.
        property <int> hovered:
        t-up.has-hover ? 6 :
        t-down.has-hover ? 7 :
        t-middle.has-hover ? 2 :
        t-forward.has-hover ? 3 :
        t-back.has-hover ? 4 :
        t-left.has-hover ? 0 :
        t-right.has-hover ? 1 : -1;
        // Show the hovered button, or the selected one when nothing is hovered.
        property <int> shown: hovered >= 0 ? hovered : selected;

        width: 280px;
        height: 448px;

        // Body: narrower at the front, wider at the back
        Path {
            width: 280px;
            height: 448px;
            viewbox-width: 200;
            viewbox-height: 320;
            commands: "M100 8 C142 8 162 40 166 100 L178 215 C182 280 146 312 100 312 C54 312 18 280 22 215 L34 100 C38 40 58 8 100 8 Z";
            fill: #1d2126;
            stroke: #8a93a0;
            stroke-width: 1.5px;
        }

        // 0: Left
        Part {
            commands: "M98 10 C62 10 42 42 38 100 L35 128 Q64 138 98 141 Z";
            selected: root.selected == 0;
            hovered: root.hovered == 0;
        }
        // 1: Right
        Part {
            commands: "M102 10 C138 10 158 42 162 100 L165 128 Q136 138 102 141 Z";
            selected: root.selected == 1;
            hovered: root.hovered == 1;
        }
        // 6: Wheel up (above the wheel)
        Part {
            commands: "M94 24 L106 24 Q109 24 109 27 L109 37 Q109 40 106 40 L94 40 Q91 40 91 37 L91 27 Q91 24 94 24 Z";
            selected: root.selected == 6;
            hovered: root.hovered == 6;
        }
        // 2: Middle (scroll wheel)
        Part {
            commands: "M92 50 Q92 44 100 44 Q108 44 108 50 L108 78 Q108 84 100 84 Q92 84 92 78 Z";
            selected: root.selected == 2;
            hovered: root.hovered == 2;
        }
        // 7: Wheel down (below the wheel)
        Part {
            commands: "M94 88 L106 88 Q109 88 109 91 L109 101 Q109 104 106 104 L94 104 Q91 104 91 101 L91 91 Q91 88 94 88 Z";
            selected: root.selected == 7;
            hovered: root.hovered == 7;
        }
        // 3: Forward (front side button)
        Part {
            commands: "M22 116 Q16 116 16 122 L16 150 Q16 156 22 156 L30 156 L32 116 Z";
            selected: root.selected == 3;
            hovered: root.hovered == 3;
        }
        // 4: Back (rear side button)
        Part {
            commands: "M20 160 Q14 160 14 166 L14 194 Q14 200 20 200 L26 200 L28 160 Z";
            selected: root.selected == 4;
            hovered: root.hovered == 4;
        }

        // Arrows on the wheel up/down parts
        Text {
            x: 91px * root.k; y: 24px * root.k; width: 18px * root.k; height: 16px * root.k;
            text: "▲"; font-size: 9px; color: #c8cdd4;
            horizontal-alignment: center; vertical-alignment: center;
        }
        Text {
            x: 91px * root.k; y: 88px * root.k; width: 18px * root.k; height: 16px * root.k;
            text: "▼"; font-size: 9px; color: #c8cdd4;
            horizontal-alignment: center; vertical-alignment: center;
        }

        // Unsaved-change dots
        if root.buttons[0].edited : Dot { x: 68px * root.k - 4.5px; y: 70px * root.k - 4.5px; }
        if root.buttons[1].edited : Dot { x: 132px * root.k - 4.5px; y: 70px * root.k - 4.5px; }
        if root.buttons[2].edited : Dot { x: 115px * root.k - 4.5px; y: 64px * root.k - 4.5px; }
        if root.buttons[3].edited : Dot { x: 24px * root.k - 4.5px; y: 136px * root.k - 4.5px; }
        if root.buttons[4].edited : Dot { x: 21px * root.k - 4.5px; y: 180px * root.k - 4.5px; }
        if root.buttons[6].edited : Dot { x: 115px * root.k - 4.5px; y: 32px * root.k - 4.5px; }
        if root.buttons[7].edited : Dot { x: 115px * root.k - 4.5px; y: 96px * root.k - 4.5px; }

        // Mapping label in the palm area
        if root.shown >= 0 && root.shown < root.buttons.length : Text {
            x: 30px * root.k; y: 186px * root.k; width: 140px * root.k;
            text: root.buttons[root.shown].name;
            font-size: 16px; font-weight: 700; color: #e8ecf1;
            horizontal-alignment: center;
        }
        if root.shown >= 0 && root.shown < root.buttons.length : Text {
            x: 30px * root.k; y: 205px * root.k; width: 140px * root.k;
            text: root.buttons[root.shown].summary;
            font-size: 13px; color: #5ad16a;
            horizontal-alignment: center; wrap: word-wrap;
        }

        // Click areas. Later ones sit on top of earlier ones.
        t-left := TouchArea {
            x: 38px * root.k; y: 10px * root.k; width: 60px * root.k; height: 130px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 0; }
        }
        t-right := TouchArea {
            x: 102px * root.k; y: 10px * root.k; width: 60px * root.k; height: 130px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 1; }
        }
        t-middle := TouchArea {
            x: 90px * root.k; y: 42px * root.k; width: 20px * root.k; height: 44px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 2; }
        }
        t-up := TouchArea {
            x: 89px * root.k; y: 22px * root.k; width: 22px * root.k; height: 20px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 6; }
        }
        t-down := TouchArea {
            x: 89px * root.k; y: 86px * root.k; width: 22px * root.k; height: 20px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 7; }
        }
        t-forward := TouchArea {
            x: 12px * root.k; y: 114px * root.k; width: 22px * root.k; height: 44px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 3; }
        }
        t-back := TouchArea {
            x: 10px * root.k; y: 158px * root.k; width: 22px * root.k; height: 44px * root.k;
            mouse-cursor: pointer;
            clicked => { root.selected = 4; }
        }
    }

    export component MainWindow inherits Window {
        title: "egctl";
        preferred-width: 920px;
        preferred-height: 660px;

        in property <string> firmware;
        in-out property <string> status;
        in-out property <bool> status-error;
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
        in-out property <bool> dirty: false;

        callback refresh();
        callback apply();

        // Called whenever the user changes anything.
        function mark() {
            root.dirty = true;
            root.status-error = false;
        }

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
                    MouseView { selected <=> root.selected; buttons: root.buttons; }
                    Button {
                        text: root.buttons[5].edited ? "DPI button (underside)  •" : "DPI button (underside)";
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
                                                selected(v) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        if btn.kind == 0 : SettingRow {
                                            label: "Mouse button";
                                            ComboBox {
                                                model: ["left", "right", "middle", "back", "forward"];
                                                current-index <=> btn.mouse;
                                                selected(v) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        if btn.kind == 1 : SettingRow {
                                            label: "Direction";
                                            ComboBox {
                                                model: ["up", "down"];
                                                current-index <=> btn.scroll;
                                                selected(v) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        if btn.kind == 2 : SettingRow {
                                            label: "Key";
                                            LineEdit {
                                                placeholder-text: "e.g. f5 or ctrl+c";
                                                text <=> btn.key;
                                                edited(t) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        if btn.kind == 4 : SettingRow {
                                            label: "DPI while held";
                                            SpinBox {
                                                minimum: 50; maximum: 26000; step-size: 50;
                                                value <=> btn.cpi;
                                                edited(v) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        if btn.kind == 5 : SettingRow {
                                            label: "Media key";
                                            ComboBox {
                                                model: ["play-pause", "next", "previous", "mute", "volume-up", "volume-down", "browser", "explorer"];
                                                current-index <=> btn.media;
                                                selected(v) => { btn.edited = true; root.mark(); }
                                            }
                                        }
                                        SettingRow {
                                            label: "Debounce";
                                            SpinBox {
                                                minimum: 0; maximum: 25;
                                                enabled: btn.spdt == "off";
                                                value <=> btn.debounce;
                                                edited(v) => { btn.edited = true; root.mark(); }
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
                                    selected(v) => { root.mark(); }
                                }
                            }
                            SettingRow {
                                label: "Lift-off distance";
                                Text { text: root.lod-text + "  (read-only for now)"; vertical-alignment: center; }
                            }
                            Switch { text: "Angle snapping"; checked <=> root.angle-snapping; toggled => { root.mark(); } }
                            Switch { text: "Ripple control"; checked <=> root.ripple-control; toggled => { root.mark(); } }
                            Switch { text: "Motion sync"; checked <=> root.motion-sync; toggled => { root.mark(); } }
                            Switch { text: "Glass mode"; checked <=> root.glass-mode; toggled => { root.mark(); } }
                            Switch { text: "Force max fps"; checked <=> root.force-max-fps; toggled => { root.mark(); } }
                            Switch { text: "Slamclick filter"; checked <=> root.slamclick-filter; toggled => { root.mark(); } }
                            Switch { text: "Jitter filter"; checked <=> root.jitter-filter; toggled => { root.mark(); } }
                        }
                    }

                    Tab {
                        title: "CPI";
                        VerticalLayout {
                            padding: 16px;
                            spacing: 10px;
                            alignment: start;
                            SettingRow { label: "Active levels"; SpinBox { minimum: 1; maximum: 4; value <=> root.cpi-levels; edited(v) => { root.mark(); } } }
                            SettingRow { label: "CPI 1"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi1; edited(v) => { root.mark(); } } }
                            SettingRow { label: "CPI 2"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi2; edited(v) => { root.mark(); } } }
                            SettingRow { label: "CPI 3"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi3; edited(v) => { root.mark(); } } }
                            SettingRow { label: "CPI 4"; SpinBox { minimum: 50; maximum: 26000; step-size: 50; value <=> root.cpi4; edited(v) => { root.mark(); } } }
                        }
                    }
                }
            }

            // Bottom bar
            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: root.status-error ? root.status : (root.dirty ? "Unsaved changes" : root.status);
                    color: root.status-error ? #e05050 : (root.dirty ? #e0b050 : #50c060);
                    wrap: word-wrap;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                }
                Button { text: "Revert"; clicked => { root.refresh(); } }
                Button { text: "Apply"; primary: true; enabled: root.dirty; clicked => { root.apply(); } }
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
        summary: b.mapping.to_string().into(),
        edited: false,
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
            ui.set_dirty(false);
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
