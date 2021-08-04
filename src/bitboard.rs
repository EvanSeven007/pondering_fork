use std::ops::{BitAnd, BitOr, BitXor, Shl, Shr};
use std::fmt::{Display, Formatter, Result};
use crate::square::Square;

/* a bitboard to express positions
 * uses standard form, so H8G8F8 (...) C1B1A1.
 */
#[derive(Copy, Clone, Debug)]
pub struct Bitboard(pub u64);

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand (self, rhs: Self) -> Self::Output {
        return Self(self.0 & rhs.0);
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        return Self(self.0 | rhs.0);
    }
}

impl BitXor for Bitboard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        return Self(self.0 ^ rhs.0);
    }
}

impl Shl<i8> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: i8) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<i8> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: i8) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl Shl<i32> for Bitboard {
    type Output = Self;

    fn shl(self, rhs: i32) -> Self::Output {
        Bitboard(self.0 << rhs)
    }
}

impl Shr<i32> for Bitboard {
    type Output = Self;

    fn shr(self, rhs: i32) -> Self::Output {
        Bitboard(self.0 >> rhs)
    }
}

impl PartialEq for Bitboard {
    fn eq(&self, rhs: &Bitboard) -> bool {
        return self.0 == rhs.0;
    }
}
impl Eq for Bitboard {}

impl From<Square> for Bitboard {
    fn from(sq: Square) -> Bitboard {
        Bitboard(1 << sq.0)
    }
}

impl Display for Bitboard {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "0x{:16x}", self.0)
    }
}
