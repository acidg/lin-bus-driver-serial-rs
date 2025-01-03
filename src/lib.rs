pub use lin_bus;
pub use serial;

use lin_bus::{driver, PID};
use serial::{SerialPort, SystemPort};
use std::io::{Read, Write};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

pub struct SerialLin(pub SystemPort);

impl SerialLin {
    pub fn new(path: &str) -> Result<Self, SerialError> {
        let port = SystemPort::open(Path::new(path)).unwrap();

        return Ok(SerialLin(port));
    }

    pub fn send_break(&mut self) -> Result<(), SerialError> {
        self.0.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud4800)?;
            settings.set_char_size(serial::Bits7);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;

        self.0.write_all(&[0])?;
        // wait a short time before switching baudrate again, otherwise the zero byte won't be sent
        // with the lower baudrate
        sleep(Duration::from_millis(1));

        self.0.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud9600)?;
            settings.set_char_size(serial::Bits8);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;
        Ok(())
    }
}

#[derive(Debug)]
pub enum SerialError {
    LinError(driver::Error),
    SerialError(serial::Error),
}

impl From<serial::Error> for SerialError {
    fn from(error: serial::Error) -> SerialError {
        SerialError::SerialError(error)
    }
}

impl From<SerialError> for driver::Error {
    fn from(error: SerialError) -> driver::Error {
        match error {
            SerialError::LinError(lin_error) => lin_error,
            SerialError::SerialError(ser_error) => match ser_error.kind() {
                serial::ErrorKind::Io(std::io::ErrorKind::TimedOut) => driver::Error::Timeout,
                _ => driver::Error::PhysicalBus,
            },
        }
    }
}

impl From<driver::Error> for SerialError {
    fn from(error: driver::Error) -> SerialError {
        SerialError::LinError(error)
    }
}

impl From<std::io::Error> for SerialError {
    fn from(error: std::io::Error) -> SerialError {
        SerialError::SerialError(serial::Error::from(error))
    }
}

impl driver::Master for SerialLin {
    type Error = SerialError;

    fn send_wakeup(&mut self) -> Result<(), SerialError> {
        // self.0.set_timeout(Duration::from_millis(10000))?;

        self.0.reconfigure(&|settings| {
            settings.set_baud_rate(serial::Baud4800)?;
            settings.set_char_size(serial::Bits7);
            settings.set_parity(serial::ParityNone);
            settings.set_stop_bits(serial::Stop1);
            settings.set_flow_control(serial::FlowNone);
            Ok(())
        })?;

        self.0.write_all(&[0])?;
        // let mut buf = [0; 1];
        // self.0.read_exact(&mut buf)?;

        // if buf[0] != 0 {
        //     Err(SerialError::LinError(driver::Error::PhysicalBus))
        // } else {
            sleep(Duration::from_millis(100));
            Ok(())
        // }
    }

    fn send_header(&mut self, pid: PID) -> Result<(), SerialError> {
        self.0.set_timeout(Duration::from_millis(1000))?;

        self.send_break()?;

        self.0.write_all(&[0x55, pid.get()])?;

        // let mut buf = [0; 2];
        // self.0.read_exact(&mut buf)?;

        // if buf != [0x55, pid.get()] {
        //     Err(SerialError::LinError(driver::Error::PhysicalBus))
        // } else {
            Ok(())
        // }
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<(), SerialError> {
        self.0.read_exact(buf)?;
        Ok(())
    }
    fn write(&mut self, data: &[u8]) -> Result<(), SerialError> {
        assert!(
            data.len() <= 9,
            "Data must be at most 8 bytes + 1 checksum byte"
        );
        self.0.write_all(data)?;
        let mut buf = [0; 9];
        // self.0.read_exact(&mut buf[0..data.len()])?;
        // if &buf[0..data.len()] != data {
        //     Err(SerialError::LinError(driver::Error::PhysicalBus))
        // } else {
            Ok(())
        // }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}