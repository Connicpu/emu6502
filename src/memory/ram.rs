use bus::{Backend, BusEntry};
use std::fmt;

pub struct Ram {
    data: [u8; 0x8000],
}

impl Ram {
    pub fn new_entry() -> BusEntry {
        BusEntry::new(
            Box::new(Ram { data: [0; 0x8000] }),
            "RAM".into(),
            0,
        )
    }
}

impl fmt::Debug for Ram {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "Ram{{[32k]}}")
    }
}

impl Backend for Ram {
    fn size(&self) -> u16 {
        0x8000
    }
    
    fn read(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }
    
    fn write(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value
    }
}
