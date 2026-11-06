//! Renderer-independent, fixed-step battlefield simulation in original pixel units.
use crate::importer::ImportedLevel;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn distance(self, b: Self) -> f32 {
        ((self.x - b.x).powi(2) + (self.y - b.y).powi(2)).sqrt()
    }
    pub fn toward(self, b: Self, d: f32) -> Self {
        let n = self.distance(b);
        if n <= d {
            return b;
        }
        Self::new(
            self.x + (b.x - self.x) * d / n,
            self.y + (b.y - self.y) * d / n,
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Weapon {
    Rifle,
    Grenade,
    Rocket,
}
#[derive(Clone, Debug)]
pub struct Unit {
    pub id: u32,
    pub name: String,
    pub pos: Point,
    pub previous: Point,
    pub target: Point,
    pub aim: Point,
    pub enemy: bool,
    pub alive: bool,
    pub group: u8,
    pub cooldown: f32,
    pub fire: bool,
    pub rocket: bool,
}
#[derive(Clone, Debug)]
pub struct Building {
    pub pos: Point,
    pub health: i32,
    pub tiles: Vec<usize>,
    pub spawn: f32,
    pub vehicle: bool,
}
#[derive(Clone, Debug)]
pub struct Projectile {
    pub pos: Point,
    pub start: Point,
    pub target: Point,
    pub age: f32,
    pub duration: f32,
    pub enemy: bool,
    pub weapon: Weapon,
}
#[derive(Clone, Debug)]
pub struct Pickup {
    pub pos: Point,
    pub weapon: Weapon,
    pub active: bool,
}
#[derive(Clone, Debug)]
pub enum Effect {
    Shot(Point),
    Explosion(Point),
    Death(Point),
    Pickup(Point),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Playing,
    Victory,
    Defeat,
}
#[derive(Clone, Debug)]
pub enum Command {
    Move(Point),
    Aim(Option<Point>),
    Explosive(Point),
    Select(u32),
    Group(u8),
    Split,
    Join,
    Weapon(Weapon),
}
#[derive(Clone, Debug)]
pub struct Battle {
    pub map: ImportedLevel,
    pub units: Vec<Unit>,
    pub buildings: Vec<Building>,
    pub projectiles: Vec<Projectile>,
    pub pickups: Vec<Pickup>,
    pub effects: Vec<Effect>,
    pub active_group: u8,
    pub selected: Option<u32>,
    pub weapon: Weapon,
    pub grenades: u32,
    pub rockets: u32,
    pub outcome: Outcome,
    pub clock: f32,
    pub destroy_objective: bool,
    pub kill_objective: bool,
    pub seed: u64,
    pub hazards: Vec<Point>,
}
impl Battle {
    pub fn new(map: ImportedLevel, seed: u64) -> Self {
        let units = map
            .sprites
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.kind, 0 | 5 | 36))
            .map(|(i, s)| {
                let p = Point::new(s.x() as f32, s.y as f32);
                Unit {
                    id: i as u32,
                    name: format!("Recruit {}", i + 1),
                    pos: p,
                    previous: p,
                    target: p,
                    aim: p,
                    enemy: s.kind != 0,
                    alive: true,
                    group: 0,
                    cooldown: 0.5 + (i % 7) as f32 * 0.1,
                    fire: false,
                    rocket: s.kind == 36,
                }
            })
            .collect();
        let mut buildings: Vec<_> = map
            .structures
            .iter()
            .map(|s| Building {
                pos: Point::new(s.x, s.y),
                health: 1,
                tiles: s.tiles.clone(),
                spawn: 12.,
                vehicle: false,
            })
            .collect();
        buildings.extend(
            map.sprites
                .iter()
                .filter(|s| matches!(s.kind,63..=65|69|80..=82))
                .map(|s| Building {
                    pos: Point::new(s.x() as f32, s.y as f32),
                    health: 2,
                    tiles: vec![],
                    spawn: f32::INFINITY,
                    vehicle: true,
                }),
        );
        let pickups = map
            .sprites
            .iter()
            .filter(|s| matches!(s.kind, 37 | 38))
            .map(|s| Pickup {
                pos: Point::new(s.x() as f32, s.y as f32),
                weapon: if s.kind == 37 {
                    Weapon::Grenade
                } else {
                    Weapon::Rocket
                },
                active: true,
            })
            .collect();
        let hazards = map
            .sprites
            .iter()
            .filter(|s| matches!(s.kind, 54..=56))
            .map(|s| Point::new(s.x() as f32, s.y as f32))
            .collect();
        Self {
            map,
            units,
            buildings,
            projectiles: vec![],
            pickups,
            effects: vec![],
            active_group: 0,
            selected: None,
            weapon: Weapon::Grenade,
            grenades: 0,
            rockets: 0,
            outcome: Outcome::Playing,
            clock: 0.,
            destroy_objective: true,
            kill_objective: true,
            seed,
            hazards,
        }
    }
    pub fn selected_unit(&self, u: &Unit) -> bool {
        u.alive
            && !u.enemy
            && self
                .selected
                .map_or(u.group == self.active_group, |id| id == u.id)
    }
    pub fn command(&mut self, c: Command) {
        if self.outcome != Outcome::Playing {
            return;
        }
        match c {
            Command::Select(id) => {
                if let Some(u) = self
                    .units
                    .iter()
                    .find(|u| u.id == id && u.alive && !u.enemy)
                {
                    self.active_group = u.group;
                    self.selected = Some(id);
                }
            }
            Command::Group(g) => {
                self.active_group = g;
                self.selected = None;
            }
            Command::Weapon(w) => self.weapon = w,
            Command::Split => {
                if let Some(id) = self.selected {
                    let g = (0..3).find(|g| {
                        !self
                            .units
                            .iter()
                            .any(|u| u.alive && !u.enemy && u.group == *g)
                    });
                    if let Some(g) = g {
                        if let Some(u) = self.units.iter_mut().find(|u| u.id == id) {
                            u.group = g;
                            self.active_group = g;
                            self.selected = None;
                        }
                    }
                }
            }
            Command::Join => {
                for u in self.units.iter_mut().filter(|u| !u.enemy) {
                    u.group = self.active_group;
                }
                self.selected = None;
            }
            Command::Move(p) => {
                let selected: Vec<_> = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| self.selected_unit(u))
                    .map(|(i, _)| i)
                    .collect();
                for (n, i) in selected.into_iter().enumerate() {
                    self.units[i].target =
                        Point::new(p.x + (n % 2) as f32 * 7., p.y + (n / 2) as f32 * 8.);
                    self.units[i].fire = false;
                }
            }
            Command::Aim(p) => {
                let selected: Vec<_> = self
                    .units
                    .iter()
                    .enumerate()
                    .filter(|(_, u)| self.selected_unit(u))
                    .map(|(i, _)| i)
                    .collect();
                for i in selected {
                    self.units[i].fire = p.is_some();
                    if let Some(p) = p {
                        self.units[i].aim = p;
                    }
                }
            }
            Command::Explosive(p) => {
                let Some(i) = self
                    .units
                    .iter()
                    .position(|u| self.selected_unit(u) && u.cooldown <= 0.)
                else {
                    return;
                };
                let ammo = match self.weapon {
                    Weapon::Grenade => &mut self.grenades,
                    Weapon::Rocket => &mut self.rockets,
                    Weapon::Rifle => return,
                };
                if *ammo == 0 {
                    return;
                }
                *ammo -= 1;
                let start = self.units[i].pos;
                self.units[i].cooldown = 0.65;
                self.units[i].aim = p;
                self.launch(start, p, false, self.weapon);
            }
        }
    }
    fn launch(&mut self, start: Point, target: Point, enemy: bool, weapon: Weapon) {
        let range = match weapon {
            Weapon::Rifle => 180.,
            Weapon::Grenade => 110.,
            Weapon::Rocket => 260.,
        };
        let end = start.toward(target, range);
        let distance = start.distance(end);
        self.projectiles.push(Projectile {
            pos: start,
            start,
            target: end,
            age: 0.,
            duration: match weapon {
                Weapon::Rifle => distance / 340.,
                Weapon::Grenade => 0.9,
                Weapon::Rocket => distance / 220.,
            }
            .max(0.02),
            enemy,
            weapon,
        });
        self.effects.push(Effect::Shot(start));
    }
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (self.seed >> 32) as u32 as f32 / u32::MAX as f32
    }
    pub fn tick(&mut self, dt: f32) {
        if self.outcome != Outcome::Playing {
            return;
        }
        self.clock += dt;
        let live: Vec<_> = self
            .units
            .iter()
            .filter(|u| u.alive)
            .map(|u| (u.id, u.pos, u.enemy))
            .collect();
        let mut shots = vec![];
        for u in &mut self.units {
            u.previous = u.pos;
            if !u.alive {
                continue;
            }
            u.cooldown = (u.cooldown - dt).max(0.);
            if u.enemy {
                if let Some((_, p, _)) = live
                    .iter()
                    .filter(|(_, _, e)| !*e)
                    .min_by(|a, b| u.pos.distance(a.1).total_cmp(&u.pos.distance(b.1)))
                {
                    let d = u.pos.distance(*p);
                    u.fire = d < 160.;
                    u.aim = *p;
                    if d < 220. && d > 90. {
                        u.target = *p;
                    } else {
                        u.target = u.pos;
                    }
                } else {
                    u.fire = false;
                }
            }
            if u.fire && u.cooldown <= 0. {
                let weapon = if u.enemy && u.rocket {
                    Weapon::Rocket
                } else {
                    Weapon::Rifle
                };
                shots.push((u.pos, u.aim, u.enemy, weapon));
                u.cooldown = if u.enemy {
                    if u.rocket { 1.7 } else { 0.8 }
                } else {
                    0.16
                };
            }
            if !u.fire {
                let terrain = self.map.terrain_at(u.pos.x, u.pos.y);
                let speed = if matches!(terrain, 5 | 6) { 20. } else { 48. };
                let next = u.pos.toward(u.target, dt * speed);
                let clear = |p: Point| {
                    self.map.walkable(p.x, p.y)
                        && self
                            .buildings
                            .iter()
                            .all(|b| b.health <= 0 || p.distance(b.pos) > 14.)
                };
                if clear(next) {
                    u.pos = next;
                } else if clear(Point::new(next.x, u.pos.y)) {
                    u.pos.x = next.x;
                } else if clear(Point::new(u.pos.x, next.y)) {
                    u.pos.y = next.y;
                }
            }
        }
        for (start, mut target, enemy, weapon) in shots {
            if enemy {
                target.x += (self.random() - 0.5) * 28.;
                target.y += (self.random() - 0.5) * 28.;
            }
            self.launch(start, target, enemy, weapon);
        }
        let mut explosions = vec![];
        let mut hits = vec![];
        for p in &mut self.projectiles {
            let old = p.pos;
            p.age += dt;
            let t = (p.age / p.duration).min(1.);
            p.pos = Point::new(
                p.start.x + (p.target.x - p.start.x) * t,
                p.start.y + (p.target.y - p.start.y) * t,
            );
            if p.weapon != Weapon::Grenade {
                let steps = (old.distance(p.pos) / 2.).ceil().max(1.) as usize;
                for step in 1..=steps {
                    let sample =
                        old.toward(p.pos, old.distance(p.pos) * step as f32 / steps as f32);
                    let hit = self
                        .units
                        .iter()
                        .position(|u| u.alive && u.enemy != p.enemy && u.pos.distance(sample) < 4.);
                    let wall = !self.map.walkable(sample.x, sample.y)
                        || self
                            .buildings
                            .iter()
                            .any(|b| b.health > 0 && b.pos.distance(sample) < 15.);
                    if hit.is_some() || wall {
                        p.pos = sample;
                        p.age = p.duration;
                        if p.weapon == Weapon::Rifle {
                            if let Some(i) = hit {
                                hits.push(i);
                            }
                        }
                        break;
                    }
                }
            }
            if p.age >= p.duration && p.weapon != Weapon::Rifle {
                explosions.push(p.pos);
            }
        }
        self.projectiles.retain(|p| p.age < p.duration);
        for i in hits {
            if self.units[i].alive {
                self.units[i].alive = false;
                self.effects.push(Effect::Death(self.units[i].pos));
            }
        }
        for p in &mut self.pickups {
            if p.active
                && self
                    .units
                    .iter()
                    .any(|u| u.alive && !u.enemy && u.pos.distance(p.pos) < 10.)
            {
                p.active = false;
                match p.weapon {
                    Weapon::Grenade => self.grenades += 6,
                    Weapon::Rocket => self.rockets += 4,
                    _ => {}
                }
                self.effects.push(Effect::Pickup(p.pos));
            }
        }
        self.hazards.retain(|p| {
            if self
                .units
                .iter()
                .any(|u| u.alive && u.pos.distance(*p) < 5.)
            {
                explosions.push(*p);
                false
            } else {
                true
            }
        });
        for p in explosions {
            self.explode(p);
        }
        let mut spawns = vec![];
        for b in &mut self.buildings {
            if b.health > 0 && !b.vehicle {
                b.spawn -= dt;
                if b.spawn <= 0. && live.iter().any(|(_, p, e)| !*e && p.distance(b.pos) < 220.) {
                    spawns.push(Point::new(b.pos.x, b.pos.y + 24.));
                    b.spawn = 15.;
                }
            }
        }
        for p in spawns {
            if self.units.iter().filter(|u| u.enemy && u.alive).count() < 60 {
                let id = self.units.iter().map(|u| u.id).max().unwrap_or(0) + 1;
                self.units.push(Unit {
                    id,
                    name: "Enemy".into(),
                    pos: p,
                    previous: p,
                    target: p,
                    aim: p,
                    enemy: true,
                    alive: true,
                    group: 0,
                    cooldown: 1.,
                    fire: false,
                    rocket: false,
                });
            }
        }
        if !self.units.iter().any(|u| !u.enemy && u.alive) {
            self.outcome = Outcome::Defeat;
        } else if (!self.kill_objective || !self.units.iter().any(|u| u.enemy && u.alive))
            && (!self.destroy_objective
                || !self.buildings.iter().any(|b| !b.vehicle && b.health > 0))
        {
            self.outcome = Outcome::Victory;
        }
    }
    pub fn explode(&mut self, p: Point) {
        self.effects.push(Effect::Explosion(p));
        for u in &mut self.units {
            if u.alive && u.pos.distance(p) < 27. {
                u.alive = false;
                self.effects.push(Effect::Death(u.pos));
            }
        }
        for b in &mut self.buildings {
            if b.health > 0 && b.pos.distance(p) < 42. {
                b.health -= 1;
                if b.health == 0 {
                    self.map.destroy_tiles(&b.tiles);
                    self.effects.push(Effect::Explosion(b.pos));
                }
            }
        }
    }
}
