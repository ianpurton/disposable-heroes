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
    pub version: u32,
    pub mission: usize,
    pub roster: Vec<Soldier>,
    pub memorial: Vec<Soldier>,
}
impl Default for Campaign {
    fn default() -> Self {
        Self {
            version: 1,
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
        if value.version != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported save version",
            ));
        }
        Ok(value)
    }
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p)?;
        }
        let temp = path.with_extension("tmp");
        let mut file = fs::File::create(&temp)?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;
        fs::rename(temp, path)
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
