#![windows_subsystem = "windows"]

use std::{
    io::Cursor,
    process::exit,
    thread::{self, sleep},
    time::Duration,
};

use hidapi::{DeviceInfo, HidApi};
use image::io::Reader;
use include_dir::{include_dir, Dir};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIconBuilder,
};

fn get_rival650(api: &HidApi) -> Option<&DeviceInfo> {
    let wired_rival650 = api
        .device_list()
        .find(|d| d.vendor_id() == 4152 && d.product_id() == 5931 && d.interface_number() == 0);
    match wired_rival650 {
        Some(device) => Some(device),
        None => api
            .device_list()
            .find(|d| d.vendor_id() == 4152 && d.product_id() == 5926 && d.interface_number() == 0),
    }
}

fn get_rival650_battery() -> Option<u8> {
    let api = HidApi::new().ok()?;
    let device = get_rival650(&api)?;
    let opened_device = device.open_device(&api).ok()?;
    opened_device.write(&[0x00, 0xAA, 0x01]).ok()?;
    let mut buf = [0u8; 1];
    opened_device.read(&mut buf).ok()?;
    return Some(buf[0]);
}

fn main() {
    static ICONS: Dir = include_dir!("icons");
    let tray_menu = Menu::new();
    tray_menu
        .append(&MenuItem::with_id("quit", "Quit", true, None))
        .unwrap();
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .build()
        .unwrap();
    let event_loop = EventLoopBuilder::with_user_event().build();
    let proxy = event_loop.create_proxy();
    thread::spawn(move || loop {
        let battery = get_rival650_battery();
        let battery_text = match battery {
            Some(battery) => format!("Rival 650 battery level is {}%", battery),
            None => "Rival 650 battery level is unknown".to_owned(),
        };
        let image = Reader::new(Cursor::new(
            ICONS
                .get_file(format!("{}.png", battery.unwrap_or(0)))
                .unwrap()
                .contents(),
        ))
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .to_rgba8()
        .to_vec();
        let icon = Icon::from_rgba(image, 16, 16).unwrap();
        proxy.send_event((icon, battery_text)).unwrap();
        sleep(Duration::from_secs(60));
    });
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let tao::event::Event::UserEvent((icon, text)) = event {
            tray_icon.set_icon(Some(icon)).unwrap();
            tray_icon.set_tooltip(Some(text)).unwrap();
        }

        if let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id == "quit" {
                exit(0);
            }
        }
    });
}
