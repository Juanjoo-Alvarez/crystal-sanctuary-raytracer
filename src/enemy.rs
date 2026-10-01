use crate::{
    character::{CHARACTER_MATERIAL_COUNT, CHARACTER_MATERIAL_START},
    math::Vec3,
    scene::{Block, Material, MaterialKind},
};

pub const ENEMY_MATERIAL_START: usize = CHARACTER_MATERIAL_START + CHARACTER_MATERIAL_COUNT;

#[derive(Clone, Copy, Debug)]
pub enum EnemyKind {
    Goomba,
    ShyGuy,
    Biddybud,
    Boo,
    Peepa,
    InnertubeGoomba,
    Stingby,
}

pub struct Enemy {
    pub kind: EnemyKind,
    pub position: Vec3,
    origin: Vec3,
    axis: Vec3,
    range: f32,
    speed: f32,
    phase: f32,
    blocks: Vec<Block>,
}

impl Enemy {
    fn new(kind: EnemyKind, origin: Vec3, axis: Vec3, range: f32, speed: f32, phase: f32) -> Self {
        let mut enemy = Self {
            kind,
            position: origin,
            origin,
            axis: axis.normalized(),
            range,
            speed,
            phase,
            blocks: Vec::with_capacity(7),
        };
        enemy.rebuild_geometry();
        enemy
    }

    pub fn update(&mut self, dt: f32, observed: bool) {
        // Like the classic ghost, a Boo only advances while it is outside the
        // player's view. Peepas keep their circular patrol to differentiate them.
        if matches!(self.kind, EnemyKind::Boo) && observed {
            self.rebuild_geometry();
            return;
        }
        self.phase += dt * self.speed;
        let patrol = self.phase.sin() * self.range;
        let bob = match self.kind {
            EnemyKind::Boo | EnemyKind::Peepa | EnemyKind::Stingby => {
                0.18 + (self.phase * 1.7).sin() * 0.12
            }
            _ => (self.phase * 2.0).sin().abs() * 0.035,
        };
        self.position = self.origin + self.axis * patrol + Vec3::new(0.0, bob, 0.0);
        self.rebuild_geometry();
    }

    pub fn reset(&mut self) {
        self.position = self.origin;
        self.phase = 0.0;
        self.rebuild_geometry();
    }

    pub fn touches(&self, player_position: Vec3) -> bool {
        let delta = self.position - player_position;
        let horizontal = Vec3::new(delta.x, 0.0, delta.z).length();
        horizontal < 0.48 && delta.y.abs() < 0.75
    }

    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    fn rebuild_geometry(&mut self) {
        self.blocks.clear();
        let p = self.position;
        let m = |local| ENEMY_MATERIAL_START + local;
        let mut add = |offset: Vec3, size: Vec3, material| {
            self.blocks.push(Block::new(p + offset, size, material));
        };

        match self.kind {
            EnemyKind::Goomba => {
                add(Vec3::new(0.0, 0.24, 0.0), Vec3::new(0.58, 0.42, 0.48), m(0));
                add(
                    Vec3::new(0.0, 0.48, -0.02),
                    Vec3::new(0.48, 0.28, 0.42),
                    m(1),
                );
                add(
                    Vec3::new(-0.13, 0.53, -0.235),
                    Vec3::new(0.08, 0.13, 0.035),
                    m(2),
                );
                add(
                    Vec3::new(0.13, 0.53, -0.235),
                    Vec3::new(0.08, 0.13, 0.035),
                    m(2),
                );
                add(
                    Vec3::new(-0.22, 0.06, -0.03),
                    Vec3::new(0.28, 0.12, 0.42),
                    m(3),
                );
                add(
                    Vec3::new(0.22, 0.06, -0.03),
                    Vec3::new(0.28, 0.12, 0.42),
                    m(3),
                );
                add(
                    Vec3::new(0.0, 0.40, -0.235),
                    Vec3::new(0.16, 0.05, 0.03),
                    m(2),
                );
            }
            EnemyKind::ShyGuy => {
                add(Vec3::new(0.0, 0.24, 0.0), Vec3::new(0.48, 0.48, 0.42), m(3));
                add(
                    Vec3::new(0.0, 0.52, -0.03),
                    Vec3::new(0.38, 0.30, 0.32),
                    m(4),
                );
                add(
                    Vec3::new(-0.10, 0.55, -0.205),
                    Vec3::new(0.06, 0.10, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(0.10, 0.55, -0.205),
                    Vec3::new(0.06, 0.10, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(0.0, 0.45, -0.21),
                    Vec3::new(0.07, 0.07, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(-0.16, 0.04, 0.0),
                    Vec3::new(0.20, 0.08, 0.30),
                    m(0),
                );
                add(
                    Vec3::new(0.16, 0.04, 0.0),
                    Vec3::new(0.20, 0.08, 0.30),
                    m(0),
                );
            }
            EnemyKind::Biddybud => {
                add(Vec3::new(0.0, 0.22, 0.0), Vec3::new(0.48, 0.34, 0.46), m(7));
                add(
                    Vec3::new(0.0, 0.40, -0.03),
                    Vec3::new(0.36, 0.24, 0.34),
                    m(4),
                );
                add(
                    Vec3::new(-0.09, 0.44, -0.215),
                    Vec3::new(0.055, 0.08, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(0.09, 0.44, -0.215),
                    Vec3::new(0.055, 0.08, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(-0.22, 0.17, 0.0),
                    Vec3::new(0.16, 0.08, 0.30),
                    m(3),
                );
                add(
                    Vec3::new(0.22, 0.17, 0.0),
                    Vec3::new(0.16, 0.08, 0.30),
                    m(3),
                );
                add(Vec3::new(0.0, 0.08, 0.0), Vec3::new(0.30, 0.10, 0.34), m(0));
            }
            EnemyKind::Boo | EnemyKind::Peepa => {
                let body = if matches!(self.kind, EnemyKind::Boo) {
                    m(4)
                } else {
                    m(5)
                };
                add(Vec3::new(0.0, 0.42, 0.0), Vec3::new(0.58, 0.56, 0.48), body);
                add(
                    Vec3::new(-0.14, 0.51, -0.255),
                    Vec3::new(0.08, 0.14, 0.035),
                    m(2),
                );
                add(
                    Vec3::new(0.14, 0.51, -0.255),
                    Vec3::new(0.08, 0.14, 0.035),
                    m(2),
                );
                add(
                    Vec3::new(0.0, 0.34, -0.255),
                    Vec3::new(0.18, 0.12, 0.035),
                    m(3),
                );
                add(
                    Vec3::new(-0.20, 0.12, 0.04),
                    Vec3::new(0.18, 0.18, 0.30),
                    body,
                );
                add(
                    Vec3::new(0.02, 0.10, 0.05),
                    Vec3::new(0.16, 0.16, 0.28),
                    body,
                );
                add(
                    Vec3::new(0.25, 0.08, 0.08),
                    Vec3::new(0.16, 0.12, 0.24),
                    body,
                );
            }
            EnemyKind::InnertubeGoomba => {
                add(Vec3::new(0.0, 0.26, 0.0), Vec3::new(0.72, 0.18, 0.62), m(6));
                add(Vec3::new(0.0, 0.45, 0.0), Vec3::new(0.48, 0.38, 0.42), m(0));
                add(
                    Vec3::new(0.0, 0.64, -0.02),
                    Vec3::new(0.38, 0.22, 0.36),
                    m(1),
                );
                add(
                    Vec3::new(-0.11, 0.66, -0.21),
                    Vec3::new(0.07, 0.11, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(0.11, 0.66, -0.21),
                    Vec3::new(0.07, 0.11, 0.03),
                    m(2),
                );
                add(
                    Vec3::new(-0.24, 0.18, 0.0),
                    Vec3::new(0.18, 0.12, 0.42),
                    m(8),
                );
                add(
                    Vec3::new(0.24, 0.18, 0.0),
                    Vec3::new(0.18, 0.12, 0.42),
                    m(8),
                );
            }
            EnemyKind::Stingby => {
                add(Vec3::new(0.0, 0.42, 0.0), Vec3::new(0.42, 0.48, 0.40), m(7));
                add(
                    Vec3::new(0.0, 0.42, -0.23),
                    Vec3::new(0.24, 0.16, 0.08),
                    m(2),
                );
                add(
                    Vec3::new(-0.10, 0.50, -0.28),
                    Vec3::new(0.05, 0.07, 0.03),
                    m(4),
                );
                add(
                    Vec3::new(0.10, 0.50, -0.28),
                    Vec3::new(0.05, 0.07, 0.03),
                    m(4),
                );
                add(
                    Vec3::new(-0.28, 0.43, 0.0),
                    Vec3::new(0.24, 0.08, 0.42),
                    m(4),
                );
                add(
                    Vec3::new(0.28, 0.43, 0.0),
                    Vec3::new(0.24, 0.08, 0.42),
                    m(4),
                );
                add(
                    Vec3::new(0.0, 0.14, 0.16),
                    Vec3::new(0.10, 0.24, 0.10),
                    m(2),
                );
            }
        }
    }
}

pub fn materials() -> Vec<Material> {
    let solid = |albedo: Vec3, specular: f32, emission: Vec3| Material {
        kind: MaterialKind::Solid,
        albedo,
        specular,
        reflectivity: specular * 0.08,
        transparency: 0.0,
        ior: 1.0,
        roughness: 0.68,
        emission,
    };
    vec![
        solid(Vec3::new(0.38, 0.16, 0.05), 0.18, Vec3::ZERO),
        solid(Vec3::new(0.82, 0.55, 0.28), 0.12, Vec3::ZERO),
        solid(Vec3::new(0.025, 0.025, 0.035), 0.55, Vec3::ZERO),
        solid(Vec3::new(0.82, 0.035, 0.045), 0.20, Vec3::ZERO),
        solid(
            Vec3::new(0.90, 0.94, 1.0),
            0.40,
            Vec3::new(0.06, 0.08, 0.13),
        ),
        solid(
            Vec3::new(0.58, 0.40, 0.78),
            0.30,
            Vec3::new(0.06, 0.02, 0.12),
        ),
        solid(
            Vec3::new(0.04, 0.62, 0.78),
            0.65,
            Vec3::new(0.01, 0.08, 0.12),
        ),
        solid(Vec3::new(0.94, 0.64, 0.04), 0.24, Vec3::ZERO),
        solid(Vec3::new(0.03, 0.20, 0.44), 0.42, Vec3::ZERO),
    ]
}

pub fn for_theme(theme: usize) -> Vec<Enemy> {
    match theme {
        1 => vec![
            Enemy::new(
                EnemyKind::Boo,
                Vec3::new(-3.5, -0.55, -0.5),
                Vec3::new(0.0, 0.0, 1.0),
                1.1,
                1.4,
                0.0,
            ),
            Enemy::new(
                EnemyKind::Peepa,
                Vec3::new(2.9, 0.12, -0.4),
                Vec3::new(1.0, 0.0, 0.0),
                0.65,
                1.8,
                1.7,
            ),
            Enemy::new(
                EnemyKind::Boo,
                Vec3::new(3.8, -0.55, 3.25),
                Vec3::new(1.0, 0.0, 0.0),
                0.65,
                1.2,
                3.1,
            ),
        ],
        2 => vec![
            Enemy::new(
                EnemyKind::InnertubeGoomba,
                Vec3::new(-3.6, -0.80, 0.0),
                Vec3::new(0.0, 0.0, 1.0),
                0.8,
                1.3,
                0.0,
            ),
            Enemy::new(
                EnemyKind::InnertubeGoomba,
                Vec3::new(3.0, -0.10, -0.35),
                Vec3::new(1.0, 0.0, 0.0),
                0.55,
                1.5,
                2.0,
            ),
            Enemy::new(
                EnemyKind::Stingby,
                Vec3::new(3.9, -0.55, 3.25),
                Vec3::new(1.0, 0.0, 0.0),
                0.6,
                1.9,
                4.0,
            ),
        ],
        _ => vec![
            Enemy::new(
                EnemyKind::Goomba,
                Vec3::new(-3.6, -0.80, -0.25),
                Vec3::new(0.0, 0.0, 1.0),
                0.75,
                1.4,
                0.0,
            ),
            Enemy::new(
                EnemyKind::ShyGuy,
                Vec3::new(3.0, -0.10, -0.40),
                Vec3::new(1.0, 0.0, 0.0),
                0.55,
                1.3,
                1.8,
            ),
            Enemy::new(
                EnemyKind::Biddybud,
                Vec3::new(3.8, -0.80, 3.30),
                Vec3::new(1.0, 0.0, 0.0),
                0.65,
                1.8,
                3.2,
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boo_freezes_while_observed() {
        let mut boo = Enemy::new(
            EnemyKind::Boo,
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
            1.0,
            1.0,
            0.0,
        );
        boo.update(0.5, true);
        assert!((boo.position - Vec3::ZERO).length() < 0.0001);
        boo.update(0.5, false);
        assert!(boo.position.length() > 0.1);
    }
}
