use std::mem::size_of;

use crate::prelude::*;
use crate::primitive::{Intersection, Primitive};

use super::AggregateT;

const INVALID_IDX: usize = usize::MAX;

// NOTE: using a u32 instead of a usize (on 64 bit) makes intersection code notably faster (~10%).
// It also places a platform independent limit of 2^32 nodes in a bvh.
type IndexType = u32;

struct LeafNode {
    num_primitives: IndexType,
    index: IndexType,
}

struct InteriorNode {
    children: [IndexType; 2],
}

enum BvhNodeType {
    Leaf(LeafNode),
    Interior(InteriorNode),
}

struct BvhNode {
    bounds: Bounds3,
    node_type: BvhNodeType,
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
    primitives: Vec<Primitive>,
}

#[derive(Debug)]
struct PrimitiveInfo {
    index: usize,
    bounds: Bounds3,
    centroid: Point3,
}

pub enum SplitMethod {
    EqualCounts,
    Sah,
}

impl Bvh {
    /// Generate the optimal SAH Bvh from a set of primitives by brute force.
    /// Mostly just here for comparison and because its easy to implement.
    pub fn new_by_brute_force(primitives: Vec<Primitive>) -> Self {
        let mut nodes = primitives
            .iter()
            .enumerate()
            .map(|(i, p)| BvhNode {
                bounds: p.make_bounds(),
                node_type: BvhNodeType::Leaf(LeafNode {
                    num_primitives: 1,
                    index: i as IndexType,
                }),
            })
            .collect::<Vec<_>>();

        nodes.reserve(primitives.len() - 1);

        let mut active = (0..primitives.len()).collect::<Vec<_>>();

        while nodes.len() < primitives.len() * 2 - 1 {
            let mut best = f32::INFINITY;
            let mut bounds = Bounds3::default();
            let mut children = [INVALID_IDX, INVALID_IDX];

            let mut i = INVALID_IDX;
            let mut j = INVALID_IDX;
            for (i1, n1) in active.iter().enumerate() {
                for (j1, n2) in active[(i1 + 1)..].iter().enumerate() {
                    let b = nodes[*n1].bounds.union(nodes[*n2].bounds);
                    let s = b.surface_area();
                    if s <= best {
                        best = s;
                        bounds = b;
                        children = [*n1, *n2];
                        i = i1;
                        j = j1 + i1 + 1;
                    }
                }
            }

            active.swap_remove(j);
            active[i] = nodes.len();

            nodes.push(BvhNode {
                bounds,
                node_type: BvhNodeType::Interior(InteriorNode {
                    children: children.map(|c| c as IndexType),
                }),
            });
        }

        Self { nodes, primitives }
    }

    fn recursive_split(
        nodes: &mut Vec<BvhNode>,
        primitive_info: &mut Vec<PrimitiveInfo>,
        primitives: &[Primitive],
        ordered_primitives: &mut Vec<Primitive>,
        start: usize,
        end: usize,
    ) -> usize {
        assert_ne!(start, end);
        let mut bounds = primitive_info[start].bounds;
        primitive_info
            .iter()
            .take(end)
            .skip(start + 1)
            .for_each(|p| bounds = bounds.union(p.bounds));

        let n_primitives = end - start;

        if n_primitives == 1 {
            ordered_primitives.push(primitives[primitive_info[start].index].clone());
            nodes.push(BvhNode {
                bounds,
                node_type: BvhNodeType::Leaf(LeafNode {
                    num_primitives: 1 as IndexType,
                    index: (ordered_primitives.len() - 1) as IndexType,
                }),
            });
        } else {
            let mut centroid_bounds = Bounds3::from_point(primitive_info[start].centroid);
            primitive_info
                .iter()
                .take(end)
                .skip(start + 1)
                .for_each(|pi| centroid_bounds = centroid_bounds.expand(pi.centroid));

            let dim = centroid_bounds.max_extent_idx();

            let mut middle = (start + end) / 2;
            let mut split_by_equal_counts = || {
                primitive_info[start..end].select_nth_unstable_by((end - start) / 2, |a, b| {
                    b.centroid[dim].partial_cmp(&a.centroid[dim]).unwrap()
                });
            };
            match SplitMethod::Sah {
                SplitMethod::EqualCounts => split_by_equal_counts(),
                SplitMethod::Sah => {
                    if n_primitives == 2 {
                        split_by_equal_counts();
                    } else {
                        const NUM_BUCKETS: usize = 12;
                        let mut buckets = [(0, Bounds3::default()); NUM_BUCKETS];

                        let get_bucket = |p: &PrimitiveInfo| {
                            ((NUM_BUCKETS as f32 * centroid_bounds.offset(p.centroid)[dim])
                                as usize)
                                .min(NUM_BUCKETS - 1)
                        };
                        primitive_info.iter().take(end).skip(start).for_each(|pi| {
                            let b = get_bucket(pi);
                            buckets[b].0 += 1;
                            buckets[b].1 = buckets[b].1.union(pi.bounds);
                        });

                        let mut costs = [0.0; NUM_BUCKETS - 1];
                        for i in 0..NUM_BUCKETS - 1 {
                            let mut b0 = buckets[0].1;
                            let mut b1 = buckets[i + 1].1;
                            let mut c0 = buckets[0].0;
                            let mut c1 = buckets[i + 1].0;

                            for (c, b) in buckets.iter().take(i + 1).skip(1) {
                                b0 = b0.union(*b);
                                c0 += c;
                            }

                            for (c, b) in buckets.iter().take(NUM_BUCKETS).skip(i + 1 + 1) {
                                b1 = b1.union(*b);
                                c1 += c;
                            }

                            costs[i] = 1.0
                                + (c0 as f32 * b0.surface_area() + c1 as f32 * b1.surface_area())
                                    / bounds.surface_area()
                        }
                        let mut min_cost = costs[0];
                        let mut min_idx = 0;

                        costs
                            .iter()
                            .enumerate()
                            .take(NUM_BUCKETS - 1)
                            .skip(1)
                            .for_each(|(i, &cost)| {
                                if cost < min_cost {
                                    min_cost = cost;
                                    min_idx = i;
                                }
                            });

                        let leaf_cost = n_primitives as f32;
                        const MAX_PRIMITIVES_IN_LEAF: usize = 4;

                        if n_primitives > MAX_PRIMITIVES_IN_LEAF || min_cost < leaf_cost {
                            middle = start
                                + primitive_info[start..end]
                                    .iter_mut()
                                    .partition_in_place(|p| {
                                        let b = get_bucket(p);
                                        assert!(b < NUM_BUCKETS);
                                        b <= min_idx
                                    });
                        } else {
                            let index = ordered_primitives.len();
                            for i in start..end {
                                ordered_primitives
                                    .push(primitives[primitive_info[i].index].clone());
                            }
                            nodes.push(BvhNode {
                                bounds,
                                node_type: BvhNodeType::Leaf(LeafNode {
                                    num_primitives: n_primitives as IndexType,
                                    index: index as IndexType,
                                }),
                            });
                            return nodes.len() - 1;
                        }
                    }
                }
            };

            let a = Self::recursive_split(
                nodes,
                primitive_info,
                primitives,
                ordered_primitives,
                start,
                middle,
            );
            let b = Self::recursive_split(
                nodes,
                primitive_info,
                primitives,
                ordered_primitives,
                middle,
                end,
            );
            nodes.push(BvhNode {
                bounds,
                node_type: BvhNodeType::Interior(InteriorNode {
                    children: [a as IndexType, b as IndexType],
                }),
            });
        }

        nodes.len() - 1
    }

    pub fn new_by_recursive_split(primitives: Vec<Primitive>) -> Self {
        let mut primitive_infos = primitives
            .iter()
            .enumerate()
            .map(|(index, p)| {
                let bounds = p.make_bounds();
                PrimitiveInfo {
                    index,
                    bounds,
                    centroid: bounds.centroid(),
                }
            })
            .collect::<Vec<_>>();

        let mut nodes = Vec::with_capacity(primitives.len() * 2 - 1);
        let end = primitive_infos.len();

        let mut ordered_primitives = Vec::with_capacity(primitives.len());
        Self::recursive_split(
            &mut nodes,
            &mut primitive_infos,
            &primitives,
            &mut ordered_primitives,
            0,
            end,
        );

        Self {
            nodes,
            primitives: ordered_primitives,
        }
    }

    pub fn new(primitives: Vec<Primitive>) -> Self {
        if primitives.len() * 2 - 1 > u32::MAX as usize {
            panic!(
                "too many primitives, the maximum number of supported primitives is 2^31. Found {}",
                primitives.len()
            );
        }

        if primitives.is_empty() {
            return Self {
                primitives,
                nodes: vec![],
            };
        }
        //NOTE: just here for testing since programmatically constructing scenes can leave them in a weird state
        // let mut rand = oorandom::Rand32::new(0);
        // primitives.sort_by_cached_key(|_| rand.rand_u32());

        let ret = Self::new_by_recursive_split(primitives);
        // let ret = Self::new_by_brute_force(primitives);

        STATS
            .primitive_memory
            .add((ret.primitives.len() * size_of::<Primitive>()) as u64);
        STATS
            .aggregate_memory
            .add((ret.nodes.len() * size_of::<BvhNode>() + size_of::<Self>()) as u64);

        ret
    }
}

impl AggregateT for Bvh {
    fn intersect_p(&self, ray: Ray) -> (Option<Intersection>, usize) {
        if self.nodes.is_empty() {
            return (None, 0);
        }

        // NOTE: this is in theory not enough to store the worst case number of nodes that are still yet to be visited.
        // However in practice this is more than double what's needed to store a million+ shape scene, so its *probably* fine.
        const TO_VISIT_LEN: usize = 64;
        let mut to_visit = [INVALID_IDX; TO_VISIT_LEN];
        let mut node_idx = self.nodes.len() - 1;
        let mut to_visit_idx = 0;

        let mut num_intersection_tests = 0;
        let mut intersection = None;
        let mut t_max = f32::MAX;

        loop {
            let node = &self.nodes[node_idx];

            num_intersection_tests += 1;
            if node.bounds.intersects(ray, 0.0, t_max) {
                match &node.node_type {
                    BvhNodeType::Interior(interior) => {
                        node_idx = interior.children[0] as usize;
                        to_visit_idx += 1;
                        to_visit[to_visit_idx] = interior.children[1] as usize;
                        continue;
                    }
                    BvhNodeType::Leaf(leaf) => {
                        self.primitives
                            .iter()
                            .skip(leaf.index as usize)
                            .take(leaf.num_primitives as usize)
                            .for_each(|p| {
                                num_intersection_tests += 1;
                                if let Some(i) = p.intersect(ray) {
                                    if i.shape_intersection.t < t_max {
                                        t_max = i.shape_intersection.t;
                                        intersection = Some(i);
                                    }
                                }
                            });
                    }
                }
            }

            if to_visit_idx == 0 {
                break;
            }
            node_idx = to_visit[to_visit_idx];
            to_visit_idx -= 1;
        }

        (intersection, num_intersection_tests)
    }

    fn bounds(&self) -> Bounds3 {
        self.nodes.last().map(|n| n.bounds).unwrap_or_default()
    }
}
