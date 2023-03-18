// using https://github.com/image-rs/image | https://docs.rs/crate/image/latest

mod geometry;
mod vec_math;

use geometry::{sphere_hit, triangle_hit, RayHit, Sphere, Triangle};
use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::num::ParseIntError;
use vec_math::{mag, norm, vec, Ray, Vec3};

/// Constant null Material used as a default
const NUL: geometry::Material = geometry::Material {
    color: Vec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    },
    t: geometry::MaterialType::Matte,
};

/// Returns a ray pointing at the image frame through a given pixel
/// # Arguements
/// * 'x' - A float for the x pixel
/// * 'y' - A float for the y pixel
/// * 'starting_pos' - A coordinate in 3 space for where the ray should emmenate from. Usually where the camera is
/// * 'pixel_width' - The width in arbitrary units of a given pixel in our final image
fn get_ray(x: f32, y: f32, starting_pos: Vec3, pixel_width: f32) -> Ray {
    let img_x = (x * pixel_width) + (pixel_width / 2.0) - 1.0;
    let img_y = -((y * pixel_width) + (pixel_width / 2.0) - 1.0);
    let direction: Vec3 = norm(vec(img_x, img_y, -2.0));
    return Ray {
        start_pos: starting_pos,
        direction_vector: direction,
    };
}

/// Finds the closest surface to a ray's origin along its direction. Used to see what a Ray would hit first
/// # Arguements
/// * 'ray' - The ray we want to test
/// * 'id' - An id of objects to ignore. Used to stop shadow/reflection acne
/// * 'spheres' - a slice of spheres to check the ray against
/// * 'triangles' - a slice of triangles to check the ray against
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
    // normalized vector from point to light
    let light_dir_norm = norm(light_pos - pos);

    // reflection of light vector across surface normal vector
    let reflect = surface_norm * (surface_norm * light_dir_norm * 2.0) - light_dir_norm;

    // basically how close that reflection is to our camera
    let specular = (norm(reflect) * norm(pos * -1.0)).powf(11.0);

    // make sure the light isn't getting blocked
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

    // clamp values to the reasonable
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
    // grab our args and spit out the executable name - we don't need it
    let mut args: VecDeque<String> = env::args().collect();
    args.pop_front();

    // define some defauls
    let mut pixel_count = 512 as u32;
    let mut reflection_depth = 10;
    let file_name = "./test.ray";
    let mut lines = read_lines(file_name.to_string());

    // loop over our args to check and see what command line args we have
    for arg in args {
        // split the arguement into command and value - what we are configing and the value we are giving it
        let mut split = arg.split("=");
        let command = split.next().unwrap_or_else(|| -> &str { "none" });
        let value = split.next().unwrap_or_else(|| -> &str { "" });

        match command {
            "--res" | "--resolution" => {
                pixel_count = value
                    .parse::<u32>()
                    .unwrap_or_else(|_val: ParseIntError| -> u32 { 512 })
            }
            "--ref" | "--reflections" => {
                reflection_depth = value
                    .parse::<i32>()
                    .unwrap_or_else(|_val: ParseIntError| -> i32 { 10 })
            }
            "--file" | "--input" | "--f" => {
                lines = read_lines(value.to_string());
            }
            _ => println!("Invalid command: {:?}", command),
        }
    }
    let mut spheres: Vec<Sphere> = Vec::new();
    let mut triangles: Vec<Triangle> = Vec::new();

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
            "triangle" => {
                let a_str = split.next().unwrap_or_default();
                let b_str = split.next().unwrap_or_default();
                let c_str = split.next().unwrap_or_default();
                let color_str = split.next().unwrap_or_default();
                let mat_type_str = split.next().unwrap_or_default();
                let id_str = split.next().unwrap_or_default();

                let a = parse_vec(a_str);
                let b = parse_vec(b_str);
                let c = parse_vec(c_str);
                let color = parse_vec(color_str);
                let id = id_str.parse::<i8>().unwrap_or_else(|_val| -> i8 { -1 });
                let mat_type = match mat_type_str {
                    "matte" => geometry::MaterialType::Matte,
                    "glossy" => geometry::MaterialType::Glossy,
                    "refl" => geometry::MaterialType::Reflective,
                    _ => geometry::MaterialType::Matte,
                };
                let triangle = Triangle {
                    a,
                    b,
                    c,
                    mat: geometry::Material { color, t: mat_type },
                    id,
                };

                triangles.push(triangle);
            }
            _ => println!("Invalid line"),
        }
    }

    let image_size = 2;
    let pixel_width = image_size as f32 / pixel_count as f32;
    let mut img: image::RgbImage = image::ImageBuffer::new(pixel_count, pixel_count);

    let start_pos = vec(0.0, 0.0, 0.0);
    let light_pos = vec(-3.0, 8.0, -6.0);

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
