use std::f32::consts::PI;

use crate::{
    math::Vec3,
    scene::{Block, Material, MaterialKind},
};

pub const CHARACTER_MATERIAL_START: usize = 6;
pub const DEFAULT_CHARACTER_SCALE: f32 = 0.50;
const CHARACTER_BLOCK_CAPACITY: usize = 24;

pub struct Character {
    pub position: Vec3,
    pub rotation_y: f32,
    pub scale: f32,
    pub is_moving: bool,
    walk_timer: f32,
    blocks: Vec<Block>,
}

impl Character {
    pub fn new(position: Vec3, rotation_y: f32, scale: f32) -> Self {
        let mut character = Self {
            position,
            rotation_y,
            scale,
            is_moving: false,
            walk_timer: 0.0,
            blocks: Vec::with_capacity(CHARACTER_BLOCK_CAPACITY),
        };
        character.rebuild_geometry();
        character
    }

    pub fn update(&mut self, dt: f32, move_direction: Vec3) {
        self.is_moving = move_direction.length() > 0.001;
        if self.is_moving {
            let direction = move_direction.normalized();
            self.position += direction * (1.75 * dt);
            self.rotation_y = direction.x.atan2(direction.z) + PI;
            self.walk_timer += dt * 11.0;
        }
        self.rebuild_geometry();
    }

    pub fn reset(&mut self, position: Vec3) {
        self.position = position;
        self.rotation_y = PI;
        self.is_moving = false;
        self.walk_timer = 0.0;
        self.rebuild_geometry();
    }

    pub fn rebuild_geometry(&mut self) {
        self.blocks.clear();

        let bob = if self.is_moving {
            (self.walk_timer * 2.0).sin() * 0.035
        } else {
            0.0
        };
        let leg_swing = if self.is_moving {
            self.walk_timer.sin() * 0.15
        } else {
            0.0
        };
        let arm_swing = -leg_swing;
        let cos_rotation = self.rotation_y.cos();
        let sin_rotation = self.rotation_y.sin();
        let position = self.position;
        let scale = self.scale;

        let transform = |point: Vec3| {
            let scaled = point * scale;
            Vec3::new(
                scaled.x * cos_rotation + scaled.z * sin_rotation,
                scaled.y + bob,
                -scaled.x * sin_rotation + scaled.z * cos_rotation,
            ) + position
        };

        let mut add_box = |min: Vec3, max: Vec3, material: usize| {
            let center = transform((min + max) * 0.5);
            let size = (max - min) * scale;
            self.blocks.push(Block::new(center, size, material));
        };
        let material = |local: usize| CHARACTER_MATERIAL_START + local;

        // Shoes, body, belt and vest.
        add_box(
            Vec3::new(-0.35, 0.0, -0.20 + leg_swing),
            Vec3::new(-0.05, 0.22, 0.20 + leg_swing),
            material(10),
        );
        add_box(
            Vec3::new(0.05, 0.0, -0.20 - leg_swing),
            Vec3::new(0.35, 0.22, 0.20 - leg_swing),
            material(10),
        );
        add_box(
            Vec3::new(-0.30, 0.22, -0.25),
            Vec3::new(0.30, 0.36, 0.25),
            material(10),
        );
        add_box(
            Vec3::new(-0.32, 0.36, -0.27),
            Vec3::new(0.32, 0.43, 0.27),
            material(8),
        );
        add_box(
            Vec3::new(-0.33, 0.43, -0.28),
            Vec3::new(0.33, 0.76, 0.28),
            material(7),
        );

        // Backpack and swinging arms.
        add_box(
            Vec3::new(-0.28, 0.38, 0.28),
            Vec3::new(0.28, 0.79, 0.57),
            material(9),
        );
        add_box(
            Vec3::new(-0.49, 0.45, -0.10 + arm_swing),
            Vec3::new(-0.33, 0.68, 0.10 + arm_swing),
            material(3),
        );
        add_box(
            Vec3::new(0.33, 0.45, -0.10 - arm_swing),
            Vec3::new(0.49, 0.68, 0.10 - arm_swing),
            material(3),
        );

        // Scarf, face, eyes, highlights and smile.
        add_box(
            Vec3::new(-0.35, 0.76, -0.30),
            Vec3::new(0.35, 0.85, 0.30),
            material(6),
        );
        add_box(
            Vec3::new(-0.31, 0.85, -0.25),
            Vec3::new(0.31, 1.27, 0.25),
            material(3),
        );
        for x in [-0.18, 0.08] {
            add_box(
                Vec3::new(x, 1.00, -0.265),
                Vec3::new(x + 0.10, 1.17, -0.245),
                material(4),
            );
            add_box(
                Vec3::new(x + 0.03, 1.10, -0.275),
                Vec3::new(x + 0.08, 1.15, -0.255),
                material(5),
            );
        }
        add_box(
            Vec3::new(-0.09, 0.90, -0.27),
            Vec3::new(0.09, 0.97, -0.245),
            material(11),
        );

        // Mushroom cap, yellow rim and red spots.
        add_box(
            Vec3::new(-0.47, 1.27, -0.47),
            Vec3::new(0.47, 1.36, 0.47),
            material(2),
        );
        add_box(
            Vec3::new(-0.57, 1.36, -0.57),
            Vec3::new(0.57, 1.88, 0.57),
            material(0),
        );
        add_box(
            Vec3::new(-0.21, 1.44, -0.585),
            Vec3::new(0.21, 1.76, -0.555),
            material(1),
        );
        add_box(
            Vec3::new(-0.585, 1.44, -0.21),
            Vec3::new(-0.555, 1.76, 0.21),
            material(1),
        );
        add_box(
            Vec3::new(0.555, 1.44, -0.21),
            Vec3::new(0.585, 1.76, 0.21),
            material(1),
        );

        // Metallic headlamp with emissive lens.
        add_box(
            Vec3::new(-0.17, 1.39, -0.62),
            Vec3::new(0.17, 1.61, -0.54),
            material(12),
        );
        add_box(
            Vec3::new(-0.12, 1.43, -0.655),
            Vec3::new(0.12, 1.57, -0.615),
            material(13),
        );
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    pub fn bounding_box(&self) -> (Vec3, Vec3) {
        let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for block in &self.blocks {
            min.x = min.x.min(block.min.x);
            min.y = min.y.min(block.min.y);
            min.z = min.z.min(block.min.z);
            max.x = max.x.max(block.max.x);
            max.y = max.y.max(block.max.y);
            max.z = max.z.max(block.max.z);
        }
        (min, max)
    }

    pub fn materials() -> Vec<Material> {
        let solid = |albedo: Vec3, specular: f32, reflectivity: f32, emission: Vec3| Material {
            kind: MaterialKind::Solid,
            albedo,
            specular,
            reflectivity,
            transparency: 0.0,
            ior: 1.0,
            roughness: 0.72,
            emission,
        };
        vec![
            solid(Vec3::new(0.95, 0.95, 0.92), 0.15, 0.02, Vec3::ZERO),
            solid(Vec3::new(0.85, 0.06, 0.06), 0.20, 0.03, Vec3::ZERO),
            solid(Vec3::new(0.95, 0.72, 0.08), 0.22, 0.04, Vec3::ZERO),
            solid(Vec3::new(0.98, 0.72, 0.53), 0.18, 0.02, Vec3::ZERO),
            solid(Vec3::new(0.025, 0.025, 0.03), 0.72, 0.10, Vec3::ZERO),
            solid(Vec3::ONE, 0.80, 0.08, Vec3::ZERO),
            solid(Vec3::new(0.82, 0.05, 0.06), 0.15, 0.02, Vec3::ZERO),
            solid(Vec3::new(0.90, 0.84, 0.62), 0.12, 0.02, Vec3::ZERO),
            solid(Vec3::new(0.48, 0.28, 0.08), 0.30, 0.08, Vec3::ZERO),
            solid(Vec3::new(0.38, 0.19, 0.065), 0.14, 0.02, Vec3::ZERO),
            solid(Vec3::new(0.72, 0.035, 0.04), 0.24, 0.04, Vec3::ZERO),
            solid(Vec3::new(0.18, 0.025, 0.03), 0.12, 0.01, Vec3::ZERO),
            solid(Vec3::new(0.72, 0.75, 0.80), 0.92, 0.62, Vec3::ZERO),
            solid(
                Vec3::new(1.0, 0.84, 0.28),
                0.75,
                0.10,
                Vec3::new(1.8, 1.25, 0.35),
            ),
        ]
    }
}
