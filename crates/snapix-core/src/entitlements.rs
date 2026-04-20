use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tier {
    Free,
    Pro,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    UnlimitedExports,
    AiRedact,
    CloudUpload,
    ScreenRecording,
    CustomTemplates,
    ScrollingCapture,
    WindowMockup,
    OcrCopyText,
}

#[derive(Debug, Clone)]
pub struct Entitlements {
    pub tier: Tier,
    features: HashSet<Feature>,
}

impl Entitlements {
    pub fn free() -> Self {
        Self {
            tier: Tier::Free,
            features: HashSet::new(),
        }
    }

    pub fn pro() -> Self {
        let all = [
            Feature::UnlimitedExports,
            Feature::AiRedact,
            Feature::CloudUpload,
            Feature::ScreenRecording,
            Feature::CustomTemplates,
            Feature::ScrollingCapture,
            Feature::WindowMockup,
            Feature::OcrCopyText,
        ];
        Self {
            tier: Tier::Pro,
            features: all.into_iter().collect(),
        }
    }

    pub fn has(&self, feature: &Feature) -> bool {
        self.features.contains(feature)
    }

    pub fn is_pro(&self) -> bool {
        self.tier == Tier::Pro
    }
}

impl Default for Entitlements {
    fn default() -> Self {
        Self::free()
    }
}
