//! Desktop Profile System
//!
//! Provides atomic profile management supporting preset profiles (`Gaming`, `Coding`,
//! `Streaming`, `Work`, `Minimal`, `Travel`, `Custom`) containing layout bounds, theme,
//! materials, performance budgets, and visibility rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProfileType {
    Gaming,
    Coding,
    Streaming,
    Work,
    Minimal,
    Travel,
    Custom,
}

impl Default for ProfileType {
    fn default() -> Self {
        ProfileType::Work
    }
}

/// Complete configuration settings encapsulated in a Desktop Profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DesktopProfile {
    pub id: String,
    pub name: String,
    pub profile_type: ProfileType,
    pub theme_id: String,
    pub target_fps: u32,
    pub enable_materials: bool,
    pub active_widgets: Vec<String>,
    pub position_overrides: HashMap<String, (i32, i32)>,
    pub auto_switch_rule: Option<String>,
}

impl Default for DesktopProfile {
    fn default() -> Self {
        Self {
            id: "profile.work.default".to_string(),
            name: "Work Profile".to_string(),
            profile_type: ProfileType::Work,
            theme_id: "theme.default.dark".to_string(),
            target_fps: 30,
            enable_materials: true,
            active_widgets: vec!["perf_monitor_widget".to_string()],
            position_overrides: HashMap::new(),
            auto_switch_rule: None,
        }
    }
}

pub struct ProfileManager {
    profiles: HashMap<String, DesktopProfile>,
    active_profile_id: String,
}

impl ProfileManager {
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        let default_profile = DesktopProfile::default();
        profiles.insert(default_profile.id.clone(), default_profile.clone());

        Self {
            profiles,
            active_profile_id: default_profile.id,
        }
    }

    pub fn register_profile(&mut self, profile: DesktopProfile) {
        self.profiles.insert(profile.id.clone(), profile);
    }

    pub fn active_profile(&self) -> Option<&DesktopProfile> {
        self.profiles.get(&self.active_profile_id)
    }

    /// Atomically switches active profile by ID, returning previous profile ID for state restoration.
    pub fn switch_profile(&mut self, profile_id: &str) -> anyhow::Result<String> {
        if self.profiles.contains_key(profile_id) {
            let previous = self.active_profile_id.clone();
            self.active_profile_id = profile_id.to_string();
            Ok(previous)
        } else {
            anyhow::bail!("Profile '{}' not found", profile_id);
        }
    }
}

impl Default for ProfileManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_manager_switch_and_rollback() {
        let mut manager = ProfileManager::new();

        let gaming = DesktopProfile {
            id: "profile.gaming".to_string(),
            name: "Gaming Mode".to_string(),
            profile_type: ProfileType::Gaming,
            target_fps: 60,
            enable_materials: false,
            active_widgets: vec!["fps_counter".to_string()],
            ..Default::default()
        };

        manager.register_profile(gaming);
        let prev = manager.switch_profile("profile.gaming").unwrap();
        assert_eq!(prev, "profile.work.default");
        assert_eq!(manager.active_profile().unwrap().id, "profile.gaming");
    }
}
