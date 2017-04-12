use ::core;
use ::drivers;
use ::kobject::*;
use ::mantle::KError;
use ::memory;
use ::drivers::ioport::IOPort;

pub static COM1: HardwareSerialPort = HardwareSerialPort { port: 0x3F8 };
pub static COM2: HardwareSerialPort = HardwareSerialPort { port: 0x2F8 };
pub static COM3: HardwareSerialPort = HardwareSerialPort { port: 0x3E8 };
pub static COM4: HardwareSerialPort = HardwareSerialPort { port: 0x2E8 };

const UNSCALED_BAUD_RATE: u32 = 115200;

pub struct HardwareSerialPort {
    port: u16
}

pub struct HardwareSerial {
    r_data: IOPort,
    r_interrupt: IOPort,
    r_fifo: IOPort,
    r_lcr: IOPort,
    r_mcr: IOPort,
    r_lsr: IOPort,
    r_msr: IOPort,
    r_scratch: IOPort
}

impl HardwareSerialPort {
    pub fn configure(&self, baud: u32) -> HardwareSerial {
        let ports = drivers::ioport::request(self.port, 8);
        let mut serial = HardwareSerial {
            r_data: ports.get(0),
            r_interrupt: ports.get(1),
            r_fifo: ports.get(2),
            r_lcr: ports.get(3),
            r_mcr: ports.get(4),
            r_lsr: ports.get(5),
            r_msr: ports.get(6),
            r_scratch: ports.get(7)
        };
        serial.initialize(baud);
        serial
    }
}

impl HardwareSerial {
    fn initialize(&mut self, baud: u32) {
        self.r_interrupt.set(0x00); // disable interrupts
        self.r_lcr.set(0x80); // set DLAB
        let divisor = (UNSCALED_BAUD_RATE / baud) as u16; // TODO: what if this doesn't yield a clean divisor?
        let (div_low, div_high) = drivers::bits::u16_to_u8(divisor);
        self.r_data.set(div_low);
        self.r_data.set(div_high);
        self.r_lcr.set(0x03); // 8 bits, 1 stop bit, parity=NONE,
        self.r_fifo.set(0xC7); // enable, clear, 14-byte threshold
        self.r_mcr.set(0x0B); // RTS/DSR set, IRQs enabled
    }

    pub fn recv_ready(&mut self) -> bool {
        (self.r_lsr.get() & 0x01) != 0
    }

    pub fn send_ready(&mut self) -> bool {
        (self.r_lsr.get() & 0x20) != 0
    }

    pub fn recv(&mut self) -> u8 {
        while !self.recv_ready() {} // TODO: don't busy-wait
        self.r_data.get()
    }

    pub fn recv_char(&mut self) -> char {
        self.recv() as char
    }

    pub fn recv_line(&mut self) -> memory::string::VarStr<'static> {
        let mut sb = memory::string::StringBuilder::new();
        loop {
            let c = self.recv();
            debug!("received {}", c);
            if c == 10 || c == 13 {
                return sb.to_str().unwrap()
            }
            sb.add_u8(c);
            if sb.is_truncated() {
                return sb.to_str().unwrap()
            }
        }
    }

    pub fn send(&mut self, char: u8) {
        while !self.send_ready() {} // TODO: don't busy-wait
        self.r_data.set(char)
    }

    pub fn send_char(&mut self, char: char) {
        if char > (255 as char) {
            self.send('?' as u8)
        } else {
            self.send(char as u8)
        }
    }

    pub fn send_str(&mut self, str: &str) {
        for char in str.chars() {
            self.send_char(char)
        }
    }
}
