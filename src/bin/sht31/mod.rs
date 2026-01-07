use esp_hal::i2c::master;
use esp_hal::time::{Duration, Instant};


pub struct SHT31<'a, Dm> 
where Dm: esp_hal::DriverMode 
{
    i2c: &'a mut master::I2c<'a, Dm>,
    address: u8,
}


impl<'a, Dm> SHT31<'a, Dm>
where Dm: esp_hal::DriverMode 
{
    pub fn new(i2c: &'a mut master::I2c<'a, Dm>, address: u8) -> Self {
        Self{
            i2c: i2c,
            address: address,
        }
    }


    pub fn get_data(&mut self) -> Result<(f32, f32), master::Error> {
        self.i2c.write(self.address, &[0x24, 0x0B])?;
    
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(20) {}

        let mut read_buffer = [0u8; 6];

        self.i2c.read(self.address, &mut read_buffer)?;

        let temp_raw= ( (read_buffer[0] as u16) << 8 ) | (read_buffer[1] as u16);
        let temp_crc = read_buffer[2];

        let hum_raw = ( (read_buffer[3] as u16 ) << 8 ) | (read_buffer[4] as u16);
        let hum_crc = read_buffer[5];

        let temperature = 0.00267033 * (temp_raw as f32) - 45.0;
        let humidity = 0.0015259 * (hum_raw as f32);

        Ok((temperature, humidity))
    }
    
}