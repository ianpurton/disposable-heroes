//! Source-pixel units. These are provisional gameplay values, not a claim of cycle-exact Amiga behavior.
#[derive(Clone, Debug)]
pub struct CombatTuning {
    pub walk_speed: f32,
    pub swim_speed: f32,
    pub detection_range: f32,
    pub rifle_range: f32,
    pub grenade_range: f32,
    pub rocket_range: f32,
    pub rifle_speed: f32,
    pub rocket_speed: f32,
    pub player_cooldown: f32,
    pub enemy_cooldown: f32,
    pub reaction_seconds: f32,
    pub aim_error: f32,
    pub memory_seconds: f32,
}
impl Default for CombatTuning {
    fn default() -> Self {
        Self {
            walk_speed: 48.,
            swim_speed: 20.,
            detection_range: 220.,
            rifle_range: 180.,
            grenade_range: 110.,
            rocket_range: 260.,
            rifle_speed: 340.,
            rocket_speed: 220.,
            player_cooldown: 0.16,
            enemy_cooldown: 0.9,
            reaction_seconds: 0.55,
            aim_error: 24.,
            memory_seconds: 1.2,
        }
    }
}
impl CombatTuning {
    pub fn aggression(&mut self, range: &[u8]) {
        let level = if range.is_empty() {
            4.
        } else {
            range.iter().map(|v| *v as f32).sum::<f32>() / range.len() as f32
        }
        .clamp(0., 15.);
        self.reaction_seconds = (0.85 - level * 0.055).max(0.15);
        self.enemy_cooldown = (1.1 - level * 0.055).max(0.28);
        self.aim_error = (34. - level * 1.5).max(8.);
    }
}
