use crate::assets::{FontAsset, SamplerAsset, TextureAsset, TextureAssetData};
use crate::engine::rendering::{ElementPushConstants, ElementRenderObject, Scene};

use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use image::{DynamicImage, Rgba, RgbaImage};

use crate::engine::base::ShaderData;
use ash::vk;
use log::info;
use std::fmt;
use std::rc::Rc;

pub struct TextController {
    max_width: Option<f32>,
    text: String,
    font: FontAsset,
    image: TextureAsset,
    need_update: bool,
    sampler: SamplerAsset,
}

impl TextController {
    pub fn set_text(&mut self, text: String) {
        self.text = text;
        self.need_update = true;
    }
}

impl Scene {
    pub fn instantiate_text(
        &mut self,
        max_width: Option<f32>,
        text: String,
        placeholder: TextureAsset,
        font: FontAsset,
        sampler: SamplerAsset,
    ) -> TextController {
        TextController {
            max_width,
            text,
            font,
            image: placeholder.share(),
            need_update: true,
            sampler,
        }
    }

    pub fn render_rect(&mut self, top_left: [i32; 2]) {}

    pub fn render_text(&mut self, text: &mut TextController, top_left: [i32; 2]) {
        if text.need_update {
            info!("Needs to update TEXT {}", text.text);
            let fonts = FontCollection::from(&text.font);
            let mut paragraph = Paragraph {
                layout: Layout::new(CoordinateSystem::PositiveYDown),
                spans: vec![Span {
                    text: text.text.clone(),
                    font_size: 32.0,
                    font_index: 0,
                }],
                width: 0.0,
                height: 0.0,
            };
            paragraph.layout(text.max_width, fonts.clone());
            let image = DynamicImage::from(paragraph.draw(fonts));
            let image = image.flipv().to_rgba8();
            let (image_width, image_height) = image.dimensions();
            let image_data_len = image.len();
            let image_data = image.as_ptr();

            info!("Creates TEXT {image_width}x{image_height} L{image_data_len}");
            let data = TextureAssetData::read_image_data(
                format!("ui:{}", text.text),
                &self.device,
                self.command_pool,
                self.queue.clone(),
                image_width,
                image_height,
                image_data,
                image_data_len,
            );
            text.image.update(data);
            text.need_update = false;
        }

        let texture = &text.image;
        let image_w = texture.width as f32;
        let image_h = texture.height as f32;
        let [sprite_x, sprite_y] = [0.0, 0.0];
        let [sprite_w, sprite_h] = [image_w, image_h];
        let x = sprite_x / image_w;
        let y = sprite_y / image_h;
        let w = sprite_w / image_w;
        let h = sprite_h / image_h;
        let object = ElementRenderObject {
            constants: ElementPushConstants {
                position: [top_left[0] as f32, top_left[1] as f32],
                size: [image_w, image_h],
                coords: [x, y, w, h],
                pivot: [0.0, 0.0],
                color: [1.0; 4],
            },
            texture: self
                .sprite_pipeline
                .material
                .describe(vec![[ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: text.sampler.handle,
                    image_view: texture.view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                })]])[0],
        };
        self.ui_elements.push(object);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Weight {
    Black,
    ExtraBold,
    Bold,
    SemiBold,
    Medium,
    Normal,
    Light,
    ExtraLight,
    Thin,
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub enum Width {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Slant {
    Italic,
    Oblique,
    Upright,
}

pub struct ParagraphStyle {}

impl ParagraphStyle {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct FontRecord {
    pub font: Font,
    pub family: String,
    pub weight: Weight,
    pub width: Width,
    pub slant: Slant,
}

pub struct FontCollection {
    records: Vec<FontRecord>,
}

pub type FontCollectionRef = Rc<FontCollection>;

pub type FontIndex = usize;

impl FontCollection {
    pub fn new() -> Self {
        Self { records: vec![] }
    }

    pub fn from(asset: &FontAsset) -> FontCollectionRef {
        let collection = FontCollection {
            records: vec![FontRecord {
                font: asset.font_type.clone(),
                family: "".to_string(),
                weight: Weight::Normal,
                width: Width::Normal,
                slant: Slant::Upright,
            }],
        };
        Rc::new(collection)
    }

    pub fn get_fonts(&self) -> Vec<&Font> {
        self.records.iter().map(|record| &record.font).collect()
    }

    pub fn match_font(&self, family: &str, style: &FontStyle) -> FontIndex {
        let mut best = 0;
        let mut best_score = 0;
        for (index, record) in self.records.iter().enumerate() {
            let mut score = 0;
            if record.family == family {
                score += 1000;
            }
            if record.weight == style.weight {
                score += 100;
            }
            if record.width == style.width {
                score += 10;
            }
            if record.slant == style.slant {
                score += 1;
            }

            if score == 1111 {
                best = index;
                break;
            }

            if score > best_score {
                best = index;
                best_score = score;
            }
        }

        best
    }

    pub fn get_font_record(&self, font_id: FontIndex) -> &FontRecord {
        &self.records[font_id]
    }

    pub fn load_with_weight(&mut self, data: &[u8], family: &str, weight: Weight) {
        self.load(data, family, weight, Width::Normal, Slant::Upright);
    }

    pub fn load(&mut self, data: &[u8], family: &str, weight: Weight, width: Width, slant: Slant) {
        let font = Font::from_bytes(
            data,
            FontSettings {
                ..FontSettings::default()
            },
        )
        .unwrap();
        let record = FontRecord {
            font,
            family: family.to_string(),
            weight,
            width,
            slant,
        };
        self.records.push(record);
    }
}

pub struct FontStyle {
    pub weight: Weight,
    pub width: Width,
    pub slant: Slant,
}

impl FontStyle {
    pub fn new(weight: Weight, width: Width, slant: Slant) -> Self {
        Self {
            weight,
            width,
            slant,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub font_size: f32,
    pub font_index: FontIndex,
}

pub struct Paragraph {
    layout: Layout<()>,
    spans: Vec<Span>,
    width: f32,
    height: f32,
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.width)
            .field(&self.height)
            .finish()
    }
}

fn _round_up_power_of_2(value: i32) -> u32 {
    let mut value = value - 1;
    value |= value >> 1;
    value |= value >> 2;
    value |= value >> 4;
    value |= value >> 8;
    value |= value >> 16;
    let v = value + 1;
    v as u32
}

impl Paragraph {
    pub fn text(&self) -> String {
        self.spans
            .iter()
            .map(|span| span.text.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn draw(&self, fonts: FontCollectionRef) -> RgbaImage {
        let mut image = RgbaImage::from_pixel(
            self.width as u32,
            self.height as u32,
            Rgba([255, 255, 255, 0]),
        );
        info!(
            "Rasterize paragraph width: {}, height: {}",
            self.width, self.height
        );
        for glyph in self.layout.glyphs() {
            let (m, data) = fonts
                .get_font_record(glyph.font_index)
                .font
                .rasterize(glyph.parent, glyph.key.px);

            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            for (i, s) in data.iter().enumerate() {
                let y = glyph.y as usize + i / glyph.width;
                let x = glyph.x as usize + i % glyph.width;
                image.put_pixel(x as u32, y as u32, Rgba([255, 255, 255, *s]));
            }
        }
        image
    }

    pub fn layout(&mut self, max_width: Option<f32>, fonts: FontCollectionRef) {
        let max_width = max_width.map(|value| value.floor());
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            max_width,
            ..LayoutSettings::default()
        });
        for span in &self.spans {
            layout.append(
                &fonts.get_fonts(),
                &TextStyle::new(&span.text, span.font_size, span.font_index),
            );
        }
        let mut my_width = 0.0;
        for g in layout.glyphs() {
            let w = g.x + g.width as f32 + 1.0;
            if w > my_width {
                my_width = w;
            }
        }
        self.height = layout.height() + 1.0;
        self.width = my_width;
        self.layout = layout;
    }
}

pub struct ParagraphBuilder {
    fonts: FontCollectionRef,
    spans: Vec<Span>,
}

impl ParagraphBuilder {
    pub fn new(fonts: FontCollectionRef) -> Self {
        Self {
            fonts,
            spans: vec![],
        }
    }

    pub fn append<T: ToString>(
        &mut self,
        text: T,
        font_family: &str,
        font_size: f32,
        font_style: FontStyle,
    ) {
        let span = Span {
            text: text.to_string(),
            font_size,
            font_index: self.fonts.match_font(font_family, &font_style),
        };
        self.spans.push(span);
    }

    pub fn build(self, width: Option<f32>) -> Paragraph {
        let mut paragraph = Paragraph {
            layout: Layout::new(CoordinateSystem::PositiveYDown),
            spans: self.spans,
            width: 0.0,
            height: 0.0,
        };
        paragraph.layout(width, self.fonts);
        paragraph
    }
}
