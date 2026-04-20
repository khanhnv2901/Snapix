use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>, // RGBA8
}

impl Image {
    pub fn new(width: u32, height: u32, data: Vec<u8>) -> Self {
        Self { width, height, data }
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
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255, a: 255 };
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0, a: 255 };
    pub const TRANSPARENT: Self = Self { r: 0, g: 0, b: 0, a: 0 };
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
    Solid { color: Color },
    Gradient { from: Color, to: Color, angle_deg: f32 },
    Image { path: String },
    BlurredScreenshot { radius: f32 },
}

impl Default for Background {
    fn default() -> Self {
        Background::Gradient {
            from: Color { r: 100, g: 149, b: 237, a: 255 },
            to: Color { r: 147, g: 112, b: 219, a: 255 },
            angle_deg: 135.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameSettings {
    pub padding: f32,
    pub corner_radius: f32,
    pub shadow: bool,
    pub shadow_blur: f32,
    pub shadow_offset_y: f32,
}

impl Default for FrameSettings {
    fn default() -> Self {
        Self {
            padding: 40.0,
            corner_radius: 12.0,
            shadow: true,
            shadow_blur: 20.0,
            shadow_offset_y: 8.0,
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
