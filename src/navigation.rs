//! Deterministic navigation on the original BHT two-pixel grid.
use crate::{importer::ImportedLevel, simulation::Point};
use std::{cmp::Reverse, collections::BinaryHeap};
pub const CELL: f32 = 2.;
pub const CLEARANCE: f32 = 2.;
#[derive(Clone, Debug)]
pub struct Navigation {
    pub width: usize,
    pub height: usize,
    open: Vec<bool>,
}
impl Navigation {
    pub fn build(map: &ImportedLevel, obstacles: &[(Point, f32)]) -> Self {
        let width = map.width * 8;
        let height = map.height * 8;
        let open = (0..width * height)
            .map(|i| {
                let p = Point::new(
                    (i % width) as f32 * CELL + 1.,
                    (i / width) as f32 * CELL + 1.,
                );
                [
                    (0., 0.),
                    (-CLEARANCE, 0.),
                    (CLEARANCE, 0.),
                    (0., -CLEARANCE),
                    (0., CLEARANCE),
                    (-1.4, -1.4),
                    (1.4, 1.4),
                    (-1.4, 1.4),
                    (1.4, -1.4),
                ]
                .iter()
                .all(|&(x, y)| map.walkable(p.x + x, p.y + y))
                    && obstacles
                        .iter()
                        .all(|(o, r)| p.distance(*o) > r + CLEARANCE)
            })
            .collect();
        Self {
            width,
            height,
            open,
        }
    }
    pub fn point(&self, i: usize) -> Point {
        Point::new(
            (i % self.width) as f32 * CELL + 1.,
            (i / self.width) as f32 * CELL + 1.,
        )
    }
    fn cell(&self, p: Point) -> Option<usize> {
        if !p.x.is_finite()
            || !p.y.is_finite()
            || p.x < 0.
            || p.y < 0.
            || p.x >= self.width as f32 * CELL
            || p.y >= self.height as f32 * CELL
        {
            None
        } else {
            Some(p.y as usize / 2 * self.width + p.x as usize / 2)
        }
    }
    pub fn clear(&self, p: Point) -> bool {
        self.cell(p).is_some_and(|i| self.open[i])
    }
    pub fn segment(&self, a: Point, b: Point) -> bool {
        let count = a.distance(b).ceil().max(1.) as usize;
        (0..=count).all(|i| self.clear(a.toward(b, a.distance(b) * i as f32 / count as f32)))
    }
    pub fn route(&self, start: Point, goal: Point) -> Option<Vec<Point>> {
        if !goal.x.is_finite() || !goal.y.is_finite() {
            return None;
        }
        let goal = Point::new(
            goal.x.clamp(1., self.width as f32 * CELL - 1.),
            goal.y.clamp(1., self.height as f32 * CELL - 1.),
        );
        if self.segment(start, goal) {
            return Some(vec![goal]);
        }
        let source = self.cell(start)?;
        // Legacy sprite anchors can start on a mask edge; find a nearby clear starting cell.
        let source = if self.open[source] {
            source
        } else {
            (0..self.open.len())
                .filter(|&i| self.open[i] && self.point(i).distance(start) < 8.)
                .min_by(|&a, &b| {
                    self.point(a)
                        .distance(start)
                        .total_cmp(&self.point(b).distance(start))
                })?
        };
        let target = self.cell(goal)?;
        let heuristic = |i: usize| {
            let x = (i % self.width).abs_diff(target % self.width) as u32;
            let y = (i / self.width).abs_diff(target / self.width) as u32;
            10 * x.max(y) + 4 * x.min(y)
        };
        let mut cost = vec![u32::MAX; self.open.len()];
        let mut parent = vec![usize::MAX; self.open.len()];
        let mut heap = BinaryHeap::new();
        cost[source] = 0;
        heap.push(Reverse((heuristic(source), 0, source)));
        let mut best = source;
        while let Some(Reverse((_, g, i))) = heap.pop() {
            if g != cost[i] {
                continue;
            }
            if heuristic(i) < heuristic(best) {
                best = i;
            }
            if i == target {
                best = i;
                break;
            }
            let x = (i % self.width) as isize;
            let y = (i / self.width) as isize;
            for (dx, dy) in [
                (0, -1),
                (-1, 0),
                (1, 0),
                (0, 1),
                (-1, -1),
                (1, -1),
                (-1, 1),
                (1, 1),
            ] {
                let nx = x + dx;
                let ny = y + dy;
                if nx < 0 || ny < 0 || nx >= self.width as isize || ny >= self.height as isize {
                    continue;
                }
                let next = ny as usize * self.width + nx as usize;
                if !self.open[next] {
                    continue;
                }
                if dx != 0
                    && dy != 0
                    && (!self.open[y as usize * self.width + nx as usize]
                        || !self.open[ny as usize * self.width + x as usize])
                {
                    continue;
                }
                let next_cost = g + if dx == 0 || dy == 0 { 10 } else { 14 };
                if next_cost < cost[next] {
                    cost[next] = next_cost;
                    parent[next] = i;
                    heap.push(Reverse((next_cost + heuristic(next), next_cost, next)));
                }
            }
        }
        if best == source {
            return None;
        }
        let mut path = vec![self.point(best)];
        let mut cursor = best;
        while cursor != source {
            cursor = parent[cursor];
            path.push(self.point(cursor));
        }
        path.reverse();
        let mut simplified = vec![];
        let mut anchor = start;
        let mut first = 0;
        while first < path.len() {
            let mut far = first;
            for next in first..path.len() {
                if self.segment(anchor, path[next]) {
                    far = next;
                }
            }
            simplified.push(path[far]);
            anchor = path[far];
            first = far + 1;
        }
        Some(simplified)
    }
}
