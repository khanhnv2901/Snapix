use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8
}

impl Image {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self {
            width,
            height,
            data,
        }
    }

    pub fn from_dynamic(img: image::DynamicImage) -> Self {
        let rgba = img.to_rgba8();
        Self {
            width: rgba.width(),
            height: rgba.height(),
            data: rgba.into_raw(),
        }
    }

    pub fn to_dynamic(&self) -> image::DynamicImage {
        let buf = image::RgbaImage::from_raw(self.width, self.height, self.data.clone())
            .expect("invalid image buffer");
        image::DynamicImage::ImageRgba8(buf)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const WHITE: Self = Self {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };
    pub const BLACK: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    };
    pub const TRANSPARENT: Self = Self {
        r: 0,
        g: 0,
        b: 0,
        a: 0,
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
    pub bold: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Sans".into(),
            font_size: 16.0,
            color: Color::WHITE,
            bold: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Annotation {
    Arrow {
        from: Point,
        to: Point,
        color: Color,
        width: f32,
    },
    Rect {
        bounds: Rect,
        stroke: Stroke,
        fill: Option<Color>,
    },
    Ellipse {
        bounds: Rect,
        stroke: Stroke,
        fill: Option<Color>,
    },
    Text {
        pos: Point,
        content: String,
        style: TextStyle,
    },
    Blur {
        bounds: Rect,
        radius: f32,
    },
    Redact {
        bounds: Rect,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Background {
    Solid {
        color: Color,
    },
    Gradient {
        from: Color,
        to: Color,
        angle_deg: f32,
    },
    Image {
        path: String,
    },
    BlurredScreenshot {
        radius: f32,
    },
}

impl Default for Background {
    fn default() -> Self {
        Background::Gradient {
            from: Color {
                r: 100,
                g: 149,
                b: 237,
                a: 255,
            },
            to: Color {
                r: 147,
                g: 112,
                b: 219,
                a: 255,
            },
            angle_deg: 135.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSettings {
    pub padding: f32,
    pub corner_radius: f32,
    pub shadow: bool,
    pub shadow_offset_x: f32,
    pub shadow_padding: f32,
    pub shadow_blur: f32,
    pub shadow_offset_y: f32,
    pub shadow_strength: f32,
}

impl Default for FrameSettings {
    fn default() -> Self {
        Self {
            padding: 40.0,
            corner_radius: 12.0,
            shadow: true,
            shadow_offset_x: 18.0,
            shadow_padding: 5.0,
            shadow_blur: 28.0,
            shadow_offset_y: 18.0,
            shadow_strength: 0.28,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Document {
    pub base_image: Option<Image>,
    pub background: Background,
    pub frame: FrameSettings,
    pub annotations: Vec<Annotation>,
}

impl Document {
    pub fn new(base_image: Image) -> Self {
        Self {
            base_image: Some(base_image),
            background: Background::default(),
            frame: FrameSettings::default(),
            annotations: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_new_creates_correct_dimensions() {
        let data = vec![0u8; 100 * 50 * 4]; // 100x50 RGBA
        let img = Image::new(100, 50, data.clone());
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 50);
        assert_eq!(img.data.len(), 100 * 50 * 4);
    }

    #[test]
    fn image_roundtrip_dynamic() {
        let data = vec![128u8; 10 * 10 * 4];
        let img = Image::new(10, 10, data);
        let dyn_img = img.to_dynamic();
        let img2 = Image::from_dynamic(dyn_img);
        assert_eq!(img2.width, 10);
        assert_eq!(img2.height, 10);
        assert_eq!(img2.data.len(), 10 * 10 * 4);
    }

    #[test]
    fn color_constants() {
        assert_eq!(Color::WHITE.r, 255);
        assert_eq!(Color::WHITE.g, 255);
        assert_eq!(Color::WHITE.b, 255);
        assert_eq!(Color::WHITE.a, 255);

        assert_eq!(Color::BLACK.r, 0);
        assert_eq!(Color::BLACK.g, 0);
        assert_eq!(Color::BLACK.b, 0);
        assert_eq!(Color::BLACK.a, 255);

        assert_eq!(Color::TRANSPARENT.a, 0);
    }

    #[test]
    fn text_style_default() {
        let style = TextStyle::default();
        assert_eq!(style.font_family, "Sans");
        assert_eq!(style.font_size, 16.0);
        assert!(!style.bold);
    }

    #[test]
    fn frame_settings_default() {
        let frame = FrameSettings::default();
        assert_eq!(frame.padding, 40.0);
        assert_eq!(frame.corner_radius, 12.0);
        assert!(frame.shadow);
        assert_eq!(frame.shadow_offset_x, 18.0);
        assert_eq!(frame.shadow_padding, 5.0);
        assert_eq!(frame.shadow_blur, 28.0);
        assert_eq!(frame.shadow_offset_y, 18.0);
        assert_eq!(frame.shadow_strength, 0.28);
    }

    #[test]
    fn background_default_is_gradient() {
        let bg = Background::default();
        match bg {
            Background::Gradient { angle_deg, .. } => {
                assert_eq!(angle_deg, 135.0);
            }
            _ => panic!("Expected Gradient background"),
        }
    }

    #[test]
    fn document_new_with_image() {
        let data = vec![0u8; 100 * 100 * 4];
        let img = Image::new(100, 100, data);
        let doc = Document::new(img);
        assert!(doc.base_image.is_some());
        assert!(doc.annotations.is_empty());
    }

    #[test]
    fn document_default_has_no_image() {
        let doc = Document::default();
        assert!(doc.base_image.is_none());
    }

    #[test]
    fn annotation_serialization_roundtrip() {
        let arrow = Annotation::Arrow {
            from: Point { x: 0.0, y: 0.0 },
            to: Point { x: 100.0, y: 100.0 },
            color: Color::WHITE,
            width: 2.0,
        };
        let json = serde_json::to_string(&arrow).unwrap();
        let parsed: Annotation = serde_json::from_str(&json).unwrap();
        match parsed {
            Annotation::Arrow {
                from, to, width, ..
            } => {
                assert_eq!(from.x, 0.0);
                assert_eq!(to.x, 100.0);
                assert_eq!(width, 2.0);
            }
            _ => panic!("Expected Arrow annotation"),
        }
    }
}
