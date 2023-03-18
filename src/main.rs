// using https://github.com/image-rs/image | https://docs.rs/crate/image/latest

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
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use std::path::Path;

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
    spheres: &[Sphere],
    triangles: &[Triangle],
    id: i8,
) -> f32 {
    let light_dir_norm = norm(light_pos - pos);
    let reflect = surface_norm * (surface_norm * light_dir_norm * 2.0) - light_dir_norm;
    let specular = (norm(reflect) * norm(pos * -1.0)).powf(11.0);

    let light_blocker = find_closest_hit(
        Ray {
            start_pos: pos,
            direction_vector: light_dir_norm,
        },
        id,
        &spheres,
        &triangles,
    );

    if light_blocker.t > 0.0 && mag(&(light_pos - pos)) > light_blocker.t {
        return 0.0;
    }
    return specular.clamp(0.0, 1.0);
}

fn read_lines(filename: String) -> io::Lines<BufReader<File>> {
    // Open the file in read-only mode.
    let file = File::open(filename).unwrap();
    // Read the file line by line, and return an iterator of the lines of the file.
    return io::BufReader::new(file).lines();
}

fn parse_vec(string: &str) -> Vec3 {
    let mut chars = string.chars();
    chars.next();
    chars.next_back();
    let fixed_str = chars.as_str();
    let mut split = fixed_str.split(" ");
    let x = split
        .next()
        .unwrap_or_else(|| -> &str { "" })
        .parse::<f32>()
        .unwrap_or_else(|_val| -> f32 { 0.0 });
    let y = split
        .next()
        .unwrap_or_else(|| -> &str { "" })
        .parse::<f32>()
        .unwrap_or_else(|_val| -> f32 { 0.0 });
    let z = split
        .next()
        .unwrap_or_else(|| -> &str { "" })
        .parse::<f32>()
        .unwrap_or_else(|_val| -> f32 { 0.0 });

    return vec(x, y, z);
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
    let mut spheres: Vec<Sphere> = Vec::new();

    let lines = read_lines("./test.ray".to_string());
    for line in lines {
        let line_str = line.unwrap_or_default();
        println!("{:?}", line_str);
        let mut split = line_str.split(",");
        match split.next().unwrap_or_default() {
            "sphere" => {
                let center_str = split.next().unwrap_or_default();
                let rad_str = split.next().unwrap_or_default();
                let color_str = split.next().unwrap_or_default();
                let mat_type_str = split.next().unwrap_or_default();
                let id_str = split.next().unwrap_or_default();

                let center = parse_vec(center_str);
                let color = parse_vec(color_str);
                let radius = rad_str.parse::<f32>().unwrap_or_else(|_val| -> f32 { 0.0 });
                let id = id_str.parse::<i8>().unwrap_or_else(|_val| -> i8 { -1 });
                let mat_type = match mat_type_str {
                    "matte" => geometry::MaterialType::Matte,
                    "glossy" => geometry::MaterialType::Glossy,
                    "refl" => geometry::MaterialType::Reflective,
                    _ => geometry::MaterialType::Matte,
                };
                let sphere = Sphere {
                    center,
                    mat: geometry::Material {
                        color: color,
                        t: mat_type,
                    },
                    radius,
                    id,
                };

                spheres.push(sphere);
            }
            "triangle" => {}
            _ => println!("Invalid line"),
        }
    }

    println!("{:?}", spheres);

    let image_size = 2;
    let pixel_width = image_size as f32 / pixel_count as f32;
    let mut img: image::RgbImage = image::ImageBuffer::new(pixel_count, pixel_count);

    let start_pos = vec(0.0, 0.0, 0.0);
    let light_pos = vec(-3.0, 8.0, -6.0);

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
                    &spheres,
                    &triangles,
                    ray_hit.id,
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
