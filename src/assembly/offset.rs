#[derive(Debug, Clone, Copy)]
pub struct Offset {
    offset: i8
}

impl Offset {
    pub fn new(offset: i8) -> Self {
        if offset < -8 || offset > 7 {
            panic!("Offset {} out of range, expected -8-7", offset);
        }

        Self { offset }
    }

    pub fn offset(&self) -> i8 {
        self.offset
    }
    
    pub fn set_offset(&mut self, offset: i8) {
        if offset < -8 || offset > 7 {
            panic!("Offset {} out of range, expected -8-7", offset);
        }

        self.offset = offset;
    }
}