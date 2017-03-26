pub struct DeviceBlock {
    cap: usize,
    size_bits: u8,
    paddr: usize
}

impl DeviceBlock {
    pub fn start(&self) -> usize {
        self.paddr
    }

    pub fn len(&self) -> usize {
        1 << self.size_bits
    }

    pub fn end(&self) -> usize {
        self.paddr + self.len()
    }

    pub fn contains(&self, addr: usize) -> bool {
        self.start() <= addr && addr < self.end()
    }
}

impl ::core::fmt::Display for DeviceBlock {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        write!(f, "{} => {:#X}-{:#X}", self.cap, self.start(), self.end())
    }
}

// ordered from lowest address to highest address
static mut DEVICES: Option<::memory::LinkedList<DeviceBlock>> = None;

pub fn get_device_list() -> &'static ::memory::LinkedList<DeviceBlock> {
    let dev: &Option<::memory::LinkedList<DeviceBlock>> = unsafe { &DEVICES };
    if let &Some(ref out) = dev {
        out
    } else {
        panic!("device listing not yet initialized!");
    }
}

pub fn get_containing_block(addr: usize) -> Option<&'static DeviceBlock> {
    get_device_list().into_iter().find(|dev| dev.contains(addr))
}

pub fn init() {
    let bi = ::sel4::sel4_bootinfo();
    let count = (bi.untyped.end - bi.untyped.start) as usize;
    // these are sorted!
    let mut devices = ::memory::LinkedList::empty();
    let mut last_addr: usize = (-1 as isize) as usize;
    for ir in 0..count {
        let i = count - 1 - ir;
        let ent = bi.untypedList[i];
        if ent.isDevice != 0 {
            let newblock = DeviceBlock { cap: (bi.untyped.start as usize) + i, size_bits: ent.sizeBits, paddr: ent.paddr as usize };
            assert!(newblock.end() <= last_addr);
            last_addr = newblock.start();
            devices = devices.push(newblock).unwrap();
        }
    }
    for dev in &devices {
        writeln!(::sel4::out(), "dev {} of {} bits", dev, dev.size_bits);
    }
    unsafe {
        assert!(DEVICES.is_none());
        DEVICES = Some(devices);
    }
}
