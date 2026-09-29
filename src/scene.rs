use crate::{
    character::{Character, DEFAULT_CHARACTER_SCALE},
    math::{Vec3, hash3},
};

#[derive(Clone, Copy)]
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

#[derive(Clone, Copy, Debug)]
pub enum MaterialKind {
    MossStone,
    Sandstone,
    Wood,
    Copper,
    Crystal,
    Water,
    Solid,
}

#[derive(Clone, Copy)]
pub struct Material {
    pub kind: MaterialKind,
    pub albedo: Vec3,
    pub specular: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub ior: f32,
    pub roughness: f32,
    pub emission: Vec3,
}

impl Material {
    pub fn sample_albedo(&self, point: Vec3, normal: Vec3, time: f32) -> Vec3 {
        match self.kind {
            MaterialKind::MossStone => {
                let grid = ((point.x * 2.0).floor() as i32 + (point.z * 2.0).floor() as i32) & 1;
                let noise = hash3(Vec3::new(point.x.floor(), point.y.floor(), point.z.floor()));
                let stone = self.albedo * (0.78 + noise * 0.28);
                let moss = Vec3::new(0.12, 0.50, 0.10) * (0.8 + noise * 0.35);
                if normal.y > 0.45 && (noise > 0.34 || grid == 0) {
                    stone.lerp(moss, 0.72)
                } else {
                    stone
                }
            }
            MaterialKind::Sandstone => {
                let bands = (point.y * 9.0 + (point.x * 2.7).sin()).sin() * 0.08;
                self.albedo * (0.94 + bands + hash3(point * 5.0) * 0.08)
            }
            MaterialKind::Wood => {
                let rings = ((point.x * 4.0 + point.z * 5.0).sin() * 0.5 + 0.5) * 0.22;
                let seams = if (point.y * 2.0).fract().abs() < 0.06 {
                    0.55
                } else {
                    1.0
                };
                self.albedo * (0.8 + rings) * seams
            }
            MaterialKind::Copper => {
                let patina = hash3(point * 3.0);
                if patina > 0.87 {
                    self.albedo.lerp(Vec3::new(0.08, 0.42, 0.36), 0.55)
                } else {
                    self.albedo
                }
            }
            MaterialKind::Crystal => {
                let shimmer =
                    ((point.y * 8.0 + point.x * 5.0 - time * 2.8).sin() * 0.5 + 0.5) * 0.12;
                self.albedo + Vec3::new(0.0, shimmer * 0.5, shimmer)
            }
            MaterialKind::Water => {
                let horizontal = (point.x * 7.0 + time * 1.7).sin();
                let vertical = (point.z * 9.0 - time * 2.2 + point.y * 5.0).cos();
                let ripples = (horizontal + vertical) * 0.035;
                self.albedo + Vec3::new(0.0, ripples, ripples * 1.7)
            }
            MaterialKind::Solid => self.albedo,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Block {
    pub min: Vec3,
    pub max: Vec3,
    pub material: usize,
}

impl Block {
    pub fn new(center: Vec3, size: Vec3, material: usize) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
            material,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Hit {
    pub distance: f32,
    pub point: Vec3,
    pub normal: Vec3,
    pub material: usize,
    pub front_face: bool,
}

pub struct Scene {
    pub blocks: Vec<Block>,
    pub materials: Vec<Material>,
    pub light_position: Vec3,
    pub light_color: Vec3,
    bvh_nodes: Vec<BvhNode>,
    block_indices: Vec<usize>,
    character_start: usize,
    character_count: usize,
    walkable_blocks: Vec<usize>,
    decorative_blocks: Vec<usize>,
    collectibles: Vec<Collectible>,
    crystal_activated: bool,
}

#[derive(Clone, Copy)]
struct Collectible {
    block_index: usize,
    position: Vec3,
    material: usize,
    active: bool,
}

#[derive(Clone, Copy)]
pub struct PuzzleStatus {
    pub collected: usize,
    pub total: usize,
    pub crystal_activated: bool,
}

#[derive(Clone, Copy)]
struct BvhNode {
    min: Vec3,
    max: Vec3,
    left: usize,
    right: usize,
    start: usize,
    count: usize,
}

impl Scene {
    pub fn sanctuary() -> Self {
        let mut materials = vec![
            Material {
                kind: MaterialKind::MossStone,
                albedo: Vec3::new(0.34, 0.42, 0.23),
                specular: 0.10,
                reflectivity: 0.03,
                transparency: 0.0,
                ior: 1.0,
                roughness: 0.88,
                emission: Vec3::ZERO,
            },
            Material {
                kind: MaterialKind::Sandstone,
                albedo: Vec3::new(0.72, 0.43, 0.24),
                specular: 0.18,
                reflectivity: 0.04,
                transparency: 0.0,
                ior: 1.0,
                roughness: 0.70,
                emission: Vec3::ZERO,
            },
            Material {
                kind: MaterialKind::Wood,
                albedo: Vec3::new(0.30, 0.13, 0.055),
                specular: 0.24,
                reflectivity: 0.06,
                transparency: 0.0,
                ior: 1.0,
                roughness: 0.58,
                emission: Vec3::ZERO,
            },
            Material {
                kind: MaterialKind::Copper,
                albedo: Vec3::new(0.76, 0.29, 0.12),
                specular: 0.92,
                reflectivity: 0.58,
                transparency: 0.0,
                ior: 1.0,
                roughness: 0.16,
                emission: Vec3::ZERO,
            },
            Material {
                kind: MaterialKind::Crystal,
                albedo: Vec3::new(0.10, 0.62, 0.92),
                specular: 0.95,
                reflectivity: 0.16,
                transparency: 0.78,
                ior: 1.46,
                roughness: 0.05,
                emission: Vec3::new(0.02, 0.16, 0.28),
            },
            Material {
                kind: MaterialKind::Water,
                albedo: Vec3::new(0.08, 0.48, 0.58),
                specular: 0.98,
                reflectivity: 0.12,
                transparency: 0.72,
                ior: 1.33,
                roughness: 0.04,
                emission: Vec3::ZERO,
            },
        ];
        materials.extend(Character::materials());
        let foliage_material = materials.len();
        materials.push(Material {
            kind: MaterialKind::Solid,
            albedo: Vec3::new(0.08, 0.34, 0.16),
            specular: 0.16,
            reflectivity: 0.025,
            transparency: 0.0,
            ior: 1.0,
            roughness: 0.82,
            emission: Vec3::ZERO,
        });
        let fragment_material = materials.len();
        materials.push(Material {
            kind: MaterialKind::Crystal,
            albedo: Vec3::new(0.12, 0.88, 1.0),
            specular: 0.98,
            reflectivity: 0.28,
            transparency: 0.22,
            ior: 1.42,
            roughness: 0.03,
            emission: Vec3::new(0.08, 0.65, 0.92),
        });

        // Floating island core and raised sanctuary.
        let mut blocks = vec![
            Block::new(Vec3::new(0.0, -1.3, 0.0), Vec3::new(10.0, 1.0, 8.0), 0),
            Block::new(Vec3::new(0.0, -2.15, 0.0), Vec3::new(8.0, 0.7, 6.0), 0),
            Block::new(Vec3::new(0.0, -2.8, 0.0), Vec3::new(5.6, 0.6, 4.2), 0),
            Block::new(Vec3::new(0.0, -3.35, 0.0), Vec3::new(3.0, 0.5, 2.2), 0),
            Block::new(Vec3::new(1.7, -0.45, 1.0), Vec3::new(5.4, 0.7, 4.2), 1),
        ];
        let mut walkable_blocks = vec![0, 1, 2, 3, 4];
        let mut decorative_blocks = Vec::new();
        // Wide, walkable steps connect the bridge level with the raised sanctuary.
        // Their top surfaces exactly match `ground_height`, avoiding invisible walls.
        for (index, (z, top)) in [
            (-1.68, -0.40),
            (-1.46, -0.30),
            (-1.24, -0.20),
            (-1.02, -0.10),
        ]
        .into_iter()
        .enumerate()
        {
            let height = top + 1.30;
            walkable_blocks.push(blocks.len());
            blocks.push(Block::new(
                Vec3::new(-0.36, top - height * 0.5, z),
                Vec3::new(1.36 + index as f32 * 0.02, height, 0.24),
                1,
            ));
        }

        // Four ruined columns frame the focal crystal.
        for (x, z, height) in [
            (-0.2, 0.0, 2.6),
            (3.5, 0.0, 3.1),
            (-0.2, 2.3, 2.1),
            (3.5, 2.3, 2.7),
        ] {
            blocks.push(Block::new(
                Vec3::new(x, height * 0.5, z),
                Vec3::new(0.55, height, 0.55),
                1,
            ));
            blocks.push(Block::new(
                Vec3::new(x, height + 0.08, z),
                Vec3::new(0.82, 0.18, 0.82),
                1,
            ));
        }

        // Wooden bridge over a narrow ravine.
        for i in 0..8 {
            walkable_blocks.push(blocks.len());
            blocks.push(Block::new(
                Vec3::new(-3.25 + i as f32 * 0.48, -0.48, -2.35),
                Vec3::new(0.40, 0.16, 1.42),
                2,
            ));
        }
        // The rails start after the western landing, leaving a generous entrance.
        blocks.push(Block::new(
            Vec3::new(-1.30, -0.22, -3.08),
            Vec3::new(3.40, 0.10, 0.10),
            2,
        ));
        blocks.push(Block::new(
            Vec3::new(-2.05, -0.22, -1.62),
            Vec3::new(1.90, 0.10, 0.10),
            2,
        ));

        // Copper mechanism: wheel-like cross and supports.
        blocks.push(Block::new(
            Vec3::new(4.25, 0.25, -1.85),
            Vec3::new(0.35, 2.8, 0.35),
            3,
        ));
        blocks.push(Block::new(
            Vec3::new(4.25, 1.65, -1.85),
            Vec3::new(2.3, 0.24, 0.30),
            3,
        ));
        blocks.push(Block::new(
            Vec3::new(4.25, 1.65, -1.85),
            Vec3::new(0.24, 2.3, 0.30),
            3,
        ));
        for &dx in &[-0.82, 0.82] {
            for &dy in &[-0.82, 0.82] {
                blocks.push(Block::new(
                    Vec3::new(4.25 + dx, 1.65 + dy, -1.85),
                    Vec3::new(0.34, 0.34, 0.34),
                    3,
                ));
            }
        }

        // Multi-block crystal gives a faceted, readable silhouette using only cubes.
        blocks.push(Block::new(
            Vec3::new(1.65, 0.95, 1.15),
            Vec3::new(0.95, 1.7, 0.95),
            4,
        ));
        blocks.push(Block::new(
            Vec3::new(1.65, 2.0, 1.15),
            Vec3::new(0.62, 0.62, 0.62),
            4,
        ));
        blocks.push(Block::new(
            Vec3::new(1.08, 0.78, 1.42),
            Vec3::new(0.35, 0.88, 0.35),
            4,
        ));

        // Reflective pool and waterfall make the refraction readable in context.
        blocks.push(Block::new(
            Vec3::new(1.65, -0.055, 1.18),
            Vec3::new(2.35, 0.10, 1.55),
            5,
        ));
        blocks.push(Block::new(
            Vec3::new(1.65, -0.13, 2.65),
            Vec3::new(1.05, 0.10, 1.55),
            5,
        ));
        blocks.push(Block::new(
            Vec3::new(1.65, -0.95, 3.46),
            Vec3::new(1.05, 1.70, 0.10),
            5,
        ));

        // Small decorative cubes create vegetation and visual rhythm.
        for (x, z, h) in [
            (-4.55, 1.7, 0.7),
            (-3.9, 3.5, 0.45),
            (-1.35, 3.45, 0.6),
            (4.55, 2.8, 0.8),
            (3.7, 3.3, 0.5),
            (-4.3, -0.6, 0.55),
        ] {
            blocks.push(Block::new(
                Vec3::new(x, -0.75 + h * 0.5, z),
                Vec3::new(0.42, h, 0.42),
                0,
            ));
        }

        // Silhouetted trees and shrubs make the island feel inhabited and frame the temple.
        for (x, z, height) in [(-4.65, 3.25, 1.55), (4.35, 3.15, 1.45)] {
            blocks.push(Block::new(
                Vec3::new(x, -0.80 + height * 0.36, z),
                Vec3::new(0.24, height * 0.72, 0.24),
                2,
            ));
            for (dx, dy, dz, scale) in [
                (0.0, 0.78, 0.0, 0.82),
                (-0.28, 0.58, 0.05, 0.60),
                (0.28, 0.56, -0.04, 0.58),
                (0.02, 1.06, 0.02, 0.52),
            ] {
                decorative_blocks.push(blocks.len());
                blocks.push(Block::new(
                    Vec3::new(x + dx, -0.80 + height * dy, z + dz),
                    Vec3::new(scale, scale * 0.72, scale),
                    foliage_material,
                ));
            }
        }
        for (x, z, scale) in [
            (-3.4, 3.25, 0.55),
            (-2.65, 3.45, 0.42),
            (3.35, -3.25, 0.48),
            (4.15, 0.55, 0.40),
            (-4.45, 0.75, 0.46),
        ] {
            decorative_blocks.push(blocks.len());
            blocks.push(Block::new(
                Vec3::new(x, -0.80 + scale * 0.42, z),
                Vec3::new(scale, scale * 0.72, scale),
                foliage_material,
            ));
        }

        // Two luminous wayfinding lanterns subtly lead from the spawn toward the sanctuary.
        for (x, z) in [(-2.15, 1.20), (3.25, -2.75)] {
            blocks.push(Block::new(
                Vec3::new(x, -0.24, z),
                Vec3::new(0.14, 1.12, 0.14),
                3,
            ));
            blocks.push(Block::new(
                Vec3::new(x, 0.38, z),
                Vec3::new(0.38, 0.12, 0.38),
                3,
            ));
            blocks.push(Block::new(
                Vec3::new(x, 0.55, z),
                Vec3::new(0.22, 0.28, 0.22),
                fragment_material,
            ));
            blocks.push(Block::new(
                Vec3::new(x, 0.74, z),
                Vec3::new(0.34, 0.10, 0.34),
                3,
            ));
        }

        // A broken arch and scattered masonry sell the history of the floating ruin.
        for x in [-3.90, -2.20] {
            blocks.push(Block::new(
                Vec3::new(x, 0.05, 1.45),
                Vec3::new(0.38, 1.70, 0.38),
                1,
            ));
        }
        blocks.push(Block::new(
            Vec3::new(-3.05, 0.86, 1.45),
            Vec3::new(2.08, 0.28, 0.38),
            1,
        ));
        for (x, z, rotation_hint) in [(-4.55, 0.95, 0.36), (4.45, -0.65, 0.30), (2.75, 3.55, 0.25)]
        {
            blocks.push(Block::new(
                Vec3::new(x, -0.62, z),
                Vec3::new(0.68, rotation_hint, 0.48),
                1,
            ));
        }

        // Three crystal fragments form the level's collection puzzle.
        let mut collectibles = Vec::new();
        for position in [
            Vec3::new(-3.15, -0.48, 0.55),
            Vec3::new(-2.15, -0.08, -2.35),
            Vec3::new(4.05, 0.22, 1.10),
        ] {
            let block_index = blocks.len();
            blocks.push(Block::new(
                position,
                Vec3::new(0.28, 0.46, 0.28),
                fragment_material,
            ));
            collectibles.push(Collectible {
                block_index,
                position,
                material: fragment_material,
                active: true,
            });
        }

        let character_start = blocks.len();
        let default_character =
            Character::new(Vec3::new(-2.15, -0.8, 2.50), 0.0, DEFAULT_CHARACTER_SCALE);
        blocks.extend_from_slice(default_character.blocks());
        let character_count = blocks.len() - character_start;

        let mut scene = Self {
            blocks,
            materials,
            light_position: Vec3::new(-4.5, 8.5, -5.5),
            light_color: Vec3::new(1.0, 0.72, 0.48) * 6.0,
            bvh_nodes: Vec::new(),
            block_indices: Vec::new(),
            character_start,
            character_count,
            walkable_blocks,
            decorative_blocks,
            collectibles,
            crystal_activated: false,
        };
        scene.rebuild_bvh();
        scene
    }

    pub fn update_character(&mut self, character: &[Block]) {
        debug_assert_eq!(character.len(), self.character_count);
        self.blocks[self.character_start..self.character_start + self.character_count]
            .copy_from_slice(character);
        self.rebuild_bvh();
    }

    pub fn ground_height(x: f32, z: f32) -> f32 {
        if (-1.0..=4.4).contains(&x) && (-1.1..=3.1).contains(&z) {
            -0.10
        } else if (-1.05..=0.32).contains(&x) && (-1.80..=-1.56).contains(&z) {
            -0.40
        } else if (-1.05..=0.32).contains(&x) && (-1.56..=-1.34).contains(&z) {
            -0.30
        } else if (-1.05..=0.32).contains(&x) && (-1.34..=-1.12).contains(&z) {
            -0.20
        } else if (-1.05..=0.32).contains(&x) && (-1.12..=-0.90).contains(&z) {
            -0.10
        } else if (-3.5..=0.3).contains(&x) && (-3.06..=-1.8).contains(&z) {
            -0.40
        } else {
            -0.80
        }
    }

    pub fn collides_with_world(&self, bounds_min: Vec3, bounds_max: Vec3) -> bool {
        const EDGE_MARGIN: f32 = 0.04;
        if bounds_min.x < -5.0 + EDGE_MARGIN
            || bounds_max.x > 5.0 - EDGE_MARGIN
            || bounds_min.z < -4.0 + EDGE_MARGIN
            || bounds_max.z > 4.0 - EDGE_MARGIN
        {
            return true;
        }

        self.blocks[..self.character_start]
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                !self.walkable_blocks.contains(index)
                    && !self.decorative_blocks.contains(index)
                    && !self
                        .collectibles
                        .iter()
                        .any(|collectible| collectible.block_index == *index)
            })
            .any(|(_, block)| aabbs_overlap(bounds_min, bounds_max, block.min, block.max))
    }

    pub fn collect_near(&mut self, player_position: Vec3) -> bool {
        let mut collected_any = false;
        for collectible in &mut self.collectibles {
            let offset = collectible.position - player_position;
            let horizontal_distance = Vec3::new(offset.x, 0.0, offset.z).length();
            if collectible.active && horizontal_distance < 0.72 {
                collectible.active = false;
                self.blocks[collectible.block_index] = Block::new(
                    Vec3::new(0.0, -100.0, 0.0),
                    Vec3::new(0.01, 0.01, 0.01),
                    collectible.material,
                );
                collected_any = true;
            }
        }
        if collected_any {
            self.rebuild_bvh();
        }
        collected_any
    }

    pub fn try_activate_crystal(&mut self, player_position: Vec3) -> bool {
        if self.crystal_activated || self.collectibles.iter().any(|item| item.active) {
            return false;
        }
        let crystal_position = Vec3::new(1.65, -0.10, 1.15);
        let offset = crystal_position - player_position;
        if Vec3::new(offset.x, 0.0, offset.z).length() < 1.15 {
            self.crystal_activated = true;
            self.light_color = Vec3::new(0.42, 0.82, 1.0) * 7.5;
            self.materials[4].albedo = Vec3::new(0.18, 0.82, 1.0);
            self.materials[4].emission = Vec3::new(0.18, 0.72, 1.18);
            true
        } else {
            false
        }
    }

    pub fn puzzle_status(&self) -> PuzzleStatus {
        let total = self.collectibles.len();
        let remaining = self.collectibles.iter().filter(|item| item.active).count();
        PuzzleStatus {
            collected: total - remaining,
            total,
            crystal_activated: self.crystal_activated,
        }
    }

    pub fn reset_puzzle(&mut self) {
        for collectible in &mut self.collectibles {
            collectible.active = true;
            self.blocks[collectible.block_index] = Block::new(
                collectible.position,
                Vec3::new(0.28, 0.46, 0.28),
                collectible.material,
            );
        }
        self.crystal_activated = false;
        self.light_color = Vec3::new(1.0, 0.72, 0.48) * 6.0;
        self.materials[4].albedo = Vec3::new(0.10, 0.62, 0.92);
        self.materials[4].emission = Vec3::new(0.02, 0.16, 0.28);
        self.rebuild_bvh();
    }

    pub fn hit(&self, ray: Ray, min_distance: f32, max_distance: f32) -> Option<Hit> {
        let mut closest = max_distance;
        let mut result = None;

        // A balanced BVH only needs logarithmic traversal depth. Keeping this
        // stack compact avoids clearing 2 KiB for every primary/shadow ray.
        let mut stack = [0_usize; 64];
        let mut stack_len = 1;
        while stack_len > 0 {
            stack_len -= 1;
            let node = self.bvh_nodes[stack[stack_len]];
            if intersect_bounds(ray, node.min, node.max, min_distance, closest).is_none() {
                continue;
            }

            if node.count > 0 {
                for &block_index in &self.block_indices[node.start..node.start + node.count] {
                    if let Some(hit) =
                        intersect_block(ray, self.blocks[block_index], min_distance, closest)
                    {
                        closest = hit.distance;
                        result = Some(hit);
                    }
                }
            } else {
                let left = self.bvh_nodes[node.left];
                let right = self.bvh_nodes[node.right];
                let left_distance =
                    intersect_bounds(ray, left.min, left.max, min_distance, closest);
                let right_distance =
                    intersect_bounds(ray, right.min, right.max, min_distance, closest);
                match (left_distance, right_distance) {
                    (Some(left_near), Some(right_near)) => {
                        let (near_child, far_child) = if left_near < right_near {
                            (node.left, node.right)
                        } else {
                            (node.right, node.left)
                        };
                        stack[stack_len] = far_child;
                        stack[stack_len + 1] = near_child;
                        stack_len += 2;
                    }
                    (Some(_), None) => {
                        stack[stack_len] = node.left;
                        stack_len += 1;
                    }
                    (None, Some(_)) => {
                        stack[stack_len] = node.right;
                        stack_len += 1;
                    }
                    (None, None) => {}
                }
            }
        }
        result
    }

    fn rebuild_bvh(&mut self) {
        self.block_indices = (0..self.blocks.len()).collect();
        self.bvh_nodes.clear();
        build_bvh_node(
            &self.blocks,
            &mut self.block_indices,
            &mut self.bvh_nodes,
            0,
            self.blocks.len(),
        );
    }
}

fn aabbs_overlap(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
    const EPSILON: f32 = 0.006;
    a_min.x < b_max.x - EPSILON
        && a_max.x > b_min.x + EPSILON
        && a_min.y < b_max.y - EPSILON
        && a_max.y > b_min.y + EPSILON
        && a_min.z < b_max.z - EPSILON
        && a_max.z > b_min.z + EPSILON
}

fn build_bvh_node(
    blocks: &[Block],
    indices: &mut [usize],
    nodes: &mut Vec<BvhNode>,
    start: usize,
    end: usize,
) -> usize {
    let mut min = Vec3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
    let mut max = Vec3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
    for &index in &indices[start..end] {
        min.x = min.x.min(blocks[index].min.x);
        min.y = min.y.min(blocks[index].min.y);
        min.z = min.z.min(blocks[index].min.z);
        max.x = max.x.max(blocks[index].max.x);
        max.y = max.y.max(blocks[index].max.y);
        max.z = max.z.max(blocks[index].max.z);
    }

    let node_index = nodes.len();
    nodes.push(BvhNode {
        min,
        max,
        left: 0,
        right: 0,
        start,
        count: end - start,
    });

    if end - start <= 4 {
        return node_index;
    }

    let extent = max - min;
    let axis = if extent.x > extent.y && extent.x > extent.z {
        0
    } else if extent.y > extent.z {
        1
    } else {
        2
    };
    indices[start..end].sort_unstable_by(|&left, &right| {
        let left_center = (blocks[left].min + blocks[left].max) * 0.5;
        let right_center = (blocks[right].min + blocks[right].max) * 0.5;
        let left_value = [left_center.x, left_center.y, left_center.z][axis];
        let right_value = [right_center.x, right_center.y, right_center.z][axis];
        left_value.total_cmp(&right_value)
    });

    let middle = start + (end - start) / 2;
    let left = build_bvh_node(blocks, indices, nodes, start, middle);
    let right = build_bvh_node(blocks, indices, nodes, middle, end);
    nodes[node_index].left = left;
    nodes[node_index].right = right;
    nodes[node_index].count = 0;
    node_index
}

fn intersect_bounds(
    ray: Ray,
    min: Vec3,
    max: Vec3,
    min_distance: f32,
    max_distance: f32,
) -> Option<f32> {
    let inv = Vec3::new(
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );
    let tx1 = (min.x - ray.origin.x) * inv.x;
    let tx2 = (max.x - ray.origin.x) * inv.x;
    let ty1 = (min.y - ray.origin.y) * inv.y;
    let ty2 = (max.y - ray.origin.y) * inv.y;
    let tz1 = (min.z - ray.origin.z) * inv.z;
    let tz2 = (max.z - ray.origin.z) * inv.z;
    let near = tx1.min(tx2).max(ty1.min(ty2)).max(tz1.min(tz2));
    let far = tx1.max(tx2).min(ty1.max(ty2)).min(tz1.max(tz2));
    if far >= near.max(min_distance) && near <= max_distance {
        Some(near.max(min_distance))
    } else {
        None
    }
}

fn intersect_block(ray: Ray, block: Block, min_distance: f32, max_distance: f32) -> Option<Hit> {
    let inv = Vec3::new(
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );
    let tx1 = (block.min.x - ray.origin.x) * inv.x;
    let tx2 = (block.max.x - ray.origin.x) * inv.x;
    let ty1 = (block.min.y - ray.origin.y) * inv.y;
    let ty2 = (block.max.y - ray.origin.y) * inv.y;
    let tz1 = (block.min.z - ray.origin.z) * inv.z;
    let tz2 = (block.max.z - ray.origin.z) * inv.z;

    let near = tx1.min(tx2).max(ty1.min(ty2)).max(tz1.min(tz2));
    let far = tx1.max(tx2).min(ty1.max(ty2)).min(tz1.max(tz2));
    if far < near || far < min_distance || near > max_distance {
        return None;
    }

    let (distance, front_face) = if near >= min_distance {
        (near, true)
    } else {
        (far, false)
    };
    if distance < min_distance || distance > max_distance {
        return None;
    }
    let point = ray.origin + ray.direction * distance;
    let center = (block.min + block.max) * 0.5;
    let half = (block.max - block.min) * 0.5;
    let local = point - center;
    let scaled = Vec3::new(
        local.x.abs() / half.x,
        local.y.abs() / half.y,
        local.z.abs() / half.z,
    );
    let outward = if scaled.x > scaled.y && scaled.x > scaled.z {
        Vec3::new(local.x.signum(), 0.0, 0.0)
    } else if scaled.y > scaled.z {
        Vec3::new(0.0, local.y.signum(), 0.0)
    } else {
        Vec3::new(0.0, 0.0, local.z.signum())
    };
    let actual_front = ray.direction.dot(outward) < 0.0;
    let normal = if actual_front { outward } else { -outward };
    Some(Hit {
        distance,
        point,
        normal,
        material: block.material,
        front_face: actual_front && front_face,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_front_face_of_block() {
        let block = Block::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0), 0);
        let ray = Ray {
            origin: Vec3::new(0.0, 0.0, -4.0),
            direction: Vec3::new(0.0, 0.0, 1.0),
        };
        let hit = intersect_block(ray, block, 0.001, 100.0).expect("ray should hit");
        assert!((hit.distance - 3.0).abs() < 0.0001);
        assert!(hit.front_face);
        assert!(hit.normal.z < -0.99);
    }

    #[test]
    fn ray_can_exit_transparent_block() {
        let block = Block::new(Vec3::ZERO, Vec3::new(2.0, 2.0, 2.0), 0);
        let ray = Ray {
            origin: Vec3::ZERO,
            direction: Vec3::new(1.0, 0.0, 0.0),
        };
        let hit = intersect_block(ray, block, 0.001, 100.0).expect("ray should exit");
        assert!((hit.distance - 1.0).abs() < 0.0001);
        assert!(!hit.front_face);
        assert!(hit.normal.x < -0.99);
    }

    #[test]
    fn character_update_moves_geometry_and_rebuilds_bvh() {
        let mut scene = Scene::sanctuary();
        let original_min = scene.blocks[scene.character_start].min;
        let mut character = Character::new(Vec3::new(2.0, -0.10, 1.0), 1.2, 1.0);
        character.rebuild_geometry();
        scene.update_character(character.blocks());
        let moved_min = scene.blocks[scene.character_start].min;
        assert!((moved_min.x - original_min.x).abs() > 1.0);
        assert!(!scene.bvh_nodes.is_empty());
    }

    #[test]
    fn ground_height_matches_main_platforms() {
        assert!((Scene::ground_height(0.0, 0.0) + 0.10).abs() < 0.001);
        assert!((Scene::ground_height(-2.0, -2.3) + 0.40).abs() < 0.001);
        assert!((Scene::ground_height(-3.0, 0.0) + 0.80).abs() < 0.001);
        assert!((Scene::ground_height(-0.4, -1.68) + 0.40).abs() < 0.001);
        assert!((Scene::ground_height(-0.4, -1.46) + 0.30).abs() < 0.001);
        assert!((Scene::ground_height(-0.4, -1.24) + 0.20).abs() < 0.001);
        assert!((Scene::ground_height(-0.4, -1.02) + 0.10).abs() < 0.001);
    }

    #[test]
    fn collision_detects_column_but_not_spawn() {
        let scene = Scene::sanctuary();
        let spawn = Character::new(Vec3::new(-2.15, -0.8, 2.50), 0.0, DEFAULT_CHARACTER_SCALE);
        let (spawn_min, spawn_max) = spawn.bounding_box();
        assert!(!scene.collides_with_world(spawn_min, spawn_max));

        let column = Character::new(Vec3::new(-0.2, -0.10, 0.0), 0.0, DEFAULT_CHARACTER_SCALE);
        let (column_min, column_max) = column.bounding_box();
        assert!(scene.collides_with_world(column_min, column_max));
    }

    #[test]
    fn bridge_fragment_and_steps_are_walkable() {
        let scene = Scene::sanctuary();
        for (x, y, z) in [
            (-3.70, -0.80, -2.35),
            (-3.35, -0.40, -2.35),
            (-2.15, -0.40, -2.35),
            (-0.36, -0.40, -1.68),
            (-0.36, -0.30, -1.46),
            (-0.36, -0.20, -1.24),
            (-0.36, -0.10, -1.02),
        ] {
            let character = Character::new(Vec3::new(x, y, z), 0.0, DEFAULT_CHARACTER_SCALE);
            let (bounds_min, bounds_max) = character.bounding_box();
            assert!(
                !scene.collides_with_world(bounds_min, bounds_max),
                "expected walkable position at ({x}, {y}, {z})"
            );
        }
    }

    #[test]
    fn collectible_routes_have_clear_standing_space() {
        let scene = Scene::sanctuary();
        for (x, z) in [(-3.15, 0.55), (-3.05, 1.45), (4.05, 1.10)] {
            let y = Scene::ground_height(x, z);
            let character = Character::new(Vec3::new(x, y, z), 0.0, DEFAULT_CHARACTER_SCALE);
            let (bounds_min, bounds_max) = character.bounding_box();
            assert!(
                !scene.collides_with_world(bounds_min, bounds_max),
                "expected clear objective route at ({x}, {y}, {z})"
            );
        }

        let bush = Character::new(
            Vec3::new(4.15, Scene::ground_height(4.15, 0.55), 0.55),
            0.0,
            DEFAULT_CHARACTER_SCALE,
        );
        let (bounds_min, bounds_max) = bush.bounding_box();
        assert!(!scene.collides_with_world(bounds_min, bounds_max));
    }

    #[test]
    fn collecting_all_fragments_unlocks_crystal() {
        let mut scene = Scene::sanctuary();
        let positions: Vec<Vec3> = scene
            .collectibles
            .iter()
            .map(|item| item.position)
            .collect();
        for position in positions {
            assert!(scene.collect_near(position));
        }
        let status = scene.puzzle_status();
        assert_eq!(status.collected, status.total);
        assert!(scene.try_activate_crystal(Vec3::new(1.65, -0.10, 1.15)));
        assert!(scene.puzzle_status().crystal_activated);
    }
}
