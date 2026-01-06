#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

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

    loop {
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(500) {}
        
        match read_temp(&mut i2c) {
            Ok((temp, hum)) => println!("Temperature: {},   Humidity: {}", temp, hum),
            Err(err) => println!("Error: {}", err)
        }
    }
}


fn read_temp(i2c: &mut master::I2c<impl esp_hal::DriverMode>) -> Result<(f32, f32), master::Error> {
    i2c.write(0x44, &[0x24, 0x0B])?;
    
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(20) {}

    let mut read_buffer = [0u8; 6];

    i2c.read(0x44, &mut read_buffer)?;

    println!("array {:?}", read_buffer);

    let temp_raw= ( (read_buffer[0] as u16) << 8 ) | (read_buffer[1] as u16);
    let temp_crc = read_buffer[2];

    let hum_raw = ( (read_buffer[3] as u16 ) << 8 ) | (read_buffer[4] as u16);
    let hum_crc = read_buffer[5];

    let temperature = 0.00267033 * (temp_raw as f32) - 45.0;
    let humidity = 0.0015259 * (hum_raw as f32);

    Ok((temperature, humidity))
}