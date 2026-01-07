use esp_hal::i2c::master;
use esp_hal::time::{Duration, Instant};
use esp_println::println;


pub enum ClockStretch {
    Streching,
    NoStreching
}


pub enum Repeatability {
    High,
    Medium,
    Low
}   


struct SHT31RawData {
    temperature: u16,
    temperature_crc: u8,
    humidity: u16,
    humidity_crc: u8,
}


pub struct SHT31<'a, Dm> 
where Dm: esp_hal::DriverMode 
{
    i2c: &'a mut master::I2c<'a, Dm>,
    address: u8,
    clock_strech: ClockStretch,
    repeatability: Repeatability,
    crc: bool,
}


impl<'a, Dm> SHT31<'a, Dm>
where Dm: esp_hal::DriverMode 
{
    pub fn new(i2c: &'a mut master::I2c<'a, Dm>, address: u8) -> Self {
        Self{
            i2c: i2c,
            address: address,
            clock_strech: ClockStretch::NoStreching,
            repeatability: Repeatability::Medium,
            crc: false,
        }
    }


    pub fn with_repeatability(&mut self, repeatability: Repeatability) -> &mut Self {
        self.repeatability = repeatability;
        self
    }


    pub fn with_clock_strech(&mut self, clock_strech: ClockStretch) -> &mut Self {
        self.clock_strech = clock_strech;
        self
    }


    pub fn get_data(&mut self) -> Result<(f32, f32), master::Error> {
        let read_type = data_command(&self.clock_strech, &self.repeatability);
        let raw_data = self.get_raw_data(read_type)?;

        //Conversion fromulas
        let temperature = 0.00267033 * (raw_data.temperature as f32) - 45.0;
        let humidity = 0.0015259 * (raw_data.humidity as f32);

        Ok((temperature, humidity))
        
    }

    fn get_raw_data(&mut self, command: [u8; 2]) -> Result<SHT31RawData, master::Error> {
        self.i2c.write(self.address, &command)?;
    
        let delay_start = Instant::now();
        while delay_start.elapsed() < Duration::from_millis(20) {}

        let mut read_buffer = [0u8; 6];
        self.i2c.read(self.address, &mut read_buffer)?;

        let data =  SHT31RawData {
                temperature: ( (read_buffer[0] as u16) << 8 ) | (read_buffer[1] as u16),
                temperature_crc: read_buffer[2],
                humidity: ( (read_buffer[3] as u16 ) << 8 ) | (read_buffer[4] as u16),
                humidity_crc: read_buffer[5],
            };

        if self.crc == false {
            return Ok(data);
        }

        if self.is_crc_valid(&data) == false {
            //TODO: make it better
            println!("CRC failed!");
            return Ok(data);
        }

        Ok (data)
    }


    //TODO: make it error based return
    fn is_crc_valid(&mut self, data: &SHT31RawData) -> bool {
        if data.temperature_crc != crc8(0xFF, 0x31, data.temperature) {
            return false;
        }

        if data.humidity_crc != crc8(0xFF, 0x31, data.humidity) {
            return false;
        }

        true
    }
    
}


fn crc8(init_crc: u8, poly: u8, data: u16) -> u8 {
    let bytes: [u8; 2] = data.to_be_bytes();
    let mut crc: u8 = init_crc;

    for byte in bytes {
        crc = crc ^ byte;
        for _ in 0..8 {
            crc = if (crc & 0x80) != 0 { (crc<<1) ^ poly } else { crc<<1 }
        }
    }

    crc
}


fn data_command(clock_mode: &ClockStretch, repeatability: &Repeatability) -> [u8; 2] {
    let msb = match clock_mode {
        ClockStretch::Streching => 0x2C,
        ClockStretch::NoStreching => 0x24,
    };

    let lsb = match (clock_mode, repeatability) {
        (ClockStretch::Streching, Repeatability::High) => 0x06,
        (ClockStretch::Streching, Repeatability::Medium) => 0x0D, 
        (ClockStretch::Streching, Repeatability::Low) => 0x10,
        
        (ClockStretch::NoStreching, Repeatability::High) => 0x00,
        (ClockStretch::NoStreching, Repeatability::Medium) => 0x0B, 
        (ClockStretch::NoStreching, Repeatability::Low) => 0x16, 
    };

    [msb, lsb]
}