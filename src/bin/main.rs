#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

mod sht31;

use sht31::SHT31;

use esp_println::println;
use esp_hal::{
    clock::CpuClock,
    main,
    time::{Duration, Instant, Rate},
    i2c::master,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let config = master::Config::default().with_frequency(Rate::from_khz(400));
    let mut i2c = master::I2c::new(peripherals.I2C0, config)
            .unwrap()
            .with_sda(peripherals.GPIO23)
            .with_scl(peripherals.GPIO15);

    let mut sht = SHT31::new(&mut i2c, 0x44);

    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
        
        match sht.one_shot_data() {
            Ok((temp, hum)) => println!("Temperature: {},   Humidity: {}", temp, hum),
            Err(err) => println!("Error: {}", err)
        }
    }
}
