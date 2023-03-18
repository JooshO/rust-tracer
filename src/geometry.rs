use crate::vec_math::{cross, norm, Ray, Vec3};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MaterialType {
    Reflective,
    Glossy,
    Matte,
}

#[derive(Debug, Copy, Clone)]
pub struct Material {
    pub(crate) color: Vec3,
    pub(crate) t: MaterialType,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f32,
    pub mat: Material,
    pub id: i8,
}

#[derive(Debug, Copy, Clone)]
pub struct Triangle {
    pub a: Vec3,
    pub b: Vec3,
    pub c: Vec3,
    pub mat: Material,
    pub id: i8,
}

pub fn triangle_hit(tr: Triangle, r: Ray, close: RayHit) -> RayHit {
    let a = tr.a.x - tr.b.x;
    let b = tr.a.y - tr.b.y;
    let c = tr.a.z - tr.b.z;
    let d = tr.a.x - tr.c.x;
    let e = tr.a.y - tr.c.y;
    let f = tr.a.z - tr.c.z;
    let g = r.direction_vector.x;
    let h = r.direction_vector.y;
    let i = r.direction_vector.z;
    let j = tr.a.x - r.start_pos.x;
    let k = tr.a.y - r.start_pos.y;
    let l = tr.a.z - r.start_pos.z;
    let m = a * (e * i - h * f) + b * (g * f - d * i) + c * (d * h - e * g);
    let t = -(f * (a * k - j * b) + e * (j * c - a * l) + d * (b * l - k * c)) / m;

    if t < 0.0 || t > close.t {
        return close;
    }

    let gamma = (i * (a * k - j * b) + h * (j * c - a * l) + g * (b * l - k * c)) / m;
    if gamma < 0.0 || gamma > 1.0 {
        return close;
    }

    let beta = (j * (e * i - h * f) + k * (g * f - d * i) + l * (d * h - e * g)) / m;
    if beta < 0.0 || beta > 1.0 - gamma {
        return close;
    }

    return RayHit {
        t,
        mat: tr.mat,
        intersect: r.start_pos + (r.direction_vector * t),
        surface_normal: norm(cross(tr.b - tr.a, tr.c - tr.a)),
        id: tr.id,
    };
}

// pub fn sphere(c: Vec3, r: f32, m: Material, i: i8) -> Sphere {
//     return Sphere {
//         center: c,
//         radius: r,
//         mat: m,
//         id: i,
//     };
// }

#[derive(Debug, Copy, Clone)]
pub struct RayHit {
    pub t: f32, // units to target
    pub mat: Material,
    pub intersect: Vec3,
    pub surface_normal: Vec3,
    pub id: i8,
}

pub fn sphere_intersect(s: &Sphere, r: &Ray) -> f32 {
    let emc = r.start_pos - s.center;
    let ddd = r.direction_vector * r.direction_vector;
    let ddemc = r.direction_vector * emc;
    let discriminant = (ddemc * ddemc) - ddd * ((emc * emc) - (s.radius * s.radius));

    if discriminant < 0.0 {
        return -1.0;
    };

    let t1 = (-ddemc + f32::sqrt(discriminant)) / ddd;
    let t2 = (-ddemc - f32::sqrt(discriminant)) / ddd;

    if t1 < 0.0 {
        return t2;
    } else if t2 < 0.0 {
        return t1;
    }
    return f32::min(t1, t2);
}

pub fn sphere_hit(s: Sphere, r: Ray) -> RayHit {
    let t_out = sphere_intersect(&s, &r);
    let intersection = r.start_pos + (r.direction_vector * t_out);
    return RayHit {
        t: t_out,
        mat: s.mat,
        intersect: intersection,
        surface_normal: norm(intersection - s.center),
        id: s.id,
    };
}
