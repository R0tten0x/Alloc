use crate::memory;
use image::GenericImageView;
use std::time::{Duration, Instant};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::event_loop::{ControlFlow, EventLoop};

// how often the timer tick in the event loop re-checks memory pressure.
const POLL_INTERVAL: Duration = Duration::from_secs(5);

// ******** handle icons *************
fn load_icon(path: &str) -> Icon {
    let img = image::open(path).expect("Failed to open path");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    Icon::from_rgba(rgba, width, height).unwrap()
}

pub fn menubar() {
    let event_loop = EventLoop::new().unwrap();

    // --- Startup sequence ---
    // 1. Get a reading before the loop even starts, so there's something
    //    correct on screen before the first timer tick fires.
    let info = memory::poll_memory().expect("failed to read initial memory info");
    // 2 & 3. Pressure level -> which file -> decoded Icon.
    let icon = load_icon(info.pressure_level.icon_path());
    // 4. Remember what's currently displayed so the timer tick (below) can
    //    tell whether a new reading actually changed anything, instead of
    //    re-decoding + re-setting the icon every single tick.
    let mut current_pressure = info.pressure_level;

    // context menu.
    let tray_menu = Menu::new();
    let quit_app = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_app).unwrap();

    // Instantiate the system tray // menubar item.
    //
    // with_title() is the text drawn next to the icon in the menubar itself
    // (macOS-only, backed by NSStatusItem's title) — different from
    // with_tooltip(), which only shows up on hover.
    let tray_icon = TrayIconBuilder::new()
        .with_icon(icon)
        .with_title(format!("{:.1} GB", info.used_memory))
        .with_tooltip(format!(
            "{:.1} / {:.1} GB",
            info.used_memory, info.total_memory_gb
        ))
        .with_menu(Box::new(tray_menu))
        .build()
        .expect("failed to build tray icon");

    // track menu in the event loop
    let menu_channel = MenuEvent::receiver();

    // absolute time of the next scheduled memory check.
    let mut next_poll = Instant::now() + POLL_INTERVAL;

    // event looper.
    event_loop
        .run(move |_event, event_loop_window_target| {
            // winit has no built-in "fire every N seconds" event — this is
            // how you get one. WaitUntil(next_poll) tells the loop to sleep
            // until that instant *or* a real event arrives, whichever is
            // first, then run this closure again either way.
            event_loop_window_target.set_control_flow(ControlFlow::WaitUntil(next_poll));

            if Instant::now() >= next_poll {
                if let Some(info) = memory::poll_memory() {
                    // only touch the icon when the level actually changed —
                    // avoids needless file reads/decodes every tick.
                    if info.pressure_level != current_pressure {
                        let icon = load_icon(info.pressure_level.icon_path());
                        let _ = tray_icon.set_icon(Some(icon));
                        current_pressure = info.pressure_level;
                    }

                    // the GB figure moves every tick even when the icon
                    // doesn't, so title + tooltip always refresh.
                    let _ = tray_icon.set_title(Some(format!("{:.1} GB", info.used_memory)));
                    let _ = tray_icon.set_tooltip(Some(format!(
                        "{:.1} / {:.1} GB",
                        info.used_memory, info.total_memory_gb
                    )));
                }
                next_poll = Instant::now() + POLL_INTERVAL;
            }

            if let Ok(event) = menu_channel.try_recv() {
                if event.id == quit_app.id() {
                    event_loop_window_target.exit();
                }
            }
        })
        .unwrap();
}
