use std::fmt;

pub trait Backend: fmt::Debug {
    fn size(&self) -> u16;
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, value: u8);
}

#[derive(Debug)]
pub struct Bus {
    entries: Vec<BusEntry>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus { entries: vec![] }
    }
    
    pub fn attach(&mut self, entry: BusEntry) {
        self.entries.push(entry);
    }
    
    pub fn read(&self, addr: u16) -> u8 {
        let entry = self.backend(addr);
        entry.read(addr)
    }
    
    pub fn read_u16(&self, addr: u16) -> u16 {
        let entry = self.backend(addr);
        if addr == entry.end {
            panic!("Unaligned 16-bit read at end of address range");
        }
        
        entry.read(addr) as u16 | ((entry.read(addr + 1) as u16) << 8)
    }
    
    pub fn write(&mut self, addr: u16, value: u8) {
        let entry = self.backend_mut(addr);
        entry.write(addr, value);
    }
    
    pub fn write_u16(&mut self, addr: u16, value: u16) {
        let entry = self.backend_mut(addr);
        if addr == entry.end {
            panic!("Unaligned 16-bit write at end of address range");
        }
        
        entry.write(addr, value as u8);
        entry.write(addr + 1, (value >> 8) as u8);
    }
    
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    
    fn backend(&self, addr: u16) -> &BusEntry {
        for entry in &self.entries {
            if entry.start <= addr && addr <= entry.end {
                return entry;
            }
        }
        panic!("No backend for address {:x}", addr);
    }
    
    fn backend_mut(&mut self, addr: u16) -> &mut BusEntry {
        for entry in &mut self.entries {
            if entry.start <= addr && addr <= entry.end {
                return entry;
            }
        }
        panic!("No backend for address {:x}", addr);
    }
}

#[derive(Debug)]
pub struct BusEntry {
    backend: Box<Backend>,
    name: String,
    start: u16,
    end: u16,
}

impl BusEntry {
    pub fn new(backend: Box<Backend>, name: String, start: u16) -> BusEntry {
        let size = backend.size();
        BusEntry {
            backend: backend,
            name: name,
            start: start,
            end: start + size,
        }
    }
    
    pub fn read(&self, addr: u16) -> u8 {
        self.backend.read(addr - self.start)
    }
    
    pub fn write(&mut self, addr: u16, value: u8) {
        self.backend.write(addr - self.start, value);
    }
}

