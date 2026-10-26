use egctl::device::Device;
use egctl::types::config::BUTTON_NAMES;
use egctl::types::{validate_dpi, MouseConfig, PollingRate, SpdtMode};

slint::slint! {
    import { Button, GroupBox, VerticalBox, ComboBox, Switch, SpinBox } from "std-widgets.slint";

    export struct Row {
        label: string,
        value: string,
    }

    component RowList inherits VerticalLayout {
        in property <[Row]> rows;
        spacing: 4px;
        for row in root.rows : HorizontalLayout {
            Text { text: row.label; width: 140px; color: #a0a0a0; }
            Text { text: row.value; }
        }
    }

    component SettingRow inherits HorizontalLayout {
        in property <string> label;
        spacing: 8px;
        Text { text: root.label; width: 140px; color: #a0a0a0; vertical-alignment: center; }
        @children
    }

    export component MainWindow inherits Window {
        title: "egctl";
        preferred-width: 460px;
        preferred-height: 900px;

        in property <string> firmware;
        in property <string> status;
        in property <bool> status-error;
        in property <string> lod-text;
        in property <[Row]> buttons;

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

        callback refresh();
        callback apply();

        VerticalBox {
            HorizontalLayout {
                spacing: 8px;
                Text {
                    text: "Endgame Gear OP1 8k v2";
                    font-size: 18px;
                    font-weight: 700;
                    vertical-alignment: center;
                    horizontal-stretch: 1;
                }
            }
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
                title: "Buttons (read-only for now)";
                RowList { rows: root.buttons; }
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

/// Read the mouse. Read-only.
fn read_mouse() -> anyhow::Result<(String, MouseConfig)> {
    let dev = Device::open()?;
    let (major, minor) = dev.get_firmware_version()?;
    let config = dev.read_config()?;
    Ok((format!("Firmware {minor:x}.{major:x}"), config))
}

fn buttons_model(c: &MouseConfig) -> slint::ModelRc<Row> {
    let rows: Vec<Row> = c
    .buttons
    .iter()
    .zip(BUTTON_NAMES.iter())
    .map(|(b, name)| {
        let mut value = b.mapping.to_string();
        if b.spdt != SpdtMode::Off {
            value += &format!("  ·  SPDT {}", b.spdt);
        } else if b.multiclick > 0 {
            value += &format!("  ·  debounce {}", b.multiclick);
        }
        Row {
            label: (*name).into(),
         value: value.into(),
        }
    })
    .collect();
    slint::ModelRc::new(slint::VecModel::from(rows))
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
    ui.set_buttons(buttons_model(c));
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

/// Read fresh, change only what the window shows, write back.
fn apply(ui: &MainWindow) -> anyhow::Result<()> {
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
                    ui.set_status(format!("Save failed: {e:#}").into());
                }
            }
        }
    });

    ui.run()
}
