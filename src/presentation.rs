use raylib::prelude::*;

use crate::scene::LevelKind;

#[derive(Clone, Copy)]
enum ParticleShape {
    Leaf,
    Wisp,
    Bubble,
    Spark,
}

#[derive(Clone, Copy)]
struct Particle {
    position: Vector2,
    velocity: Vector2,
    life: f32,
    lifetime: f32,
    size: f32,
    rotation: f32,
    color: Color,
    shape: ParticleShape,
}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    seed: u32,
    ambient_timer: f32,
}

impl ParticleSystem {
    pub fn new() -> Self {
        Self {
            particles: Vec::with_capacity(180),
            seed: 0x2485_6A17,
            ambient_timer: 0.0,
        }
    }

    fn random(&mut self) -> f32 {
        self.seed = self
            .seed
            .wrapping_mul(1_664_525)
            .wrapping_add(1_013_904_223);
        ((self.seed >> 8) as f32) / 16_777_215.0
    }

    pub fn clear(&mut self) {
        self.particles.clear();
        self.ambient_timer = 0.0;
    }

    pub fn update(&mut self, dt: f32, width: f32, height: f32, level: LevelKind) {
        self.ambient_timer -= dt;
        if self.ambient_timer <= 0.0 && self.particles.len() < 150 {
            self.ambient_timer = match level {
                LevelKind::Hills => 0.13,
                LevelKind::Haunted => 0.10,
                LevelKind::Aquatic => 0.08,
            };
            let x = self.random() * width;
            let drift = self.random();
            let (position, velocity, lifetime, size, color, shape) = match level {
                LevelKind::Hills => (
                    Vector2::new(x, -12.0),
                    Vector2::new(-24.0 - drift * 30.0, 28.0 + self.random() * 30.0),
                    5.0 + self.random() * 2.5,
                    3.0 + self.random() * 3.0,
                    Color::new(210, 239, 119, 150),
                    ParticleShape::Leaf,
                ),
                LevelKind::Haunted => (
                    Vector2::new(x, height + 12.0),
                    Vector2::new(-8.0 + drift * 16.0, -17.0 - self.random() * 22.0),
                    5.5 + self.random() * 3.0,
                    5.0 + self.random() * 6.0,
                    Color::new(126, 247, 220, 105),
                    ParticleShape::Wisp,
                ),
                LevelKind::Aquatic => (
                    Vector2::new(x, height + 8.0),
                    Vector2::new(-6.0 + drift * 12.0, -24.0 - self.random() * 34.0),
                    4.0 + self.random() * 3.0,
                    2.0 + self.random() * 5.0,
                    Color::new(154, 244, 255, 120),
                    ParticleShape::Bubble,
                ),
            };
            let rotation = self.random() * 360.0;
            self.particles.push(Particle {
                position,
                velocity,
                life: lifetime,
                lifetime,
                size,
                rotation,
                color,
                shape,
            });
        }

        for particle in &mut self.particles {
            particle.life -= dt;
            particle.position.x += particle.velocity.x * dt;
            particle.position.y += particle.velocity.y * dt;
            particle.rotation += dt * 75.0;
            if matches!(particle.shape, ParticleShape::Leaf | ParticleShape::Wisp) {
                particle.position.x += (particle.life * 2.3).sin() * 9.0 * dt;
            }
        }
        self.particles.retain(|particle| {
            particle.life > 0.0
                && particle.position.x > -40.0
                && particle.position.x < width + 40.0
                && particle.position.y > -40.0
                && particle.position.y < height + 40.0
        });
    }

    pub fn burst(&mut self, position: Vector2, level: LevelKind, amount: usize) {
        let color = match level {
            LevelKind::Hills => Color::new(255, 219, 105, 245),
            LevelKind::Haunted => Color::new(118, 255, 228, 245),
            LevelKind::Aquatic => Color::new(107, 231, 255, 245),
        };
        for index in 0..amount {
            let angle = self.random() * std::f32::consts::TAU;
            let speed = 45.0 + self.random() * 150.0;
            let lifetime = 0.65 + self.random() * 0.75;
            let size = 2.0 + self.random() * 4.5 + (index % 3) as f32;
            self.particles.push(Particle {
                position,
                velocity: Vector2::new(angle.cos() * speed, angle.sin() * speed - 20.0),
                life: lifetime,
                lifetime,
                size,
                rotation: angle.to_degrees(),
                color,
                shape: ParticleShape::Spark,
            });
        }
    }

    pub fn draw(&self, drawing: &mut impl RaylibDraw) {
        for particle in &self.particles {
            let normalized = (particle.life / particle.lifetime).clamp(0.0, 1.0);
            let alpha = (particle.color.a as f32 * normalized.min(0.82) / 0.82) as u8;
            let color = Color::new(particle.color.r, particle.color.g, particle.color.b, alpha);
            match particle.shape {
                ParticleShape::Leaf => drawing.draw_rectangle_pro(
                    Rectangle::new(
                        particle.position.x,
                        particle.position.y,
                        particle.size * 1.7,
                        particle.size,
                    ),
                    Vector2::new(particle.size * 0.85, particle.size * 0.5),
                    particle.rotation,
                    color,
                ),
                ParticleShape::Wisp => {
                    drawing.draw_circle_v(particle.position, particle.size * 1.8, color);
                    drawing.draw_circle_v(
                        particle.position,
                        particle.size * 0.7,
                        Color::new(226, 255, 250, alpha),
                    );
                }
                ParticleShape::Bubble => {
                    drawing.draw_circle_lines(
                        particle.position.x as i32,
                        particle.position.y as i32,
                        particle.size,
                        color,
                    );
                    drawing.draw_circle_v(
                        particle.position
                            + Vector2::new(-particle.size * 0.25, -particle.size * 0.25),
                        (particle.size * 0.18).max(1.0),
                        color,
                    );
                }
                ParticleShape::Spark => drawing.draw_poly(
                    particle.position,
                    4,
                    particle.size,
                    particle.rotation,
                    color,
                ),
            }
        }
    }
}

pub fn ease_out_back(value: f32) -> f32 {
    let x = value.clamp(0.0, 1.0) - 1.0;
    1.0 + 2.70158 * x * x * x + 1.70158 * x * x
}

pub fn smoothstep(value: f32) -> f32 {
    let value = value.clamp(0.0, 1.0);
    value * value * (3.0 - 2.0 * value)
}

pub fn level_copy(level: LevelKind) -> (&'static str, &'static str, Color) {
    match level {
        LevelKind::Hills => (
            "COLINAS DEL SANTUARIO",
            "El último fragmento espera sobre la cumbre",
            Color::new(255, 211, 105, 255),
        ),
        LevelKind::Haunted => (
            "MANSION DE LOS BOO",
            "Una ruta oculta conduce al campanario",
            Color::new(127, 250, 220, 255),
        ),
        LevelKind::Aquatic => (
            "RUINAS DE LA MAREA",
            "La marea abre paso hacia el faro",
            Color::new(105, 225, 255, 255),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn draw_centered(
    drawing: &mut impl RaylibDraw,
    font: &Font,
    text: &str,
    center_x: f32,
    y: f32,
    size: f32,
    spacing: f32,
    color: Color,
) {
    let width = font.measure_text(text, size, spacing).x;
    drawing.draw_text_ex(
        font,
        text,
        Vector2::new(center_x - width * 0.5, y),
        size,
        spacing,
        color,
    );
}

pub fn draw_star(
    drawing: &mut impl RaylibDraw,
    center: Vector2,
    outer_radius: f32,
    inner_radius: f32,
    color: Color,
) {
    let mut points = [Vector2::zero(); 10];
    for (index, point) in points.iter_mut().enumerate() {
        let radius = if index % 2 == 0 {
            outer_radius
        } else {
            inner_radius
        };
        let angle = -std::f32::consts::FRAC_PI_2 + index as f32 * std::f32::consts::PI / 5.0;
        *point = center + Vector2::new(angle.cos() * radius, angle.sin() * radius);
    }
    for index in 0..10 {
        drawing.draw_triangle(center, points[(index + 1) % 10], points[index], color);
    }
}
