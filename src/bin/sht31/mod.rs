use esp_hal::i2c::master;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[derive(Copy, Clone)]
pub enum ClockStretch {
    Enable,
    Disable
}


impl ClockStretch {
    pub fn msb(self) -> u8 {
        match self {
            ClockStretch::Enable => 0x2C,
            ClockStretch::Disable => 0x24,
        }
    }
}


impl RepeatabilityContext for ClockStretch {
    fn lsb(self, repeatability: Repeatability) -> u8{
        match (self, repeatability) {
            (ClockStretch::Enable, Repeatability::High) => 0x06,
            (ClockStretch::Enable, Repeatability::Medium) => 0x0D, 
            (ClockStretch::Enable, Repeatability::Low) => 0x10,
            
            (ClockStretch::Disable, Repeatability::High) => 0x00,
            (ClockStretch::Disable, Repeatability::Medium) => 0x0B, 
            (ClockStretch::Disable, Repeatability::Low) => 0x16, 
        }
    }
}


#[derive(Copy, Clone)]
pub enum MeasurmentsPerSecond {
    None,
    Half,
    One,
    Two,
    Four,
    Ten
}


impl MeasurmentsPerSecond {
    pub fn msb(self) -> u8 {
        match self {
            MeasurmentsPerSecond::Half => 0x20,
            MeasurmentsPerSecond::One => 0x21,
            MeasurmentsPerSecond::Two => 0x22,
            MeasurmentsPerSecond::Four => 0x23,
            MeasurmentsPerSecond::Ten => 0x27,
            MeasurmentsPerSecond::None => todo!(),
        }
    }
}


impl RepeatabilityContext for MeasurmentsPerSecond {
    fn lsb(self, repeatability: Repeatability) -> u8{
        match (self, repeatability) {
            (MeasurmentsPerSecond::Half, Repeatability::High) => 0x32,
            (MeasurmentsPerSecond::Half, Repeatability::Medium) => 0x24, 
            (MeasurmentsPerSecond::Half, Repeatability::Low) => 0x2F,
            
            (MeasurmentsPerSecond::One, Repeatability::High) => 0x30,
            (MeasurmentsPerSecond::One, Repeatability::Medium) => 0x26, 
            (MeasurmentsPerSecond::One, Repeatability::Low) => 0x2D,

            (MeasurmentsPerSecond::Two, Repeatability::High) => 0x36,
            (MeasurmentsPerSecond::Two, Repeatability::Medium) => 0x20, 
            (MeasurmentsPerSecond::Two, Repeatability::Low) => 0x2B,

            (MeasurmentsPerSecond::Four, Repeatability::High) => 0x34,
            (MeasurmentsPerSecond::Four, Repeatability::Medium) => 0x22, 
            (MeasurmentsPerSecond::Four, Repeatability::Low) => 0x29,

            (MeasurmentsPerSecond::Ten, Repeatability::High) => 0x37,
            (MeasurmentsPerSecond::Ten, Repeatability::Medium) => 0x21, 
            (MeasurmentsPerSecond::Ten, Repeatability::Low) => 0x2A,

            (MeasurmentsPerSecond::None, _) => todo!(),
        }
    }
}

#[derive(Copy, Clone)]
pub enum Repeatability {
    High,
    Medium,
    Low
} 

pub trait RepeatabilityContext {
    fn lsb(self, rep: Repeatability) -> u8;
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
    msp: MeasurmentsPerSecond,
    crc: bool,

}


impl<'a, Dm> SHT31<'a, Dm>
where Dm: esp_hal::DriverMode 
{
    pub fn new(i2c: &'a mut master::I2c<'a, Dm>, address: u8) -> Self {
        Self{
            i2c: i2c,
            address: address,
            clock_strech: ClockStretch::Disable,
            repeatability: Repeatability::Medium,
            mps: MeasurmentsPerSecond::None,
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


    pub fn with_msp(&mut self, msp: MeasurmentsPerSecond) -> &mut Self {
        self.msp = msp;
        self
    }

    pub fn enable_mps(&mut self) -> Result<&mut Self, master::Error> {
        let mps_config = [
            self.mps.msb(),
            self.mps.lsb(self.repeatability),
        ];

        self.i2c.write(self.address, &mps_config)?;
        Ok(self)
    }


    pub fn disable_mps(&mut self) -> Result<&mut Self, master::Error> {
        let stop_mps = [
            0x30,
            0x93,
        ];

        self.i2c.write(self.address, &stop_mps)?;
        Ok(self)
    }


    pub fn soft_reset(&mut self) -> Result<&mut Self, master::Error> {
        let msb = 0x30;
        let lsb = 0xA2;

        self.i2c.write(self.address, &[msb, lsb])?;

        Ok(self)
    }


    pub fn one_shot_data(&mut self) -> Result<(f32, f32), master::Error> {
        let read_type = [
            self.clock_strech.msb(),
            self.clock_strech.lsb(self.repeatability),
        ];

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