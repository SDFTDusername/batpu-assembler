#[derive(Debug, Clone, Copy)]
pub struct Address {
    address: u16
}

impl Address {
    pub fn new(address: u16) -> Self {
        if address > 1023 {
            panic!("Address {} out of range, expected 0-1023", address);
        }

        Self { address }
    }

    pub fn address(&self) -> u16 {
        self.address
    }
    
    pub fn set_address(&mut self, address: u16) {
        if address > 1023 {
            panic!("Address {} out of range, expected 0-1023", address);
        }

        self.address = address;
    }
}