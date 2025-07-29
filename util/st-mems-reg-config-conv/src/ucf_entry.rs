pub struct UcfLineExt {
    pub address: u8,
    pub data: u8,
    pub op: MemsUcfOp
}

#[repr(u8)]
#[derive(Clone, Debug, Copy, PartialEq)]
pub enum MemsUcfOp {
    Read = 0x0,
    Write = 0x1,
    Delay = 0x2,
    PollSet = 0x3,
    PollReset = 0x4
}

impl From<u8> for MemsUcfOp {
    fn from(value: u8) -> Self{
        unsafe {
            core::mem::transmute(value)
        }
    }
}

impl MemsUcfOp {
    pub fn to_string(&self) -> &str {
        match self {
            MemsUcfOp::Read => "Read",
            MemsUcfOp::Write => "Write",
            MemsUcfOp::Delay => "Delay",
            MemsUcfOp::PollSet => "PollSet",
            MemsUcfOp::PollReset => "PollReset"
        }
    }
}
