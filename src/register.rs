#[derive(Debug, Clone, Copy)]
pub struct Register {
    register: u8
}

impl Register {
    pub fn new(register: u8) -> Self {
        if register > 15 {
            panic!("Register {} out of range, expected 0-15", register);
        }

        Self { register }
    }

    pub fn register(&self) -> u8 {
        self.register
    }
    
    pub fn set_register(&mut self, register: u8) {
        if register > 15 {
            panic!("Register {} out of range, expected 0-15", register);
        }

        self.register = register;
    }
}