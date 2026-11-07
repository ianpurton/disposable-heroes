//! Renderer-independent, fixed-step battlefield simulation in original pixel units.
use crate::importer::ImportedLevel;
use crate::{navigation::Navigation, tuning::CombatTuning};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiState {
    Idle,
    Reacting,
    Engaging,
    Searching,
}
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
    pub path: VecDeque<Point>,
    pub history: VecDeque<Point>,
    pub follow: Option<u32>,
    pub slot: usize,
    pub ai: AiState,
    pub home: Point,
    pub aggression: u8,
    pub acquired: Option<u32>,
    pub reaction: f32,
    pub last_seen: Option<Point>,
    pub memory: f32,
    pub repath: f32,
    pub pose: Option<(Weapon, f32)>,
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
    Shot {
        origin: Point,
        direction: Point,
        weapon: Weapon,
    },
    Impact(Point),
    Order {
        position: Point,
        accepted: bool,
    },
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
    pub invalid_objectives: bool,
    /// Explicit indices of designated enemy structures. None uses imported enemy huts.
    pub objective_buildings: Option<Vec<usize>>,
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
    pub navigation: Navigation,
    pub tuning: CombatTuning,
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
                    path: VecDeque::new(),
                    history: VecDeque::from([p]),
                    follow: None,
                    slot: 0,
                    ai: AiState::Idle,
                    home: p,
                    aggression: 255,
                    acquired: None,
                    reaction: 0.,
                    last_seen: None,
                    memory: 0.,
                    repath: 0.,
                    pose: None,
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
        let navigation = Navigation::build(
            &map,
            &buildings
                .iter()
                .filter(|b| b.health > 0)
                .map(|b| (b.pos, 14.))
                .collect::<Vec<_>>(),
        );
        Self {
            invalid_objectives: false,
            objective_buildings: Some((0..map.structures.len()).collect()),
            navigation,
            tuning: CombatTuning::default(),
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
                let Some(&leader_index) = selected.first() else {
                    return;
                };
                let leader_id = self.units[leader_index].id;
                let Some(leader_route) = self.navigation.route(self.units[leader_index].pos, p)
                else {
                    self.effects.push(Effect::Order {
                        position: p,
                        accepted: false,
                    });
                    return;
                };
                let destination = *leader_route.last().expect("routes contain a destination");
                self.effects.push(Effect::Order {
                    position: destination,
                    accepted: true,
                });
                for (n, i) in selected.into_iter().enumerate() {
                    let target = if n == 0 {
                        destination
                    } else {
                        Point::new(
                            destination.x + (n % 2) as f32 * 7.,
                            destination.y + (n / 2 + 1) as f32 * 7.,
                        )
                    };
                    let route = if n == 0 {
                        Some(leader_route.clone())
                    } else {
                        self.navigation.route(self.units[i].pos, target)
                    };
                    let u = &mut self.units[i];
                    u.fire = false;
                    u.slot = n;
                    u.follow = if n == 0 { None } else { Some(leader_id) };
                    u.history = VecDeque::from([u.pos]);
                    u.repath = 0.;
                    if let Some(route) = route {
                        u.target = *route.last().expect("nonempty route");
                        u.path = route.into();
                    }
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
                self.units[i].pose = Some((self.weapon, 0.65));
                self.units[i].aim = p;
                self.launch(start, p, false, self.weapon);
            }
        }
    }
    fn launch(&mut self, start: Point, target: Point, enemy: bool, weapon: Weapon) {
        let range = match weapon {
            Weapon::Rifle => self.tuning.rifle_range,
            Weapon::Grenade => self.tuning.grenade_range,
            Weapon::Rocket => self.tuning.rocket_range,
        };
        let start = start.toward(target, 5.2);
        let end = start.toward(target, range);
        let distance = start.distance(end);
        self.projectiles.push(Projectile {
            pos: start,
            start,
            target: end,
            age: 0.,
            duration: match weapon {
                Weapon::Rifle => distance / self.tuning.rifle_speed,
                Weapon::Grenade => 0.9,
                Weapon::Rocket => distance / self.tuning.rocket_speed,
            }
            .max(0.02),
            enemy,
            weapon,
        });
        self.effects.push(Effect::Shot {
            origin: start,
            direction: end,
            weapon,
        });
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
        let leaders = self
            .units
            .iter()
            .filter(|u| u.alive && !u.enemy)
            .map(|u| (u.id, u.pos, u.target, u.history.clone()))
            .collect::<Vec<_>>();
        let mut ai_seed = self.seed;
        let mut random = || {
            ai_seed = ai_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            (ai_seed >> 32) as u32
        };
        let mut enemy_index = 0;
        for u in &mut self.units {
            u.previous = u.pos;
            if !u.alive {
                continue;
            }
            u.cooldown = (u.cooldown - dt).max(0.);
            u.repath = (u.repath - dt).max(0.);
            if let Some((_, remaining)) = &mut u.pose {
                *remaining -= dt;
                if *remaining <= 0. {
                    u.pose = None;
                }
            }
            if u.enemy {
                if u.aggression == 255 {
                    u.aggression = self.tuning.individual_aggression(enemy_index);
                }
                enemy_index += 1;
                let visible = live
                    .iter()
                    .filter(|(_, p, enemy)| {
                        !*enemy
                            && u.pos.distance(*p) < self.tuning.detection_range
                            && line_of_sight(&self.map, &self.buildings, u.pos, *p)
                    })
                    .min_by(|a, b| u.pos.distance(a.1).total_cmp(&u.pos.distance(b.1)));
                if let Some((id, point, _)) = visible {
                    if u.acquired != Some(*id) {
                        u.acquired = Some(*id);
                        u.reaction = self.tuning.attack_delay(u.aggression, random())
                            * (self.tuning.reaction_seconds / 0.685);
                    }
                    u.reaction = (u.reaction - dt).max(0.);
                    u.last_seen = Some(*point);
                    u.memory = self.tuning.memory_seconds;
                    u.aim = *point;
                    u.ai = if u.reaction > 0. {
                        AiState::Reacting
                    } else {
                        AiState::Engaging
                    };
                    u.fire = u.reaction <= 0.
                        && u.cooldown <= 0.
                        && u.pos.distance(*point) < self.tuning.rifle_range;
                    if u.reaction <= 0. && u.repath <= 0. && !u.fire {
                        let angle = (random() % 256) as f32 * std::f32::consts::TAU / 256.;
                        let destination =
                            Point::new(point.x + angle.cos() * 40., point.y + angle.sin() * 40.);
                        if destination.distance(u.home) < 200. {
                            u.path = self
                                .navigation
                                .route(u.pos, destination)
                                .unwrap_or_default()
                                .into();
                        }
                        u.repath = 0.5 + (random() % 50) as f32 / 50.;
                    }
                } else {
                    u.fire = false;
                    u.acquired = None;
                    u.memory = (u.memory - dt).max(0.);
                    if u.memory > 0. {
                        u.ai = AiState::Searching;
                        if u.repath <= 0. {
                            if let Some(p) = u.last_seen {
                                u.path = self.navigation.route(u.pos, p).unwrap_or_default().into();
                            }
                            u.repath = 0.5;
                        }
                    } else {
                        u.ai = AiState::Idle;
                        if u.repath <= 0. {
                            u.path.clear();
                            // Alternate pauses and local reachable destinations; never drift from home.
                            if random() & 3 != 0 {
                                let angle = (random() % 256) as f32 * std::f32::consts::TAU / 256.;
                                let radius = 12. + (random() % 37) as f32;
                                let p = Point::new(
                                    u.home.x + angle.cos() * radius,
                                    u.home.y + angle.sin() * radius,
                                );
                                if self.map.terrain_at(p.x, p.y)
                                    == self.map.terrain_at(u.home.x, u.home.y)
                                {
                                    u.path =
                                        self.navigation.route(u.pos, p).unwrap_or_default().into();
                                }
                            }
                            u.repath = 0.6 + (random() % 100) as f32 / 50.;
                        }
                    }
                }
            } else if let Some(id) = u.follow {
                if let Some((_, position, destination, trail)) =
                    leaders.iter().find(|(leader, _, _, _)| *leader == id)
                {
                    if u.repath <= 0. {
                        let point = if position.distance(*destination) > 3. {
                            trail
                                .iter()
                                .rev()
                                .copied()
                                .find(|p| p.distance(*position) >= u.slot as f32 * 7.)
                                .unwrap_or(*position)
                        } else {
                            u.target
                        };
                        u.path = self
                            .navigation
                            .route(u.pos, point)
                            .unwrap_or_default()
                            .into();
                        u.repath = 0.3;
                    }
                } else {
                    u.follow = None;
                    u.path = self
                        .navigation
                        .route(u.pos, u.target)
                        .unwrap_or_default()
                        .into();
                }
            }
            if u.fire && u.cooldown <= 0. {
                let weapon = if u.enemy && u.rocket {
                    Weapon::Rocket
                } else {
                    Weapon::Rifle
                };
                shots.push((u.pos, u.aim, u.enemy, weapon));
                u.pose = Some((weapon, 0.12));
                u.cooldown = if u.enemy {
                    if u.rocket {
                        1.7
                    } else {
                        self.tuning.attack_delay(u.aggression, random())
                    }
                } else {
                    self.tuning.player_cooldown
                };
            }
            if !u.fire {
                let speed = if matches!(self.map.terrain_at(u.pos.x, u.pos.y), 5 | 6) {
                    self.tuning.swim_speed
                } else {
                    self.tuning.walk_speed
                };
                let mut distance = dt * speed;
                while distance > 0. {
                    let Some(point) = u.path.front().copied() else {
                        break;
                    };
                    let step = u.pos.distance(point).min(distance);
                    let next = u.pos.toward(point, step);
                    if !self.navigation.segment(u.pos, next) {
                        u.path.clear();
                        break;
                    }
                    u.pos = next;
                    distance -= step;
                    if u.pos.distance(point) < 0.01 {
                        u.path.pop_front();
                    } else {
                        break;
                    }
                }
                if u.history.back().is_none_or(|p| p.distance(u.pos) >= 2.) {
                    u.history.push_back(u.pos);
                    if u.history.len() > 128 {
                        u.history.pop_front();
                    }
                }
            }
        }
        self.seed = ai_seed;
        for (start, mut target, enemy, weapon) in shots {
            if enemy {
                target.x += (self.random() - 0.5) * self.tuning.aim_error;
                target.y += (self.random() - 0.5) * self.tuning.aim_error;
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
                        self.effects.push(Effect::Impact(sample));
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
            if b.health > 0
                && !b.vehicle
                && (self.destroy_objective || self.units.iter().any(|u| u.enemy && u.alive))
            {
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
                    path: VecDeque::new(),
                    history: VecDeque::from([p]),
                    follow: None,
                    slot: 0,
                    ai: AiState::Idle,
                    home: p,
                    aggression: 255,
                    acquired: None,
                    reaction: 0.,
                    last_seen: None,
                    memory: 0.,
                    repath: 0.,
                    pose: None,
                });
            }
        }
        if self.invalid_objectives || !self.units.iter().any(|u| !u.enemy && u.alive) {
            self.outcome = Outcome::Defeat;
        } else if (!self.kill_objective || !self.units.iter().any(|u| u.enemy && u.alive))
            && (!self.destroy_objective
                || !self.buildings.iter().enumerate().any(|(i, b)| {
                    self.objective_buildings
                        .as_ref()
                        .map_or(!b.vehicle, |ids| ids.contains(&i))
                        && b.health > 0
                }))
        {
            self.outcome = Outcome::Victory;
        }
    }
    pub fn explode(&mut self, p: Point) {
        let mut changed = false;
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
                    changed = true;
                    self.effects.push(Effect::Explosion(b.pos));
                }
            }
        }
        if changed {
            self.navigation = Navigation::build(
                &self.map,
                &self
                    .buildings
                    .iter()
                    .filter(|b| b.health > 0)
                    .map(|b| (b.pos, 14.))
                    .collect::<Vec<_>>(),
            );
            for u in &mut self.units {
                if !u.path.is_empty() {
                    u.path = self
                        .navigation
                        .route(u.pos, u.target)
                        .unwrap_or_default()
                        .into();
                }
            }
        }
    }
}

pub fn line_of_sight(map: &ImportedLevel, buildings: &[Building], a: Point, b: Point) -> bool {
    let count = a.distance(b).ceil().max(1.) as usize;
    (1..count).all(|i| {
        let p = a.toward(b, a.distance(b) * i as f32 / count as f32);
        map.walkable(p.x, p.y)
            && buildings
                .iter()
                .all(|o| o.health <= 0 || p.distance(o.pos) > 14.)
    })
}

impl Battle {
    pub fn configure_objectives(
        &mut self,
        kinds: &[crate::campaign::ObjectiveKind],
    ) -> Result<(), String> {
        use crate::campaign::ObjectiveKind::*;
        self.invalid_objectives = true;
        if kinds.is_empty() {
            return Err("A phase requires at least one objective".into());
        }
        for kind in kinds {
            if !matches!(kind, KillAllEnemies | DestroyEnemyBuildings) {
                return Err(format!("Unsupported objective/map combination: {kind:?}"));
            }
        }
        self.invalid_objectives = false;
        self.kill_objective = kinds.contains(&KillAllEnemies);
        self.destroy_objective = kinds.contains(&DestroyEnemyBuildings);
        Ok(())
    }
    pub fn objective_progress(
        &self,
    ) -> Vec<(
        crate::campaign::ObjectiveKind,
        crate::campaign::ObjectiveStatus,
    )> {
        use crate::campaign::{ObjectiveKind::*, ObjectiveStatus::*};
        let mut result = Vec::new();
        if self.kill_objective {
            result.push((
                KillAllEnemies,
                if self.units.iter().any(|u| u.enemy && u.alive) {
                    Pending
                } else {
                    Completed
                },
            ));
        }
        if self.destroy_objective {
            result.push((
                DestroyEnemyBuildings,
                if self.buildings.iter().enumerate().any(|(i, b)| {
                    b.health > 0
                        && self
                            .objective_buildings
                            .as_ref()
                            .map_or(!b.vehicle, |ids| ids.contains(&i))
                }) {
                    Pending
                } else {
                    Completed
                },
            ));
        }
        result
    }
}
