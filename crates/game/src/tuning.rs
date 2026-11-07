//! Source-pixel units. These are provisional gameplay values, not a claim of cycle-exact Amiga behavior.
#[derive(Clone, Debug)]
pub struct CombatTuning {
    pub aggression_range: (u8, u8),
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
            aggression_range: (2, 4),
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
        self.aggression_range = (
            range.iter().copied().min().unwrap_or(4),
            range.iter().copied().max().unwrap_or(4),
        );
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

impl CombatTuning {
    /// Sprite_Reset_Move_Timer: (random & (31 >> (aggression >> 2))) + 50 - 4a.
    pub fn attack_delay(&self, aggression: u8, random: u32) -> f32 {
        let a = aggression.min(15) as u32;
        ((random & (31 >> (a >> 2))) + (50u32.saturating_sub(4 * a))).max(5) as f32 / 50.
    }
    pub fn individual_aggression(&self, index: usize) -> u8 {
        let (lo, hi) = self.aggression_range;
        let span = (hi - lo) as usize;
        if span == 0 {
            return lo;
        }
        let phase = index % (2 * span);
        lo + phase.min(2 * span - phase) as u8
    }
}
