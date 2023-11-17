use std::convert::TryFrom;

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

pub enum Direction {
    Down,
    Left,
    Right,
}

impl TryFrom<i32> for Rotation {
    type Error = ();
    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Rotation::Rot0),
            1 => Ok(Rotation::Rot90),
            2 => Ok(Rotation::Rot180),
            3 => Ok(Rotation::Rot270),
            _ => Err(()),
        }
    }
}

impl std::ops::Add for Rotation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::try_from((self as i32 + rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::AddAssign for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::try_from((*self as i32 + rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::Sub for Rotation {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::try_from((self as i32 - rhs as i32).rem_euclid(4)).unwrap()
    }
}

impl std::ops::SubAssign for Rotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::try_from((*self as i32 - rhs as i32).rem_euclid(4)).unwrap()
    }
}
