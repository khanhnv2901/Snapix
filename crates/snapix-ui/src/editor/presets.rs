use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use snapix_core::canvas::{
    Background, Document, FrameSettings, ImageAnchor, ImageScaleMode, OutputRatio,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct StylePreset {
    pub(crate) name: String,
    pub(crate) background: Background,
    pub(crate) frame: FrameSettings,
    pub(crate) output_ratio: OutputRatio,
    pub(crate) image_frame_offset_x: f32,
    pub(crate) image_frame_offset_y: f32,
    pub(crate) image_scale_mode: ImageScaleMode,
    pub(crate) image_anchor: ImageAnchor,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PresetFile {
    presets: Vec<StylePreset>,
}

impl StylePreset {
    pub(crate) fn from_document(name: String, document: &Document) -> Self {
        Self {
            name,
            background: document.background.clone(),
            frame: document.frame.clone(),
            output_ratio: document.output_ratio,
            image_frame_offset_x: document.image_frame_offset_x,
            image_frame_offset_y: document.image_frame_offset_y,
            image_scale_mode: document.image_scale_mode,
            image_anchor: document.image_anchor,
        }
    }

    pub(crate) fn apply_to_document(&self, document: &mut Document) {
        document.background = self.background.clone();
        document.frame = self.frame.clone();
        document.output_ratio = self.output_ratio;
        document.image_frame_offset_x = self.image_frame_offset_x;
        document.image_frame_offset_y = self.image_frame_offset_y;
        document.image_scale_mode = self.image_scale_mode;
        document.image_anchor = self.image_anchor;
    }
}

pub(crate) fn load_style_presets() -> Result<Vec<StylePreset>> {
    load_style_presets_from_path(&preset_file_path())
}

pub(crate) fn save_style_preset(preset: StylePreset) -> Result<Vec<StylePreset>> {
    let path = preset_file_path();
    let mut presets = load_style_presets_from_path(&path)?;
    if let Some(existing) = presets
        .iter_mut()
        .find(|existing| existing.name == preset.name)
    {
        *existing = preset;
    } else {
        presets.push(preset);
        presets.sort_by_key(|left| left.name.to_lowercase());
    }
    write_style_presets_to_path(&path, &presets)?;
    Ok(presets)
}

pub(crate) fn delete_style_preset(name: &str) -> Result<Vec<StylePreset>> {
    let path = preset_file_path();
    let mut presets = load_style_presets_from_path(&path)?;
    presets.retain(|preset| preset.name != name);
    write_style_presets_to_path(&path, &presets)?;
    Ok(presets)
}

fn preset_file_path() -> PathBuf {
    let mut path = glib::user_config_dir();
    path.push("snapix");
    path.push("style-presets.json");
    path
}

fn load_style_presets_from_path(path: &Path) -> Result<Vec<StylePreset>> {
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = fs::read_to_string(path)
        .with_context(|| format!("Failed to read preset file {}", path.display()))?;
    if contents.trim().is_empty() {
        return Ok(Vec::new());
    }

    let file: PresetFile = serde_json::from_str(&contents)
        .with_context(|| format!("Failed to parse preset file {}", path.display()))?;
    Ok(file.presets)
}

fn write_style_presets_to_path(path: &Path, presets: &[StylePreset]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create preset directory {}", parent.display()))?;
    }

    let json = serde_json::to_string_pretty(&PresetFile {
        presets: presets.to_vec(),
    })
    .context("Failed to serialize presets")?;
    fs::write(path, json)
        .with_context(|| format!("Failed to write preset file {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use snapix_core::canvas::{Background, Color, Document, Image};

    use super::{load_style_presets_from_path, write_style_presets_to_path, StylePreset};

    fn unique_test_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("snapix-presets-{nanos}.json"))
    }

    #[test]
    fn preset_roundtrip_persists_style_fields() {
        let path = unique_test_path();
        let mut document = Document::new(Image::new(8, 8, vec![255; 8 * 8 * 4]));
        document.frame.padding = 72.0;
        document.background = Background::Solid {
            color: Color {
                r: 12,
                g: 34,
                b: 56,
                a: 255,
            },
        };

        let presets = vec![StylePreset::from_document("Demo".into(), &document)];
        write_style_presets_to_path(&path, &presets).expect("write presets");

        let loaded = load_style_presets_from_path(&path).expect("load presets");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].name, "Demo");
        assert_eq!(loaded[0].frame.padding, 72.0);
        match &loaded[0].background {
            Background::Solid { color } => {
                assert_eq!(color.r, 12);
                assert_eq!(color.g, 34);
                assert_eq!(color.b, 56);
            }
            _ => panic!("expected solid background"),
        }

        let _ = fs::remove_file(path);
    }
}
