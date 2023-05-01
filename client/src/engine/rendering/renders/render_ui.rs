use std::rc::Rc;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, RwLock};
use std::{fmt, thread};

use ash::vk;
use fontdue::layout::{CoordinateSystem, Layout, LayoutSettings, TextStyle};
use fontdue::{Font, FontSettings};
use log::{error, info};
use sdl2::keyboard::Keycode;

use crate::assets::{FontAsset, SamplerAsset, TextureAsset, TextureAssetData};
use crate::engine::base::{MyQueue, ShaderData};
use crate::engine::rendering::{ElementPushConstants, ElementRenderObject, Scene, SceneMetrics};
use crate::engine::Input;
use crate::monitoring::Timer;

pub struct InputController {
    width: u32,
    height: u32,
    top: u32,
    left: u32,
    pub text: TextController,
    hover: bool,
    pub focused: bool,
    bg: TextureAsset,
    color_bg: [f32; 4],
    color_fg: [f32; 4],
    color_focused: [f32; 4],
    color_hover: [f32; 4],
    max_length: usize,
}

impl InputController {
    pub fn get_value(&self) -> String {
        self.text.get_text().to_string()
    }

    pub fn update(&mut self, input: &Input) {
        let mouse = input.mouse_position_raw();
        let mx = mouse[0] as u32;
        let my = mouse[1] as u32;
        if mx >= self.left
            && mx <= self.left + self.width
            && my >= self.top
            && my <= self.top + self.height
        {
            self.hover = true;
        } else {
            self.hover = false;
        }

        if input.left_click() {
            if self.hover {
                self.focused = true;
            } else {
                self.focused = false;
            }
        }

        if self.focused && input.pressed(Keycode::Escape) || input.pressed(Keycode::Return) {
            self.focused = false;
        }

        if self.focused {
            if input.pressed(Keycode::Backspace) {
                let mut current = self.text.get_text().trim().to_string();
                if current.len() > 0 {
                    current.pop();
                    self.text.set_text(current);
                }
            }

            if let Some(text) = input.text.clone() {
                let current = self.text.get_text().to_string();
                let updated = format!("{current}{text}");
                let text = if updated.len() > self.max_length {
                    updated.chars().take(self.max_length).collect()
                } else {
                    updated
                };
                self.text.set_text(text);
            }
        }
    }
}

pub struct ButtonController {
    width: u32,
    height: u32,
    top: u32,
    left: u32,
    text: TextController,
    hover: bool,
    pub clicked: bool,
    bg: TextureAsset,
    color_bg: [f32; 4],
    color_fg: [f32; 4],
    color_active: [f32; 4],
    color_hover: [f32; 4],
}

impl ButtonController {
    pub fn udpate(&mut self, mouse: [f32; 2], clicked: bool) {
        let mx = mouse[0] as u32;
        let my = mouse[1] as u32;
        if mx >= self.left
            && mx <= self.left + self.width
            && my >= self.top
            && my <= self.top + self.height
        {
            self.hover = true;
        } else {
            self.hover = false;
        }
        self.clicked = false;
        if self.hover {
            self.clicked = clicked;
        }
    }
}

pub struct TextController {
    max_width: u32,
    max_height: u32,
    text: String,
    font_size: f32,
    font: FontAsset,
    image: TextureAsset,
    sampler: SamplerAsset,
    rasterizer: Arc<RwLock<TextRenderThread>>,
}

impl TextController {
    pub fn get_text(&self) -> &String {
        &self.text
    }

    pub fn set_text(&mut self, text: String) {
        if self.text != text {
            self.text = text;
            self.rasterizer.write().unwrap().request(
                self.image.share(),
                self.text.clone(),
                self.font_size,
            );
        }
    }
}

pub struct TextRenderRequest {
    text: String,
    font_size: f32,
    max_width: u32,
    max_height: u32,
    canvas: vk::Image,
}

pub struct TextRenderThread {
    font: FontAsset,
    request: Sender<TextRenderRequest>,
}

impl TextRenderThread {
    pub fn spawn(
        font: FontAsset,
        queue: Arc<MyQueue>,
        pool: vk::CommandPool,
        metrics: Arc<Box<SceneMetrics>>,
    ) -> Self {
        let (request, requests) = channel::<TextRenderRequest>();
        let font_type = font.font_type.clone();
        thread::spawn(move || {
            for mut request in requests {
                let mut timer = Timer::now();
                let fonts = FontCollection::from(font_type.clone());

                let mut paragraph = Paragraph {
                    layout: Layout::new(CoordinateSystem::PositiveYDown),
                    spans: vec![Span {
                        text: request.text.clone(),
                        font_size: request.font_size,
                        font_index: 0,
                    }],
                    width: 0,
                    height: 0,
                };
                paragraph.layout(request.max_width, request.max_height, fonts.clone());

                let image_width = request.max_width as usize;
                let image_height = request.max_height as usize;
                let image_data = paragraph.draw_to_vec(fonts, image_width, image_height);
                let image_data_len = image_data.len();
                let image_data = image_data.as_ptr();

                // let image = DynamicImage::from(paragraph.draw(fonts));
                // let image = image.flipv().to_rgba8();
                // let (image_width, image_height) = image.dimensions();
                // let image_data_len = image.len();
                // let image_data = image.as_ptr();

                TextureAssetData::write_image_data(
                    request.canvas,
                    queue.clone(),
                    pool,
                    image_width as u32,
                    image_height as u32,
                    image_data,
                    image_data_len,
                );
                timer.gauge(&request.text.len().to_string(), &metrics.text);
            }
            info!("Terminates text rasterizer thread");
        });
        TextRenderThread { font, request }
    }

    pub fn request(&mut self, canvas: TextureAsset, text: String, font_size: f32) {
        let request = TextRenderRequest {
            text,
            font_size,
            max_width: canvas.width,
            max_height: canvas.height,
            canvas: canvas.image,
        };
        if self.request.send(request).is_err() {
            error!("Unable to rasterize text, rasterizer thread terminated")
        }
    }
}

impl Scene {
    pub fn instantiate_text(
        &mut self,
        max_width: u32,
        max_height: u32,
        text: String,
        font: FontAsset,
        sampler: SamplerAsset,
    ) -> TextController {
        let image_data_len = (max_height * max_height * 4) as usize;
        let image_data = vec![0; image_data_len];
        let data = TextureAssetData::read_image_data(
            format!("ui:text:{max_width}x{max_height}"),
            &self.device,
            self.command_pool,
            self.queue.clone(),
            max_width,
            max_height,
            image_data.as_ptr(),
            image_data_len,
        );
        let image = TextureAsset::from(data);
        TextController {
            max_width,
            max_height,
            text,
            font_size: 32.0,
            font,
            image,
            sampler,
            rasterizer: self.rasterizer.clone(),
        }
    }

    pub fn instantiate_button(
        &mut self,
        top: u32,
        left: u32,
        max_width: u32,
        max_height: u32,
        label: String,
        font: FontAsset,
        sampler: SamplerAsset,
        bg: TextureAsset,
    ) -> ButtonController {
        let image_data_len = (max_height * max_height * 4) as usize;
        let image_data = vec![0; image_data_len];
        let data = TextureAssetData::read_image_data(
            format!("ui:text:{max_width}x{max_height}"),
            &self.device,
            self.command_pool,
            self.queue.clone(),
            max_width,
            max_height,
            image_data.as_ptr(),
            image_data_len,
        );
        let image = TextureAsset::from(data);
        let mut text = TextController {
            max_width,
            max_height,
            text: "".to_string(),
            font_size: 48.0,
            font,
            image,
            sampler,
            rasterizer: self.rasterizer.clone(),
        };
        text.set_text(label);
        ButtonController {
            width: max_width,
            height: max_height,
            top,
            left,
            text,
            hover: false,
            clicked: false,
            bg,
            color_bg: [0.0, 0.0, 0.0, 0.25],
            color_fg: [1.0, 1.0, 1.0, 1.0],
            color_active: [0.0, 0.0, 0.0, 0.25],
            color_hover: [0.0, 0.0, 0.0, 0.5],
        }
    }

    pub fn instantiate_input(
        &mut self,
        top: u32,
        left: u32,
        max_width: u32,
        max_height: u32,
        label: String,
        font: FontAsset,
        sampler: SamplerAsset,
        bg: TextureAsset,
        max_length: usize,
    ) -> InputController {
        let image_data_len = (max_height * max_height * 4) as usize;
        let image_data = vec![0; image_data_len];
        let data = TextureAssetData::read_image_data(
            format!("ui:text:{max_width}x{max_height}"),
            &self.device,
            self.command_pool,
            self.queue.clone(),
            max_width,
            max_height,
            image_data.as_ptr(),
            image_data_len,
        );
        let image = TextureAsset::from(data);
        let mut text = TextController {
            max_width,
            max_height,
            text: "".to_string(),
            font_size: 48.0,
            font,
            image,
            sampler,
            rasterizer: self.rasterizer.clone(),
        };
        text.set_text(label);
        InputController {
            width: max_width,
            height: max_height,
            top,
            left,
            text,
            hover: false,
            focused: false,
            bg,
            color_bg: [0.0, 0.0, 0.0, 0.25],
            color_fg: [1.0, 1.0, 1.0, 1.0],
            color_focused: [0.0, 0.0, 0.25, 0.25],
            color_hover: [0.0, 0.0, 0.0, 0.5],
            max_length,
        }
    }

    pub fn render_texture(
        &mut self,
        texture: TextureAsset,
        top_left: [i32; 2],
        size: [f32; 2],
        color: [f32; 4],
    ) {
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
                position: [top_left[1] as f32, top_left[0] as f32],
                size,
                coords: [x, y, w, h],
                pivot: [0.0, 0.0],
                color,
            },
            texture: self
                .sprite_pipeline
                .material
                .describe(vec![[ShaderData::Texture(vk::DescriptorImageInfo {
                    sampler: texture.sampler,
                    image_view: texture.view,
                    image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                })]])[0],
        };
        self.ui_elements.push(object);
    }

    pub fn render_text(&mut self, text: &mut TextController, top_left: [i32; 2]) {
        self.render_texture(
            text.image.share(),
            top_left,
            [text.max_width as f32, text.max_height as f32],
            [1.0; 4],
        )
    }

    pub fn render_button(&mut self, button: &ButtonController) {
        let bg_color = if button.clicked {
            button.color_active
        } else if button.hover {
            button.color_hover
        } else {
            button.color_bg
        };
        self.render_texture(
            button.bg.share(),
            [button.top as i32, button.left as i32],
            [button.width as f32, button.height as f32],
            bg_color,
        );
        self.render_texture(
            button.text.image.share(),
            [button.top as i32, button.left as i32],
            [button.width as f32, button.height as f32],
            button.color_fg,
        )
    }

    pub fn render_input(&mut self, input: &InputController) {
        let bg_color = if input.focused {
            input.color_focused
        } else if input.hover {
            input.color_hover
        } else {
            input.color_bg
        };
        self.render_texture(
            input.bg.share(),
            [input.top as i32, input.left as i32],
            [input.width as f32, input.height as f32],
            bg_color,
        );
        self.render_texture(
            input.text.image.share(),
            [input.top as i32, input.left as i32],
            [input.width as f32, input.height as f32],
            input.color_fg,
        )
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

    pub fn from(font: Font) -> FontCollectionRef {
        let collection = FontCollection {
            records: vec![FontRecord {
                font,
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
    pub width: u32,
    pub height: u32,
}

impl fmt::Debug for Paragraph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.width)
            .field(&self.height)
            .finish()
    }
}

fn round_up_pow_2(value: i32) -> u32 {
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

    pub fn draw_to_vec(&self, fonts: FontCollectionRef, width: usize, height: usize) -> Vec<u8> {
        let mut buf = vec![0; width * height * 4];
        for glyph in self.layout.glyphs() {
            let (_m, data) = fonts
                .get_font_record(glyph.font_index)
                .font
                .rasterize(glyph.parent, glyph.key.px);

            if glyph.width == 0 || glyph.height == 0 {
                continue;
            }

            for (i, s) in data.iter().enumerate() {
                let y = glyph.y as usize + i / glyph.width;
                let x = glyph.x as usize + i % glyph.width;
                if y > height {
                    // height overflow
                    continue;
                }
                let y = height - y; // flip v
                let offset = (y * width * 4) + x * 4;
                buf[offset + 0] = 255;
                buf[offset + 1] = 255;
                buf[offset + 2] = 255;
                buf[offset + 3] = *s;
            }
        }
        buf
    }

    pub fn layout(&mut self, max_width: u32, max_height: u32, fonts: FontCollectionRef) {
        let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
        layout.reset(&LayoutSettings {
            max_width: Some(max_width as f32),
            max_height: Some(max_height as f32),
            ..LayoutSettings::default()
        });
        for span in &self.spans {
            layout.append(
                &fonts.get_fonts(),
                &TextStyle::new(&span.text, span.font_size, span.font_index),
            );
        }
        let mut text_width = 0.0;
        for g in layout.glyphs() {
            let w = g.x + g.width as f32 + 1.0;
            if w > text_width {
                text_width = w;
            }
        }
        let h = layout.height() + 1.0;
        let w = text_width;
        self.height = round_up_pow_2(h as i32);
        self.width = round_up_pow_2(w as i32);
        self.layout = layout;
    }
}

// pub struct ParagraphBuilder {
//     fonts: FontCollectionRef,
//     spans: Vec<Span>,
// }
//
// impl ParagraphBuilder {
//     pub fn new(fonts: FontCollectionRef) -> Self {
//         Self {
//             fonts,
//             spans: vec![],
//         }
//     }
//
//     pub fn append<T: ToString>(
//         &mut self,
//         text: T,
//         font_family: &str,
//         font_size: f32,
//         font_style: FontStyle,
//     ) {
//         let span = Span {
//             text: text.to_string(),
//             font_size,
//             font_index: self.fonts.match_font(font_family, &font_style),
//         };
//         self.spans.push(span);
//     }
//
//     pub fn build(self, width: Option<f32>) -> Paragraph {
//         let mut paragraph = Paragraph {
//             layout: Layout::new(CoordinateSystem::PositiveYDown),
//             spans: self.spans,
//             width: 0,
//             height: 0,
//         };
//         paragraph.layout(width, self.fonts);
//         paragraph
//     }
// }
