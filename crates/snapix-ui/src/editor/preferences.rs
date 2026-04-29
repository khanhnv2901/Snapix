use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use libadwaita::{ColorScheme, StyleManager};
use serde::{Deserialize, Serialize};
use snapix_core::entitlements::Entitlements;
use snapix_core::license::{Ed25519LicenseVerifier, LicenseVerifier, StubVerifier};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum PreferredSaveFormat {
    Png,
    Jpeg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum AppearancePreference {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppPreferences {
    pub(crate) appearance: AppearancePreference,
    pub(crate) default_save_format: PreferredSaveFormat,
    pub(crate) remember_last_export_format: bool,
    pub(crate) last_export_format: Option<PreferredSaveFormat>,
    pub(crate) jpeg_quality: u8,
    pub(crate) auto_copy_after_export: bool,
    pub(crate) auto_reframe_after_load: bool,
    pub(crate) license_key: Option<String>,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            appearance: AppearancePreference::System,
            default_save_format: PreferredSaveFormat::Png,
            remember_last_export_format: true,
            last_export_format: None,
            jpeg_quality: 92,
            auto_copy_after_export: false,
            auto_reframe_after_load: false,
            license_key: None,
        }
    }
}

impl AppPreferences {
    pub(crate) fn effective_save_format(&self) -> PreferredSaveFormat {
        if self.remember_last_export_format {
            self.last_export_format.unwrap_or(self.default_save_format)
        } else {
            self.default_save_format
        }
    }

    pub(crate) fn effective_jpeg_quality(&self) -> u8 {
        self.jpeg_quality.clamp(60, 100)
    }

    pub(crate) fn entitlements(&self) -> Entitlements {
        self.license_key
            .as_deref()
            .map(str::trim)
            .filter(|key| !key.is_empty())
            .and_then(|key| verify_license_key(key).ok())
            .filter(|entitlements| entitlements.is_pro())
            .unwrap_or_else(Entitlements::free)
    }

    pub(crate) fn activate_license_key(&mut self, key: &str) -> Result<Entitlements> {
        let normalized = key.trim();
        if normalized.is_empty() {
            return Err(anyhow!("Enter a license key."));
        }

        let entitlements = verify_license_key(normalized)?;
        if !entitlements.is_pro() {
            return Err(anyhow!("That key did not unlock Snapix Pro."));
        }

        self.license_key = Some(normalized.to_string());
        Ok(entitlements)
    }

    pub(crate) fn clear_license_key(&mut self) {
        self.license_key = None;
    }
}

fn verify_license_key(key: &str) -> Result<Entitlements> {
    if let Some(public_key_hex) = option_env!("SNAPIX_LICENSE_PUBLIC_KEY_HEX") {
        let verifier = Ed25519LicenseVerifier::from_public_key_hex(public_key_hex)
            .context("Configured Ed25519 public key is invalid")?;
        if let Ok(entitlements) = verifier.verify(key) {
            return Ok(entitlements);
        }
    }

    StubVerifier.verify(key)
}

pub(crate) fn load_preferences() -> Result<AppPreferences> {
    load_preferences_from_path(&preferences_file_path())
}

pub(crate) fn save_preferences(preferences: &AppPreferences) -> Result<()> {
    save_preferences_to_path(&preferences_file_path(), preferences)
}

pub(crate) fn apply_style_preferences(preferences: &AppPreferences) {
    let color_scheme = match preferences.appearance {
        AppearancePreference::System => ColorScheme::Default,
        AppearancePreference::Light => ColorScheme::ForceLight,
        AppearancePreference::Dark => ColorScheme::ForceDark,
    };
    StyleManager::default().set_color_scheme(color_scheme);
}

fn preferences_file_path() -> PathBuf {
    let mut path = glib::user_config_dir();
    path.push("snapix");
    path.push("preferences.json");
    path
}

fn load_preferences_from_path(path: &Path) -> Result<AppPreferences> {
    if !path.exists() {
        return Ok(AppPreferences::default());
    }

    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read preferences file {}", path.display()))?;
    if contents.trim().is_empty() {
        return Ok(AppPreferences::default());
    }

    serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse preferences file {}", path.display()))
}

fn save_preferences_to_path(path: &Path, preferences: &AppPreferences) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create preferences directory {}",
                parent.display()
            )
        })?;
    }

    let json =
        serde_json::to_string_pretty(preferences).context("Failed to serialize preferences")?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write preferences file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        load_preferences_from_path, save_preferences_to_path, AppPreferences, AppearancePreference,
        PreferredSaveFormat,
    };

    fn unique_test_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("snapix-preferences-{nanos}.json"))
    }

    #[test]
    fn preferences_roundtrip_persists_export_settings() {
        let path = unique_test_path();
        let preferences = AppPreferences {
            appearance: AppearancePreference::Dark,
            default_save_format: PreferredSaveFormat::Jpeg,
            remember_last_export_format: true,
            last_export_format: Some(PreferredSaveFormat::Png),
            jpeg_quality: 88,
            auto_copy_after_export: true,
            auto_reframe_after_load: true,
            license_key: Some("SNAPIX-PRO-DEV".into()),
        };

        save_preferences_to_path(&path, &preferences).expect("save preferences");
        let loaded = load_preferences_from_path(&path).expect("load preferences");

        assert_eq!(loaded.appearance, AppearancePreference::Dark);
        assert_eq!(loaded.default_save_format, PreferredSaveFormat::Jpeg);
        assert!(loaded.remember_last_export_format);
        assert_eq!(loaded.last_export_format, Some(PreferredSaveFormat::Png));
        assert_eq!(loaded.jpeg_quality, 88);
        assert!(loaded.auto_copy_after_export);
        assert!(loaded.auto_reframe_after_load);
        assert_eq!(loaded.license_key.as_deref(), Some("SNAPIX-PRO-DEV"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn preferences_legacy_file_uses_defaults_for_missing_fields() {
        let path = unique_test_path();
        fs::write(
            &path,
            r#"{
  "appearance": "Dark",
  "default_save_format": "Png",
  "remember_last_export_format": true,
  "last_export_format": null,
  "auto_copy_after_export": false,
  "auto_reframe_after_load": true,
  "license_key": null
}"#,
        )
        .expect("write legacy preferences");

        let loaded = load_preferences_from_path(&path).expect("load legacy preferences");

        assert_eq!(loaded.appearance, AppearancePreference::Dark);
        assert_eq!(loaded.default_save_format, PreferredSaveFormat::Png);
        assert_eq!(loaded.jpeg_quality, 92);
        assert!(loaded.remember_last_export_format);
        assert!(loaded.auto_reframe_after_load);

        let _ = fs::remove_file(path);
    }
}
