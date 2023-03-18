/// A simple 3-vector. Can be used for any 3-tuple of floats i.e. for rgb, positions, or true vectors
#[derive(Debug, Copy, Clone)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// A ray consisting of a starting position and a direction. The direction should always be normalized.
#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub start_pos: Vec3,
    pub direction_vector: Vec3,
}

pub fn mag(a: &Vec3) -> f32 {
    return f32::sqrt(a.x * a.x + a.y * a.y + a.z * a.z);
}

pub fn norm(a: Vec3) -> Vec3 {
    let magnitude = mag(&a);
    return Vec3 {
        x: a.x / magnitude,
        y: a.y / magnitude,
        z: a.z / magnitude,
    };
}

pub fn cross(a: Vec3, b: Vec3) -> Vec3 {
    return Vec3 {
        x: a.y * b.z - a.z * b.y,
        y: -(a.x * b.z - a.z * b.x),
        z: a.x * b.y - a.y * b.x,
    };
}

pub fn vec(x: f32, y: f32, z: f32) -> Vec3 {
    return Vec3 { x: x, y: y, z: z };
}

impl std::ops::Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl std::ops::Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl std::ops::Mul<f32> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f32) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

// Dot product
impl std::ops::Mul for Vec3 {
    type Output = f32;

    fn mul(self, rhs: Vec3) -> f32 {
        return self.x * rhs.x + self.y * rhs.y + self.z * rhs.z;
    }
}
