#![windows_subsystem = "windows"]

use std::{io::Cursor, thread::sleep, time::Duration};

use hidapi::{DeviceInfo, HidApi};
use image::io::Reader;
use include_dir::{include_dir, Dir};
use tray_icon::{Icon, TrayIconBuilder};

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
    let tray_icon = TrayIconBuilder::new().build().unwrap();
    loop {
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
        tray_icon.set_icon(Some(icon)).unwrap();
        tray_icon.set_tooltip(Some(battery_text)).unwrap();
        sleep(Duration::from_secs(60));
    }
}
