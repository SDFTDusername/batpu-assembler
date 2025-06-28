#[derive(Debug, Copy, Clone)]
pub enum Condition {
    Zero,
    NotZero,
    Carry,
    NotCarry
}

impl Condition {
    pub fn index(&self) -> u8 {
        match self {
            Condition::Zero => 0,
            Condition::NotZero => 1,
            Condition::Carry => 2,
            Condition::NotCarry => 3
        }
    }
}