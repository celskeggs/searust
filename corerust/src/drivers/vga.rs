use ::core;
use ::kobject::*;
use ::crust::device;
use ::mantle::KError;

const VGA_BUFFER: usize = 0xb8000;

pub struct VGA {
    addr: usize,
    mapping: Option<MappedPage4K> // None only during deconstruction
}

pub const VGA_WIDTH: u8 = 80;
pub const VGA_HEIGHT: u8 = 25;

pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGrey = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15
}

impl VGA {
    pub fn vga_default_port() -> core::result::Result<VGA, KError> {
        VGA::vga_port(VGA_BUFFER)
    }

    pub fn vga_port(addr: usize) -> core::result::Result<VGA, KError> {
        Ok(VGA { addr, mapping: Some(device::get_mapped_device_page(addr)?) })
    }

    fn mapping(&mut self) -> &mut MappedPage4K {
        let m: &mut Option<MappedPage4K> = &mut self.mapping;
        if let &mut Some(ref mut page) = m {
            page
        } else {
            panic!("partially deconstructed VGA instance!");
        }
    }

    pub fn put_raw_char(&mut self, x: u8, y: u8, char: u8, fg: Color, bg: Color, blink: bool) {
        assert!(x < VGA_WIDTH && y < VGA_HEIGHT);
        let i = ((x as usize) + (y as usize) * (VGA_WIDTH as usize)) * 2;
        let high_byte: u8 = ((fg as u8) & 0xF) | (((bg as u8) & 0x7) << 4) | (if blink { 0x80 } else { 0 });
        let array = self.mapping().get_array();
        // TODO: change this to use volatile memory accesses
        array[i] = char;
        array[i + 1] = high_byte;
    }

    pub fn put_char(&mut self, x: u8, y: u8, char: u8) {
        self.put_raw_char(x, y, char, Color::White, Color::Black, false)
    }

    pub fn scroll_up_one_line(&mut self) {
        let array = self.mapping().get_array();
        for y in 0..(VGA_HEIGHT - 1) {
            for x in 0..VGA_WIDTH {
                let i = ((x as usize) + (y as usize) * (VGA_WIDTH as usize)) * 2;
                array[i] = array[i + (VGA_WIDTH as usize) * 2];
                array[i + 1] = array[i + (VGA_WIDTH as usize) * 2 + 1];
            }
        }
        for x in 0..VGA_WIDTH {
            let i = ((x as usize) + ((VGA_HEIGHT - 1) as usize) * (VGA_WIDTH as usize)) * 2;
            array[i] = 0;
            array[i + 1] = 0;
        }
    }

    pub fn clear_screen(&mut self) {
        for x in 0..VGA_WIDTH {
            for y in 0..VGA_HEIGHT {
                self.put_raw_char(x, y, 0, Color::Black, Color::Black, false);
            }
        }
    }
}

impl Drop for VGA {
    fn drop(&mut self) {
        let mapping = core::mem::replace(&mut self.mapping, None).unwrap();
        device::return_mapped_device_page(self.addr, mapping);
    }
}

pub struct VGAOutput {
    cur_x: u8,
    cur_y: u8,
    screen: VGA
}

impl VGAOutput {
    pub fn default() -> Result<VGAOutput, KError> {
        let mut screen = VGA::vga_default_port()?;
        screen.clear_screen();
        Ok(VGAOutput { cur_x: 0, cur_y: 0, screen })
    }

    pub fn move_cursor(&mut self, x: u8, y: u8) {
        assert!(x < VGA_WIDTH && y < VGA_HEIGHT);
        self.cur_x = x;
        self.cur_y = y;
    }

    pub fn next_line(&mut self) {
        self.cur_x = 0;
        if self.cur_y == VGA_HEIGHT - 1 {
            self.screen.scroll_up_one_line();
        } else {
            self.cur_y += 1;
        }
    }

    pub fn put_char(&mut self, char: u8) {
        match char as char {
            '\n' => {
                self.next_line()
            }
            '\r' => {
                self.cur_x = 0;
            }
            _ => {
                self.screen.put_char(self.cur_x, self.cur_y, char);
                if self.cur_x == VGA_WIDTH - 1 {
                    self.next_line()
                } else {
                    self.cur_x += 1
                }
            }
        }
    }

    pub fn put_rchar(&mut self, char: char) {
        if char as u32 >= 256 {
            self.put_char('?' as u8)
        } else {
            self.put_char(char as u8)
        }
    }

    pub fn put_string(&mut self, str: &str) {
        for chr in str.chars() {
            self.put_rchar(chr);
        }
    }
}

impl core::fmt::Write for VGAOutput {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.put_string(s);
        Ok(())
    }
}
