// accessory handles.
// these represent either the absense of an accessory, or a 15-bit
// index to an accessory if one is present.

// derived implementations for partialeq, eq, partialord, ord, hash
// rely on the use of 0xffff as the exclusive sentinel value.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AccHandle {
    v: u16,
}

impl AccHandle {
    pub fn empty() -> AccHandle {
        AccHandle { v: 0xffff }
    }

    pub fn is_empty(self) -> bool {
        self.v >= 0x8000
    }

    pub fn to_index(self) -> Option<usize> {
        if self.v & 0x8000 == 0 {
            Some(self.v as usize)
        } else {
            None
        }
    }

    pub fn next_max(self, max: u16) -> Option<AccHandle> {
        let v = self.v.wrapping_add(1);
        if v & 0x8000 == 0 && v < max {
            Some(AccHandle { v })
        } else {
            None
        }
    }
}

impl From<u16> for AccHandle {
    fn from(v: u16) -> AccHandle {
        if v < 0x8000 {
            AccHandle { v }
        } else {
            AccHandle { v: 0xffff }
        }
    }
}

impl From<i16> for AccHandle {
    fn from(v: i16) -> AccHandle {
        if v >= 0 {
            AccHandle { v: v as u16 }
        } else {
            AccHandle { v: 0xffff }
        }
    }
}

impl std::fmt::Display for AccHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        if self.v & 0x8000 == 0 {
            write!(f, "{}", self.v)
        } else {
            write!(f, "--")
        }
    }
}

