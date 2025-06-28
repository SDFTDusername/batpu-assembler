#[derive(Debug, Clone, Copy)]
pub struct Immediate {
    immediate: u8
}

impl Immediate {
    pub fn new(immediate: u8) -> Self {
        Self { immediate }
    }

    pub fn new_signed(immediate: i16) -> Self {
        if immediate < -128 || immediate > 255 {
            panic!("Immediate {} out of range, expected -128-255", immediate);
        }

        Self {
            immediate: immediate as u8
        }
    }

    pub fn immediate(&self) -> u8 {
        self.immediate
    }
    
    pub fn set_immediate(&mut self, immediate: u8) {
        self.immediate = immediate;
    }

    pub fn set_immediate_signed(&mut self, immediate: i16) {
        if immediate < -128 || immediate > 255 {
            panic!("Immediate {} out of range, expected -128-255", immediate);
        }

        self.immediate = immediate as u8;
    }
}