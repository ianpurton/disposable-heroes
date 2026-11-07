use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::Path,
};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Soldier {
    pub id: u32,
    pub name: String,
    pub alive: bool,
    pub missions: u32,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Campaign {
    #[serde(default)]
    pub progress: CampaignProgress,
    #[serde(default)]
    pub committed: Vec<String>,
    #[serde(default)]
    pub veterans: Vec<u32>,
    #[serde(default)]
    pub checkpoint: String,
    pub version: u32,
    pub mission: usize,
    pub roster: Vec<Soldier>,
    pub memorial: Vec<Soldier>,
}
impl Default for Campaign {
    fn default() -> Self {
        Self {
            version: 2,
            progress: CampaignProgress::default(),
            committed: vec![],
            veterans: vec![],
            checkpoint: "Recruitment".into(),
            mission: 0,
            roster: ["Jools", "Jops", "Stoo", "Rj", "Ubik", "Cj", "Chris", "Pete"]
                .iter()
                .enumerate()
                .map(|(i, n)| Soldier {
                    id: i as u32,
                    name: n.to_string(),
                    alive: true,
                    missions: 0,
                })
                .collect(),
            memorial: vec![],
        }
    }
}
impl Campaign {
    pub fn load(path: &Path) -> io::Result<Self> {
        let value: Self = serde_json::from_slice(&fs::read(path)?)?;
        if value.version != 1 && value.version != 2 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported save version",
            ));
        }
        Ok(value)
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(p) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            fs::create_dir_all(p)?;
        }
        let temp = path.with_extension("tmp");
        let mut file = fs::File::create(&temp)?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;
        fs::rename(temp, path)
    }
    pub fn migrate_progress(&mut self, phase_counts: &[usize]) {
        if self.version != 1 {
            return;
        }
        let mut flat = self.mission;
        self.progress = CampaignProgress {
            mission_index: phase_counts.len(),
            phase_index: 0,
        };
        for (mission_index, count) in phase_counts.iter().enumerate() {
            if flat < *count {
                self.progress = CampaignProgress {
                    mission_index,
                    phase_index: flat,
                };
                break;
            }
            flat -= count;
        }
        self.version = 2;
    }
    pub fn commit_casualties(&mut self, dead: &[u32]) {
        for soldier in &mut self.roster {
            if soldier.alive && dead.contains(&soldier.id) {
                soldier.alive = false;
                self.memorial.push(soldier.clone());
            }
        }
    }
    pub fn deployable(&self) -> usize {
        self.roster.iter().filter(|s| s.alive).count()
    }
    pub fn commit_mission(&mut self) {
        let key = format!("mission:{}", self.progress.mission_index);
        if self.committed.contains(&key) {
            return;
        }
        for s in self
            .roster
            .iter_mut()
            .filter(|s| s.alive && self.veterans.contains(&s.id))
        {
            s.missions += 1;
        }
        self.committed.push(key);
        self.veterans.clear();
        self.mission += 1;
    }
    pub fn complete(&mut self, dead: &[u32]) {
        for s in &mut self.roster {
            if s.alive && dead.contains(&s.id) {
                s.alive = false;
                self.memorial.push(s.clone());
            } else if s.alive {
                s.missions += 1;
            }
        }
        self.mission += 1;
    }
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CampaignDefinition {
    pub name: String,
    pub missions: Vec<MissionDefinition>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MissionDefinition {
    pub name: String,
    pub phases: Vec<PhaseDefinition>,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PhaseDefinition {
    pub name: String,
    pub map_name: String,
    #[serde(default)]
    pub objectives: Vec<String>,
    #[serde(default)]
    pub aggression: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CampaignProgress {
    pub mission_index: usize,
    pub phase_index: usize,
}
impl Soldier {
    pub fn rank(&self) -> &'static str {
        match self.missions {
            0 => "PVT",
            1..=2 => "CPL",
            3..=5 => "SGT",
            _ => "LT",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveKind {
    KillAllEnemies,
    DestroyEnemyBuildings,
    RescueHostages,
    ProtectAllCivilians,
    KidnapEnemyLeader,
    DestroyFactory,
    DestroyComputer,
    GetCivilianHome,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ObjectiveStatus {
    Pending,
    Completed,
    Failed,
}
