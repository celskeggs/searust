use ::mantle::concurrency::SingleThreaded;
use ::core::cell::RefCell;
use ::core::cell::RefMut;

mod ps2 {
    use ::drivers::irq;
    use ::drivers::ioport;
    use ::mantle::concurrency::SingleThreaded;
    use ::core::cell::RefCell;
    use ::core::cell::RefMut;

    pub struct PS2Controller {
        port_data: ioport::IOPort,
        port_command: ioport::IOPort,
        works: (bool, bool),
        port_1_irq: Option<irq::IRQ<'static>>,
        port_2_irq: Option<irq::IRQ<'static>>
    }

    const STATUS_CAN_READ: u8 = 0x01; // from data port
    const STATUS_CAN_WRITE: u8 = 0x02; // to either port
    const STATUS_IS_TO_DEV: u8 = 0x08; // is data for device as opposed to for controller command
    const STATUS_TIMEOUT_ERROR: u8 = 0x40; // 1 for error, 0 for no error
    const STATUS_PARITY_ERROR: u8 = 0x40; // 1 for error, 0 for no error

    const CONF_PORT1_INTERRUPT: u8 = 0x01;
    const CONF_PORT2_INTERRUPT: u8 = 0x02;
    const CONF_PORT1_CLOCK: u8 = 0x10; // 1 is disabled, 0 is enabled
    const CONF_PORT2_CLOCK: u8 = 0x20; // 1 is disabled, 0 is enabled
    const CONF_PORT1_TRANSLATE: u8 = 0x40;

    impl PS2Controller {
        fn new() -> PS2Controller {
            let mut ctrl = PS2Controller { port_data: ioport::request_one(0x60), port_command: ioport::request_one(0x64), works: (false, false) };
            ctrl.initialize();
            ctrl
        }

        fn status(&self) -> u8 {
            self.port_command.get()
        }

        fn can_read(&self) -> bool {
            (self.status() & STATUS_CAN_READ) != 0
        }

        fn can_write(&self) -> bool {
            (self.status() & STATUS_CAN_WRITE) != 0
        }

        fn wait_until_readable(&self) {
            while !self.can_read() {} // TODO: don't busywait
        }

        fn wait_until_writable(&self) {
            while !self.can_write() {} // TODO: don't busywait
        }

        fn command(&mut self, cmd: u8) {
            self.wait_until_writable();
            self.port_command.set(cmd)
        }

        fn read(&mut self) -> u8 {
            self.wait_until_readable();
            self.port_data.get()
        }

        fn read_opt(&mut self) -> Option<u8> {
            if self.can_read() {
                Some(self.port_data.get())
            } else {
                None
            }
        }

        fn write(&mut self, data: u8) {
            self.wait_until_writable();
            self.port_data.set(data)
        }

        fn read_conf_byte(&mut self) -> u8 {
            self.command(0x20);
            self.read()
        }

        fn write_conf_byte(&mut self, conf: u8) {
            self.command(0x60);
            self.write(conf)
        }

        fn initialize(&mut self) -> (bool, bool) {
            // TODO: turn off USB legacy support? (requires USB driver)
            // TODO: confirm via BIOS that ps/2 controller exists (requires ACPI driver)
            self.command(0xAD); // disable first PS/2 port
            self.command(0xA7); // disable second PS/2 port
            while self.read_opt().is_some() {} // flush buffer

            // disable interrupts and scan map translation
            let mut conf = self.read_conf_byte();
            conf &= !(CONF_PORT1_INTERRUPT | CONF_PORT2_INTERRUPT | CONF_PORT1_TRANSLATE);
            self.write_conf_byte(conf);

            self.command(0xAA); // self-test
            let selftest = self.read();
            if selftest != 0x55 {
                // not working!
                debug!("ps/2 controller self-test failed! (expected 0x55, got {})", selftest);
                return (false, false);
            }

            let mut is_dual_channel = (conf & CONF_PORT2_CLOCK) != 0; // initial vague check: should be disabled
            if is_dual_channel {
                // check more closely
                self.command(0xA8); // enable second PS/2 port
                if (self.read_conf_byte() & CONF_PORT2_CLOCK) != 0 {
                    // if it's still disabled, also not a dual-channel device
                    is_dual_channel = false;
                }
                self.command(0xA7); // disable second PS/2 port
            }

            let mut works = (true, is_dual_channel);
            self.command(0xAB); // test first PS/2 port
            if self.read() != 0x00 {
                debug!("first PS/2 port failed test!");
                works.0 = false;
            }
            if is_dual_channel {
                self.command(0xA9); // test first PS/2 port
                if self.read() != 0x00 {
                    debug!("second PS/2 port failed test!");
                    works.1 = false;
                }
            }
            if !works.0 && !works.1 {
                debug!("no working PS/2 ports found!");
                return (false, false);
            }

            if works.0 {
                self.command(0xAE); // enable port
                let conf = self.read_conf_byte();
                self.write_conf_byte(conf | CONF_PORT1_INTERRUPT);
                assert!((conf & CONF_PORT1_CLOCK) == 0);
            }
            if works.1 {
                self.command(0xA8); // enable port
                let conf = self.read_conf_byte();
                self.write_conf_byte(conf | CONF_PORT2_INTERRUPT);
                assert!((conf & CONF_PORT2_CLOCK) == 0);
            }

            self.works = works;

            works
        }

        pub fn write_port_1(&mut self, b: u8) {
            assert!(self.works.0);
            self.write(b) // TODO: timeout
        }

        pub fn write_port_2(&mut self, b: u8) {
            assert!(self.works.1);
            self.command(0xD4);
            self.write(b)
        }

        pub fn start_port_1(&mut self, cb: FnMut(u8)) {
            assert!(self.works.0);
            assert!(self.port_1_irq.is_none());
            self.port_1_irq = Some(irq::request(1).unwrap());
            let portref = &self.port_1_irq.unwrap();
            portref.set_cb(|| {
                portref.ack();
                assert!(self.can_read());
                cb(self.port_data.get());
            });
        }

        pub fn start_port_2(&mut self, cb: FnMut(u8)) {
            assert!(self.works.1);
            assert!(self.port_2_irq.is_none());
            self.port_2_irq = Some(irq::request(12).unwrap());
            let portref = &self.port_2_irq.unwrap();
            portref.set_cb(|| {
                portref.ack();
                assert!(self.can_read());
                cb(self.port_data.get());
            });
        }

        pub fn get_works(&self) -> (bool, bool) {
            self.works
        }

        pub fn cpu_reset(&mut self) {
            self.command(0xFE);
        }
    }

    static CONTROLLER: SingleThreaded<RefCell<Option<PS2Controller>>> = SingleThreaded(RefCell::new(None));

    pub fn get_and_init_controller() -> RefMut<'static, PS2Controller> {
        let m = CONTROLLER.get().borrow_mut();
        if (*m).is_none() {
            *m = Some(PS2Controller::new());
        }
        RefMut::map(m, |b| b.unwrap())
    }
}

#[derive(Eq, PartialEq)]
struct PS2Handler {
    is_second: bool,
    state: PS2HandlerState
}

enum PS2HandlerState {
    PreReset,
    SentReset,
    SelfTestPassed,
    SentDisableScan,
    SentIdentify,
    PartialIdentify,
    IgnoredDevice,
    FailedInit,
    FoundKeyboard,
    GotEcho
}

impl PS2Handler {
    fn create(is_second: bool) -> PS2Handler {
        PS2Handler { is_second, state: PS2HandlerState::PreReset }
    }

    fn change_state(&mut self, state: PS2HandlerState) {
        assert!(state != self.state);
        debug!("PS/2 state change on port {}: {} -> {}", if self.is_second { 2 } else { 1 }, self.state, state);
        self.state = state;
    }

    fn on_recv(&mut self, byte: u8) { // NOTE: much of this ordering only works if this is single-threaded!
        let ctrl: &mut ps2::PS2Controller = &mut *ps2::get_and_init_controller();
        debug!("received {} from ps/2 device", byte);
        match self.state {
            PS2HandlerState::PreReset => {
                // nothing expected: ignore it all!
            }, PS2HandlerState::SentReset => {
                self.change_state(if byte == 0xAA { PS2HandlerState::SelfTestPassed } else { PS2HandlerState::FailedInit })
            }, PS2HandlerState::SelfTestPassed => {
                if byte == 0xFA {
                    self.write_change_state(ctrl, 0xF5, PS2HandlerState::SentDisableScan)
                } else {
                    self.change_state(PS2HandlerState::FailedInit)
                }
            }, PS2HandlerState::SentDisableScan => {
                if byte == 0xFA {
                    self.write_change_state(ctrl, 0xF2, PS2HandlerState::SentIdentify)
                } else {
                    self.change_state(PS2HandlerState::FailedInit)
                }
            }, PS2HandlerState::SentIdentify => {
                if byte == 0xAB {
                    self.change_state(PS2HandlerState::PartialIdentify)
                } else {
                    self.change_state(PS2HandlerState::IgnoredDevice)
                }
            }, PS2HandlerState::PartialIdentify => {
                if byte == 0x41 || byte == 0x83 || byte == 0xC1 {
                    self.write_change_state(ctrl, 0xEE, PS2HandlerState::FoundKeyboard)
                } else {
                    self.change_state(PS2HandlerState::IgnoredDevice)
                }
            }, PS2HandlerState::FoundKeyboard => {
                // we sent ECHO earlier
                if byte == 0xEE {
                    self.change_state(PS2HandlerState::GotEcho)
                } else {
                    self.change_state(PS2HandlerState::FailedInit)
                }
            }, PS2HandlerState::FailedInit => {
            }, PS2HandlerState::IgnoredDevice => {
            }, PS2HandlerState::GotEcho => {
            }
        }
    }

    fn write(&mut self, ctrl: &mut ps2::PS2Controller, command: u8) {
        if self.is_second {
            ctrl.write_port_2(command);
        } else {
            ctrl.write_port_1(command);
        }
    }

    fn write_and_state(&mut self, ctrl: &mut ps2::PS2Controller, command: u8, state: PS2HandlerState) {
        self.write(ctrl, command);
        self.change_state(state)
    }

    fn init(&mut self) {
        let ctrl: &mut ps2::PS2Controller = &mut *ps2::get_and_init_controller();
        ctrl.start_port_1(self.on_recv);
        self.write_and_state(ctrl, 0xFF, PS2HandlerState::SentReset);
    }
}

struct GlobalPS2 {
    inited: bool,
    first: Option<PS2Handler>,
    second: Option<PS2Handler>
}

static STATE: SingleThreaded<RefCell<GlobalPS2>> = SingleThreaded(RefCell::new(GlobalPS2 { inited: false, first: None, second: None }));

// NOTE: requires irq mainloop to be used
pub fn init() {
    let stateref = &mut *STATE.get().borrow_mut();
    let ctrl = &*ps2::get_and_init_controller();
    let works = ctrl.get_works();
    if works.0 {
        stateref.first = Some(PS2Handler::create(false));
        stateref.first.init();
    }
    if works.1 {
        stateref.second = Some(PS2Handler::create(true));
        stateref.second.init();
    }
    // TODO: use mainloop of some sort?
}
