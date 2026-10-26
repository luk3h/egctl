use egctl::device::Device;
use egctl::types::config::BUTTON_NAMES;
use egctl::types::{on_off, SpdtMode};

slint::slint! {
    import { Button, GroupBox, VerticalBox } from "std-widgets.slint";

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

    export component MainWindow inherits Window {
        title: "egctl";
        preferred-width: 440px;
        preferred-height: 720px;

        in property <string> firmware;
        in property <string> status;
        in property <[Row]> general;
        in property <[Row]> cpis;
        in property <[Row]> buttons;
        callback refresh();

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
                Button { text: "Refresh"; clicked => { root.refresh(); } }
            }
            Text { text: root.firmware; color: #a0a0a0; }
            if root.status != "" : Text { text: root.status; color: #e05050; wrap: word-wrap; }
            GroupBox { title: "General"; RowList { rows: root.general; } }
            GroupBox { title: "CPI levels"; RowList { rows: root.cpis; } }
            GroupBox { title: "Buttons"; RowList { rows: root.buttons; } }
        }
    }
}

struct Snapshot {
    firmware: String,
    general: Vec<(String, String)>,
    cpis: Vec<(String, String)>,
    buttons: Vec<(String, String)>,
}

/// Read everything from the mouse. Read-only: nothing here writes to the device.
fn read_mouse() -> anyhow::Result<Snapshot> {
    let dev = Device::open()?;
    let (major, minor) = dev.get_firmware_version()?;
    let c = dev.read_config()?;

    let general: Vec<(String, String)> = vec![
        ("Polling rate".into(), c.polling_rate.to_string()),
        ("Lift-off distance".into(), c.lod.to_string()),
        ("Angle snapping".into(), on_off(c.angle_snapping).into()),
        ("Ripple control".into(), on_off(c.ripple_control).into()),
        ("Motion sync".into(), on_off(c.motion_sync).into()),
        ("Glass mode".into(), on_off(c.glass_mode).into()),
        ("Force max fps".into(), on_off(c.force_max_fps).into()),
        ("Slamclick filter".into(), on_off(c.slamclick_filter).into()),
        ("Jitter filter".into(), on_off(c.jitter_filter).into()),
    ];

    let cpis = c
    .cpis
    .iter()
    .enumerate()
    .map(|(i, cpi)| {
        let value = if cpi.xy_split {
            format!("X={} Y={}", cpi.x, cpi.y)
        } else {
            cpi.x.to_string()
        };
        let active = if (i as u8) < c.cpi_levels { "" } else { "  (inactive)" };
        (format!("CPI {}", i + 1), format!("{value}{active}"))
    })
    .collect();

    let buttons = c
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
        (name.to_string(), value)
    })
    .collect();

    Ok(Snapshot {
        firmware: format!("Firmware {minor:x}.{major:x}"),
       general,
       cpis,
       buttons,
    })
}

fn to_model(rows: Vec<(String, String)>) -> slint::ModelRc<Row> {
    let rows: Vec<Row> = rows
    .into_iter()
    .map(|(label, value)| Row {
        label: label.into(),
         value: value.into(),
    })
    .collect();
    slint::ModelRc::new(slint::VecModel::from(rows))
}

fn load(ui: &MainWindow) {
    match read_mouse() {
        Ok(s) => {
            ui.set_status("".into());
            ui.set_firmware(s.firmware.into());
            ui.set_general(to_model(s.general));
            ui.set_cpis(to_model(s.cpis));
            ui.set_buttons(to_model(s.buttons));
        }
        Err(e) => ui.set_status(format!("{e:#}").into()),
    }
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

    ui.run()
}
