use bevy::math::prelude::*;

pub trait ToBevy {
    fn to_bevy(self) -> Vec3;
}

pub trait ToParry {
    fn to_parry(self) -> parry3d::math::Vector;
}

impl ToParry for Vec3 {
    fn to_parry(self) -> parry3d::math::Vector {
        parry3d::math::Vector::new(self.x, self.y, self.z)
    }
}

impl ToBevy for parry3d::math::Vector {
    fn to_bevy(self) -> Vec3 {
        Vec3::new(self.x, self.y, self.z)
    }
}