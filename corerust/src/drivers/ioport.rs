use ::mantle;

pub struct IOPort {
    port: u16
}

pub struct IOPortSet {
    first: u16,
    count: u16
}

pub fn request(first_port: u16, count: u16) -> IOPortSet {
    // TODO: make exclusive!
    IOPortSet { first: first_port, count }
}

impl IOPort {
    pub fn get(&self) -> u8 {
        let (kerr, out) = mantle::x86_ioport_in8(mantle::kernel::CAP_INIT_IOPORT, self.port);
        if kerr.is_error() {
            panic!("could not read from IO port: {:?}", kerr);
        }
        out
    }

    pub fn set(&mut self, value: u8) {
        let kerr = mantle::x86_ioport_out8(mantle::kernel::CAP_INIT_IOPORT, self.port, value);
        if kerr.is_error() {
            panic!("could not write to IO port: {:?}", kerr);
        }
    }
}

impl IOPortSet {
    pub fn get(&self, n: u16) -> IOPort {
        assert!(n < self.count);
        IOPort { port: self.first + n }
    }
}
