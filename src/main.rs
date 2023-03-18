// using https://github.com/image-rs/image | https://docs.rs/crate/image/latest

use geometry::sphere;
use geometry::sphere_hit;
use geometry::triangle_hit;
use geometry::RayHit;
use geometry::Sphere;
use geometry::Triangle;
use vec_math::mag;
use vec_math::norm;
use vec_math::vec;
use vec_math::Ray;
use vec_math::Vec3;

use crate::geometry::triangle;
use std::collections::VecDeque;
use std::env;
use std::num::ParseIntError;

mod geometry;
mod vec_math;

const BLUE: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    },
    t: geometry::MaterialType::Glossy,
};

const REFL: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    t: geometry::MaterialType::Reflective,
};

const RED: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    },
    t: geometry::MaterialType::Matte,
};

const WHITE: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    },
    t: geometry::MaterialType::Matte,
};

const NUL: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    t: geometry::MaterialType::Matte,
};

// use as f32 to cast ints to floats
fn get_ray(x: f32, y: f32, starting_pos: Vec3, pixel_width: f32) -> Ray {
    let img_x = (x * pixel_width) + (pixel_width / 2.0) - 1.0;
    let img_y = -((y * pixel_width) + (pixel_width / 2.0) - 1.0);
    let direction: Vec3 = norm(vec(img_x, img_y, -2.0));
    return Ray {
        start_pos: starting_pos,
        direction_vector: direction,
    };
}

fn find_closest_hit(ray: Ray, id: i8, spheres: &[Sphere], triangles: &[Triangle]) -> RayHit {
    let mut r: RayHit = RayHit {
        t: f32::MAX,
        mat: NUL,
        intersect: ray.start_pos,
        surface_normal: ray.start_pos,
        id: -2, // -2 is to flag as no-hit, should not come up
    };

    for sphere in spheres {
        let temp = sphere_hit(*sphere, ray);
        if (temp.t < r.t && temp.t > 0.0) && temp.id != id {
            r = temp;
        }
    }

    for triangle in triangles {
        let temp = triangle_hit(*triangle, ray, r);
        if (temp.t < r.t && temp.t > 0.0) && temp.id != id {
            r = temp;
        }
    }

    return r;
}

fn diffuse_calc(r: RayHit, light: Vec3, spheres: &[Sphere], triangles: &[Triangle]) -> f32 {
    let to_light = light - r.intersect;
    let to_light_norm = norm(to_light);
    let light_blocker = find_closest_hit(
        Ray {
            start_pos: r.intersect,
            direction_vector: to_light_norm,
        },
        r.id,
        &spheres,
        &triangles,
    );

    if light_blocker.t > 0.0 && mag(&to_light) > light_blocker.t {
        return 0.2;
    }

    return f32::clamp(to_light_norm * r.surface_normal, 0.2, 1.0); // TODO: 0.2 can be a shadow
}

fn specular_calc(
    surface_norm: Vec3,
    light_pos: Vec3,
    pos: Vec3,
    impact_direction: Vec3,
    _spheres: &[Sphere],
    _triangles: &[Triangle],
) -> f32 {
    // TODO THIS DOES NOT WORK
    let reflect = surface_norm * (-2.0 * (impact_direction * surface_norm)) + impact_direction;

    let light_dir_norm = norm(light_pos - pos);
    let half_angle = norm(norm(pos * -1.0) + light_dir_norm);
    let specular = (norm(reflect) * half_angle).powf(10.0);

    return specular.clamp(0.0, 1.0);
    // return (norm(reflect) * norm(pos)).powf(10.0).clamp(0.0, 1.0);
    // return specular.clamp(0.0, 1.0);
}

fn main() {
    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    let mut pixel_count = 512 as u32;
    let mut reflection_depth = 10;

    for arg in args {
        let mut split = arg.split("=");
        let command = split.next().unwrap_or_else(|| -> &str { "none" });
        let value = split.next().unwrap_or_else(|| -> &str { "" });

        match command {
            "--res" => {
                pixel_count = value
                    .parse::<u32>()
                    .unwrap_or_else(|_val: ParseIntError| -> u32 { 512 })
            }
            "--ref" => {
                reflection_depth = value
                    .parse::<i32>()
                    .unwrap_or_else(|_val: ParseIntError| -> i32 { 10 })
            }
            _ => println!("Invalid command: {:?}", command),
        }
    }

    let image_size = 2;
    let pixel_width = image_size as f32 / pixel_count as f32;
    let mut img: image::RgbImage = image::ImageBuffer::new(pixel_count, pixel_count);

    let start_pos = vec(0.0, 0.0, 0.0);
    let light_pos = vec(3.0, 20.0, -3.0);

    let spheres = [
        sphere(vec(0.1, 2.0, -15.0), 1.0, REFL, 1 as i8),
        sphere(vec(-3.0, 0.0, -10.0), 2.5, BLUE, 2 as i8),
    ];

    let triangles = [
        triangle(
            vec(-8.0, -2.0, -20.0),
            vec(8.0, -2.0, -20.0),
            vec(8.0, 10.0, -20.0),
            RED,
            3,
        ),
        triangle(
            vec(-8.0, -2.0, -20.0),
            vec(8.0, 10.0, -20.0),
            vec(-8.0, 10.0, -20.0),
            RED,
            4,
        ),
        triangle(
            vec(-8.0, -2.0, -20.0),
            vec(8.0, -2.0, -10.0),
            vec(8.0, -2.0, -20.0),
            WHITE,
            5,
        ),
        triangle(
            vec(-8.0, -2.0, -20.0),
            vec(-8.0, -2.0, -10.0),
            vec(8.0, -2.0, -10.0),
            WHITE,
            6,
        ),
        triangle(
            vec(8.0, -2.0, -20.0),
            vec(8.0, -2.0, -10.0),
            vec(8.0, 10.0, -20.0),
            WHITE,
            7,
        ),
    ];

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        // println!("{:?} {:?}", x, y);
        let mut r = 0 as u8;
        let mut g = 0 as u8;
        let mut b = 0 as u8;

        let mut ray_to_target = get_ray(x as f32, y as f32, start_pos, pixel_width);

        let mut ray_hit = find_closest_hit(ray_to_target, -1 as i8, &spheres, &triangles);

        if ray_hit.t >= 0.0 && ray_hit.t != f32::MAX {
            if ray_hit.mat.t == geometry::MaterialType::Matte {
                let diffuse = diffuse_calc(ray_hit, light_pos, &spheres, &triangles);

                r = (ray_hit.mat.color.x * diffuse * 255.0) as u8;
                g = (ray_hit.mat.color.y * diffuse * 255.0) as u8;
                b = (ray_hit.mat.color.z * diffuse * 255.0) as u8;
            } else if ray_hit.mat.t == geometry::MaterialType::Glossy {
                let diffuse = diffuse_calc(ray_hit, light_pos, &spheres, &triangles);
                let specular = specular_calc(
                    ray_hit.surface_normal,
                    light_pos,
                    ray_hit.intersect,
                    ray_to_target.direction_vector,
                    &spheres,
                    &triangles,
                );

                r = ((ray_hit.mat.color.x * diffuse + specular) * 255.0) as u8;
                g = ((ray_hit.mat.color.y * diffuse + specular) * 255.0) as u8;
                b = ((ray_hit.mat.color.z * diffuse + specular) * 255.0) as u8;
            } else {
                let mut hit_space = false;

                for _i in 0..reflection_depth {
                    if ray_hit.mat.t != geometry::MaterialType::Reflective {
                        break;
                    }

                    let direction = norm(
                        ray_hit.surface_normal
                            * (-2.0 * (ray_to_target.direction_vector * ray_hit.surface_normal))
                            + ray_to_target.direction_vector,
                    );

                    ray_to_target = Ray {
                        start_pos: ray_hit.intersect,
                        direction_vector: direction,
                    };

                    ray_hit = find_closest_hit(ray_to_target, ray_hit.id, &spheres, &triangles);

                    if ray_hit.t < 0.0 || ray_hit.t == f32::MAX {
                        hit_space = true;
                        break;
                    }
                }

                if ray_hit.mat.t != geometry::MaterialType::Reflective && !hit_space {
                    let diffuse = diffuse_calc(ray_hit, light_pos, &spheres, &triangles);
                    r = (ray_hit.mat.color.x * diffuse * 255.0) as u8;
                    g = (ray_hit.mat.color.y * diffuse * 255.0) as u8;
                    b = (ray_hit.mat.color.z * diffuse * 255.0) as u8;
                }
            }
        }
        *pixel = image::Rgb([r, g, b]);
    }

    img.save("test.png").unwrap();

    println!("Done!");
}
