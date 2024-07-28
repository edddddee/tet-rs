use std::convert::TryFrom;

#[derive(Debug, Clone, Copy)]
pub enum Rotation {
    Rot0,
    Rot90,
    Rot180,
    Rot270,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    Down,
    Left,
    Right,
}

impl From<i32> for Rotation {
    fn from(mut value: i32) -> Self {
        value = value.rem_euclid(4);
        match value {
            0 => Rotation::Rot0,
            1 => Rotation::Rot90,
            2 => Rotation::Rot180,
            3 => Rotation::Rot270,
            _ => unreachable!(),
        }
    }
}

impl std::ops::Add for Rotation {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::from(self as i32 + rhs as i32)
    }
}

impl std::ops::AddAssign for Rotation {
    fn add_assign(&mut self, rhs: Self) {
        *self = Self::from(*self as i32 + rhs as i32)
    }
}

impl std::ops::Sub for Rotation {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::from(self as i32 - rhs as i32)
    }
}

impl std::ops::SubAssign for Rotation {
    fn sub_assign(&mut self, rhs: Self) {
        *self = Self::from(*self as i32 - rhs as i32)
    }
}
