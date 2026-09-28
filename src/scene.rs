use crate::math::{Vec3, hash3};

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
    pub fn sample_albedo(&self, point: Vec3, normal: Vec3) -> Vec3 {
        match self.kind {
            MaterialKind::MossStone => {
                let grid = ((point.x * 2.0).floor() as i32 + (point.z * 2.0).floor() as i32) & 1;
                let noise = hash3(Vec3::new(point.x.floor(), point.y.floor(), point.z.floor()));
                let stone = self.albedo * (0.78 + noise * 0.28);
                let moss = Vec3::new(0.16, 0.38, 0.12) * (0.8 + noise * 0.35);
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
                let shimmer = ((point.y * 8.0 + point.x * 5.0).sin() * 0.5 + 0.5) * 0.12;
                self.albedo + Vec3::new(0.0, shimmer * 0.5, shimmer)
            }
            MaterialKind::Water => {
                let ripples = ((point.x * 7.0).sin() + (point.z * 9.0).cos()) * 0.035;
                self.albedo + Vec3::new(0.0, ripples, ripples * 1.7)
            }
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
        let materials = vec![
            Material {
                kind: MaterialKind::MossStone,
                albedo: Vec3::new(0.31, 0.34, 0.25),
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

        // Floating island core and raised sanctuary.
        let mut blocks = vec![
            Block::new(Vec3::new(0.0, -1.3, 0.0), Vec3::new(10.0, 1.0, 8.0), 0),
            Block::new(Vec3::new(0.0, -2.15, 0.0), Vec3::new(8.0, 0.7, 6.0), 0),
            Block::new(Vec3::new(0.0, -2.8, 0.0), Vec3::new(5.6, 0.6, 4.2), 0),
            Block::new(Vec3::new(0.0, -3.35, 0.0), Vec3::new(3.0, 0.5, 2.2), 0),
            Block::new(Vec3::new(1.7, -0.45, 1.0), Vec3::new(5.4, 0.7, 4.2), 1),
        ];
        for i in 0..4 {
            blocks.push(Block::new(
                Vec3::new(
                    -1.7 + i as f32 * 0.48,
                    -0.72 + i as f32 * 0.24,
                    -1.6 + i as f32 * 0.44,
                ),
                Vec3::new(1.45, 0.25, 0.75),
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
            blocks.push(Block::new(
                Vec3::new(-3.25 + i as f32 * 0.48, -0.48, -2.35),
                Vec3::new(0.40, 0.16, 1.10),
                2,
            ));
        }
        blocks.push(Block::new(
            Vec3::new(-1.55, -0.22, -2.85),
            Vec3::new(4.0, 0.10, 0.10),
            2,
        ));
        blocks.push(Block::new(
            Vec3::new(-1.55, -0.22, -1.85),
            Vec3::new(4.0, 0.10, 0.10),
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
            (-4.1, 2.4, 0.7),
            (-3.4, 2.8, 0.45),
            (-2.8, 1.9, 0.6),
            (4.1, 2.8, 0.8),
            (3.7, 3.3, 0.5),
            (-4.3, -0.6, 0.55),
        ] {
            blocks.push(Block::new(
                Vec3::new(x, -0.75 + h * 0.5, z),
                Vec3::new(0.42, h, 0.42),
                0,
            ));
        }

        let mut scene = Self {
            blocks,
            materials,
            light_position: Vec3::new(-4.5, 8.5, -5.5),
            light_color: Vec3::new(1.0, 0.72, 0.48) * 6.0,
            bvh_nodes: Vec::new(),
            block_indices: Vec::new(),
        };
        scene.rebuild_bvh();
        scene
    }

    pub fn hit(&self, ray: Ray, min_distance: f32, max_distance: f32) -> Option<Hit> {
        let mut closest = max_distance;
        let mut result = None;

        let mut stack = [0_usize; 256];
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
}
