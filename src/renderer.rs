use std::{
    fs::File,
    io::{self, Write},
    path::Path,
    thread,
};

use crate::math::{Vec3, hash3};
use crate::scene::{Ray, Scene};

pub struct Camera {
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub distance: f32,
    pub fov_degrees: f32,
}

impl Camera {
    pub fn position(&self) -> Vec3 {
        let cp = self.pitch.cos();
        self.target
            + Vec3::new(self.yaw.sin() * cp, self.pitch.sin(), self.yaw.cos() * cp) * self.distance
    }

    pub fn ray(&self, u: f32, v: f32, aspect: f32) -> Ray {
        let origin = self.position();
        let forward = (self.target - origin).normalized();
        let right = forward.cross(Vec3::new(0.0, 1.0, 0.0)).normalized();
        let up = right.cross(forward).normalized();
        let scale = (self.fov_degrees.to_radians() * 0.5).tan();
        let direction =
            (forward + right * ((2.0 * u - 1.0) * aspect * scale) + up * ((1.0 - 2.0 * v) * scale))
                .normalized();
        Ray { origin, direction }
    }
}

pub struct Renderer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
    pub max_bounces: u32,
    pub samples_per_pixel: u32,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height * 4],
            max_bounces: 3,
            samples_per_pixel: 1,
        }
    }

    pub fn render(&mut self, scene: &Scene, camera: &Camera, time: f32) {
        let width = self.width;
        let height = self.height;
        let aspect = width as f32 / height as f32;
        let row_bytes = width * 4;
        let thread_count = thread::available_parallelism()
            .map_or(4, usize::from)
            .min(height);
        let rows_per_chunk = height.div_ceil(thread_count);
        let chunk_bytes = rows_per_chunk * row_bytes;
        let max_bounces = self.max_bounces;
        let samples_per_pixel = self.samples_per_pixel;

        thread::scope(|scope| {
            for (chunk_index, pixel_chunk) in self.pixels.chunks_mut(chunk_bytes).enumerate() {
                scope.spawn(move || {
                    let start_y = chunk_index * rows_per_chunk;
                    for (local_y, row) in pixel_chunk.chunks_mut(row_bytes).enumerate() {
                        let y = start_y + local_y;
                        if y >= height {
                            break;
                        }
                        for x in 0..width {
                            let mut color = Vec3::ZERO;
                            for sample in 0..samples_per_pixel {
                                let jitter_x = hash3(Vec3::new(
                                    x as f32,
                                    y as f32,
                                    sample as f32 * 7.13 + time.floor(),
                                ));
                                let jitter_y = hash3(Vec3::new(
                                    y as f32,
                                    sample as f32 * 11.71 + time.floor(),
                                    x as f32,
                                ));
                                let u = (x as f32 + jitter_x) / width as f32;
                                let v = (y as f32 + jitter_y) / height as f32;
                                color += trace(scene, camera.ray(u, v, aspect), max_bounces);
                            }
                            let color =
                                (color / samples_per_pixel as f32).clamp01().powf(1.0 / 2.2);
                            let index = x * 4;
                            row[index] = (color.x * 255.0) as u8;
                            row[index + 1] = (color.y * 255.0) as u8;
                            row[index + 2] = (color.z * 255.0) as u8;
                            row[index + 3] = 255;
                        }
                    }
                });
            }
        });
    }

    pub fn save_bmp(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let pixel_bytes = self.width * self.height * 4;
        let file_size = 54 + pixel_bytes;
        let mut file = File::create(path)?;
        file.write_all(b"BM")?;
        file.write_all(&(file_size as u32).to_le_bytes())?;
        file.write_all(&[0; 4])?;
        file.write_all(&54_u32.to_le_bytes())?;
        file.write_all(&40_u32.to_le_bytes())?;
        file.write_all(&(self.width as i32).to_le_bytes())?;
        file.write_all(&(self.height as i32).to_le_bytes())?;
        file.write_all(&1_u16.to_le_bytes())?;
        file.write_all(&32_u16.to_le_bytes())?;
        file.write_all(&0_u32.to_le_bytes())?;
        file.write_all(&(pixel_bytes as u32).to_le_bytes())?;
        file.write_all(&[0; 16])?;

        for row in self.pixels.chunks(self.width * 4).rev() {
            for pixel in row.chunks_exact(4) {
                file.write_all(&[pixel[2], pixel[1], pixel[0], pixel[3]])?;
            }
        }
        Ok(())
    }
}

fn trace(scene: &Scene, ray: Ray, depth: u32) -> Vec3 {
    let Some(hit) = scene.hit(ray, 0.002, 1000.0) else {
        return sky(ray.direction);
    };
    let material = scene.materials[hit.material];
    let albedo = material.sample_albedo(hit.point, hit.normal);
    let view = -ray.direction;

    let to_light = scene.light_position - hit.point;
    let light_distance = to_light.length();
    let light_direction = to_light / light_distance;
    let shadow_origin = hit.point + hit.normal * 0.004;
    let in_shadow = scene
        .hit(
            Ray {
                origin: shadow_origin,
                direction: light_direction,
            },
            0.002,
            light_distance - 0.01,
        )
        .is_some();
    let visibility = if in_shadow { 0.16 } else { 1.0 };
    let attenuation = 1.0 / (1.0 + light_distance * light_distance * 0.028);
    let diffuse = hit.normal.dot(light_direction).max(0.0) * visibility * attenuation;
    let half_vector = (light_direction + view).normalized();
    let shininess = 4.0 + (1.0 - material.roughness) * 124.0;
    let specular = hit.normal.dot(half_vector).max(0.0).powf(shininess)
        * material.specular
        * visibility
        * attenuation;
    let ambient = Vec3::new(0.17, 0.23, 0.31) * (0.55 + hit.normal.y.max(0.0) * 0.45);
    let mut color =
        albedo * ambient + albedo * scene.light_color * diffuse + scene.light_color * specular;
    color += material.emission;

    if depth == 0 {
        return color;
    }

    let cosine = view.dot(hit.normal).clamp(0.0, 1.0);
    let fresnel = material.reflectivity + (1.0 - material.reflectivity) * (1.0 - cosine).powi(5);
    if fresnel > 0.01 {
        let reflected = ray.direction.reflect(hit.normal).normalized();
        let reflected_color = trace(
            scene,
            Ray {
                origin: hit.point + hit.normal * 0.004,
                direction: reflected,
            },
            depth - 1,
        );
        color = color.lerp(reflected_color, fresnel.clamp(0.0, 0.88));
    }

    if material.transparency > 0.01 {
        let eta = if hit.front_face {
            1.0 / material.ior
        } else {
            material.ior
        };
        if let Some(refracted) = ray.direction.refract(hit.normal, eta) {
            let refracted_color = trace(
                scene,
                Ray {
                    origin: hit.point + refracted * 0.006,
                    direction: refracted.normalized(),
                },
                depth - 1,
            );
            let tint = refracted_color * albedo.lerp(Vec3::ONE, 0.72);
            color = color.lerp(tint, material.transparency * (1.0 - fresnel));
        }
    }
    color
}

fn sky(direction: Vec3) -> Vec3 {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let horizon = Vec3::new(1.0, 0.38, 0.18);
    let zenith = Vec3::new(0.16, 0.20, 0.48);
    let mut color = horizon.lerp(zenith, t.powf(0.7));
    let sun_direction = Vec3::new(-0.42, 0.62, -0.66).normalized();
    let sun = direction.dot(sun_direction).max(0.0);
    color += Vec3::new(1.0, 0.55, 0.22) * sun.powf(96.0) * 2.8;
    color += Vec3::new(1.0, 0.82, 0.56) * sun.powf(950.0) * 5.0;
    let cloud_noise =
        ((direction.x * 11.0 + direction.z * 8.0).sin() * (direction.z * 17.0).cos()).abs();
    if direction.y > 0.03 && direction.y < 0.35 && cloud_noise > 0.72 {
        color = color.lerp(Vec3::new(1.0, 0.61, 0.46), 0.18);
    }
    color
}
