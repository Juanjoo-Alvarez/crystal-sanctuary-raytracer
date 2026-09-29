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
                                color += trace(scene, camera.ray(u, v, aspect), max_bounces, time);
                            }
                            let mut color = color / samples_per_pixel as f32;
                            color = tone_map_aces(color * 0.82).powf(1.0 / 2.2);
                            let screen_x = x as f32 / width as f32 * 2.0 - 1.0;
                            let screen_y = y as f32 / height as f32 * 2.0 - 1.0;
                            let vignette = (1.04
                                - (screen_x * screen_x + screen_y * screen_y) * 0.13)
                                .clamp(0.72, 1.0);
                            color = (color * vignette).clamp01();
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

fn trace(scene: &Scene, ray: Ray, depth: u32, time: f32) -> Vec3 {
    let Some(hit) = scene.hit(ray, 0.002, 1000.0) else {
        return sky(ray.direction);
    };
    let material = scene.materials[hit.material];
    let albedo = material.sample_albedo(hit.point, hit.normal, time);
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
    let emission_pulse = if matches!(material.kind, crate::scene::MaterialKind::Crystal) {
        0.88 + (time * 2.4 + hit.point.y * 1.7).sin() * 0.12
    } else {
        1.0
    };
    color += material.emission * emission_pulse;

    if depth == 0 {
        return apply_atmosphere(color, ray.direction, hit.distance);
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
            time,
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
                time,
            );
            let tint = refracted_color * albedo.lerp(Vec3::ONE, 0.72);
            color = color.lerp(tint, material.transparency * (1.0 - fresnel));
        }
    }
    apply_atmosphere(color, ray.direction, hit.distance)
}

fn sky(direction: Vec3) -> Vec3 {
    let t = (direction.y * 0.5 + 0.5).clamp(0.0, 1.0);
    let horizon = Vec3::new(1.0, 0.46, 0.22);
    let zenith = Vec3::new(0.10, 0.17, 0.43);
    let mut color = horizon.lerp(zenith, t.powf(0.82));

    // Layered distant silhouettes give the floating island a real sense of depth.
    let azimuth = direction.x.atan2(direction.z);
    let far_ridge =
        0.025 + (azimuth * 2.4 + 0.9).sin() * 0.020 + (azimuth * 5.7 - 1.2).sin().abs() * 0.030;
    let near_ridge =
        0.005 + (azimuth * 1.7 - 0.3).sin() * 0.035 + (azimuth * 4.1 + 0.8).cos().abs() * 0.045;
    if direction.y < far_ridge {
        color = Vec3::new(0.24, 0.19, 0.30).lerp(Vec3::new(0.42, 0.22, 0.25), 0.45);
    }
    if direction.y < near_ridge {
        color = Vec3::new(0.075, 0.10, 0.16).lerp(Vec3::new(0.14, 0.12, 0.18), 0.35);
    }

    let sun_direction = Vec3::new(-0.42, 0.62, -0.66).normalized();
    let sun = direction.dot(sun_direction).max(0.0);
    color += Vec3::new(1.0, 0.44, 0.16) * sun.powf(52.0) * 1.8;
    color += Vec3::new(1.0, 0.86, 0.58) * sun.powf(720.0) * 7.0;

    // Broad soft cloud bands instead of a noisy checker pattern.
    if direction.y > 0.08 && direction.y < 0.48 {
        let cloud_shape = (azimuth * 5.0 + direction.y * 17.0).sin() * 0.46
            + (azimuth * 11.0 - direction.y * 9.0).cos() * 0.29
            + (azimuth * 19.0 + 1.7).sin() * 0.15;
        let cloud = ((cloud_shape - 0.24) * 2.5).clamp(0.0, 1.0)
            * ((0.48 - direction.y) / 0.40).clamp(0.0, 1.0);
        color = color.lerp(Vec3::new(1.0, 0.68, 0.55), cloud * 0.28);
    }
    color
}

fn apply_atmosphere(color: Vec3, direction: Vec3, distance: f32) -> Vec3 {
    let fog = (1.0 - (-distance * 0.018).exp()).clamp(0.0, 0.30);
    color.lerp(sky(direction), fog * 0.42)
}

fn tone_map_aces(color: Vec3) -> Vec3 {
    fn channel(value: f32) -> f32 {
        let numerator = value * (2.51 * value + 0.03);
        let denominator = value * (2.43 * value + 0.59) + 0.14;
        (numerator / denominator).clamp(0.0, 1.0)
    }

    Vec3::new(channel(color.x), channel(color.y), channel(color.z))
}
