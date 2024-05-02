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

static ICONS: Dir = include_dir!("icons");

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
    opened_device.read_timeout(&mut buf, 5000).ok()?;
    return Some(buf[0]);
}

fn get_icon(number: u8) -> Icon {
    let number = if number > 100 { 0 } else { number };
    let image = Reader::new(Cursor::new(
        ICONS
            .get_file(format!("{}.png", number))
            .unwrap()
            .contents(),
    ))
    .with_guessed_format()
    .unwrap()
    .decode()
    .unwrap();
    return Icon::from_rgba(image.to_rgba8().to_vec(), image.width(), image.height()).unwrap();
}

fn main() {
    let tray_menu = Menu::new();
    tray_menu
        .append(&MenuItem::with_id("quit", "Quit", true, None))
        .unwrap();
    let tray_icon = TrayIconBuilder::new()
        .with_icon(get_icon(0))
        .with_tooltip("Rival 650 battery level is unknown")
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
        proxy
            .send_event((get_icon(battery.unwrap_or(0)), battery_text))
            .unwrap();
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
