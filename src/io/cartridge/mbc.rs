use log::warn;

pub enum Mbc {
    Mbc0,
}

impl Mbc {
    pub fn from_id(id: u8) -> Option<Self> {
        match id {
            0 => Some(Self::Mbc0),
            _ => None,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match self {
            Self::Mbc0 => warn!("unhandled rom write: {:04x} = {:02x}", address, value),
        }
    }
}
