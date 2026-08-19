use crate::memory;
use image::GenericImageView;
use std::time::{Duration, Instant};
use tray_icon::{
    Icon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem},
};
use winit::event_loop::{ControlFlow, EventLoop};
#[cfg(target_os = "macos")]
use winit::platform::macos::{ActivationPolicy, EventLoopBuilderExtMacOS};
const POLL_INTERVAL: Duration = Duration::from_secs(2);

// ******** handle icons *************
fn load_icon(bytes: &[u8]) -> Icon {
    let img = image::load_from_memory(bytes).expect("Failed to decode embedded icon");
    let (width, height) = img.dimensions();
    let rgba = img.into_rgba8().into_raw();
    Icon::from_rgba(rgba, width, height).unwrap()
}

pub fn menubar() {
    // ActivationPolicy has to be set on the *builder*, before the EventLoop
    // is built — EventLoop::new() gives you an already-built one with no
    // way to change this after the fact.
    let mut event_loop_builder = EventLoop::builder();

    #[cfg(target_os = "macos")]
    event_loop_builder.with_activation_policy(ActivationPolicy::Accessory);

    let event_loop = event_loop_builder.build().unwrap();

    let info = memory::poll_memory().expect("failed to read initial memory info");

    let icon = load_icon(info.pressure_level.icon_bytes());

    let mut current_pressure = info.pressure_level;

    // context menu.
    let tray_menu = Menu::new();
    let quit_app = MenuItem::new("Quit", true, None);
    tray_menu.append(&quit_app).unwrap();

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
            event_loop_window_target.set_control_flow(ControlFlow::WaitUntil(next_poll));

            if Instant::now() >= next_poll {
                if let Some(info) = memory::poll_memory() {
                    if info.pressure_level != current_pressure {
                        let icon = load_icon(info.pressure_level.icon_bytes());
                        let _ = tray_icon.set_icon(Some(icon));
                        current_pressure = info.pressure_level;
                    }

                    tray_icon.set_title(Some(format!("{:.1} GB", info.used_memory)));
                    let _ = tray_icon.set_tooltip(Some(format!(
                        "{:.1} / {:.1} GB",
                        info.used_memory, info.total_memory_gb
                    )));
                }
                next_poll = Instant::now() + POLL_INTERVAL;
            }

            if let Ok(event) = menu_channel.try_recv()
                && event.id == quit_app.id()
            {
                event_loop_window_target.exit();
            }
        })
        .unwrap();
}
