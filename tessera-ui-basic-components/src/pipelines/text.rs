//! Text Rendering Pipeline for UI Components
//!
//! This module implements the GPU pipeline and related utilities for efficient text rendering in Tessera UI components.
//! It leverages the Glyphon engine for font management, shaping, and rasterization, providing high-quality and performant text output.
//! Typical use cases include rendering static labels, paragraphs, and editable text fields within the UI.
//!
//! The pipeline is designed to be reusable and efficient, sharing a static font system across the application to minimize resource usage.
//! It exposes APIs for preparing, measuring, and rendering text, supporting advanced features such as font fallback, shaping, and multi-line layout.
//!
//! This module is intended for integration into custom UI components and rendering flows that require flexible and robust text display.

mod command;

use std::sync::OnceLock;

use glyphon::fontdb;
use parking_lot::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use tessera_ui::{Color, DrawablePipeline, PxPosition, PxSize, wgpu};

pub use command::{TextCommand, TextConstraint};

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
static FONT_SYSTEM: OnceLock<RwLock<glyphon::FontSystem>> = OnceLock::new();

#[cfg(target_os = "android")]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    let mut font_system = glyphon::FontSystem::new();

    font_system.db_mut().load_fonts_dir("/system/fonts");
    font_system.db_mut().set_sans_serif_family("Roboto");
    font_system.db_mut().set_serif_family("Noto Serif");
    font_system.db_mut().set_monospace_family("Droid Sans Mono");
    font_system.db_mut().set_cursive_family("Dancing Script");
    font_system.db_mut().set_fantasy_family("Dancing Script");

    RwLock::new(font_system)
}

#[cfg(not(target_os = "android"))]
fn init_font_system() -> RwLock<glyphon::FontSystem> {
    RwLock::new(glyphon::FontSystem::new())
}

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
/// This function returns a read lock of the font system.
pub fn read_font_system() -> RwLockReadGuard<'static, glyphon::FontSystem> {
    FONT_SYSTEM.get_or_init(init_font_system).read()
}

/// It costs a lot to create a glyphon font system, so we use a static one
/// to share it every where and avoid creating it multiple times.
/// This function returns a write lock of the font system.
pub fn write_font_system() -> RwLockWriteGuard<'static, glyphon::FontSystem> {
    FONT_SYSTEM.get_or_init(init_font_system).write()
}

/// A text renderer
/// Pipeline for rendering text using the Glyphon engine.
///
/// This struct manages font atlas, cache, viewport, and swash cache for efficient text rendering.
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::text::GlyphonTextRender;
///
/// let pipeline = GlyphonTextRender::new(&device, &queue, &config, sample_count);
/// ```
pub struct GlyphonTextRender {
    /// Glyphon font atlas, a heavy-weight, shared resource.
    atlas: glyphon::TextAtlas,
    /// Glyphon cache, a heavy-weight, shared resource.
    #[allow(unused)]
    cache: glyphon::Cache,
    /// Glyphon viewport, holds screen-size related buffers.
    viewport: glyphon::Viewport,
    /// Glyphon swash cache, a CPU-side cache for glyph rasterization.
    swash_cache: glyphon::SwashCache,
    /// The multisample state, needed for creating temporary renderers.
    msaa: wgpu::MultisampleState,
}

impl GlyphonTextRender {
    /// Creates a new text renderer pipeline.
    ///
    /// # Parameters
    /// - `gpu`: The wgpu device.
    /// - `queue`: The wgpu queue.
    /// - `config`: Surface configuration.
    /// - `sample_count`: Multisample count for anti-aliasing.
    pub fn new(
        gpu: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        sample_count: u32,
    ) -> Self {
        let cache = glyphon::Cache::new(gpu);
        let atlas = glyphon::TextAtlas::new(gpu, queue, &cache, config.format);
        let viewport = glyphon::Viewport::new(gpu, &cache);
        let swash_cache = glyphon::SwashCache::new();
        let msaa = wgpu::MultisampleState {
            count: sample_count,
            mask: !0,
            alpha_to_coverage_enabled: false,
        };

        Self {
            atlas,
            cache,
            viewport,
            swash_cache,
            msaa,
        }
    }
}

#[allow(unused_variables)]
impl DrawablePipeline<TextCommand> for GlyphonTextRender {
    /// Draws text in a UI component using the Glyphon engine.
    ///
    /// # Parameters
    /// - `gpu`: The wgpu device.
    /// - `gpu_queue`: The wgpu queue.
    /// - `config`: Surface configuration.
    /// - `render_pass`: The render pass to encode drawing commands.
    /// - `command`: The text command with text data.
    /// - `size`: The size of the component in pixels.
    /// - `start_pos`: The top-left position of the component.
    /// - `_scene_texture_view`: Not used for text rendering.
    fn draw(
        &mut self,
        gpu: &wgpu::Device,
        gpu_queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        render_pass: &mut wgpu::RenderPass<'_>,
        command: &TextCommand,
        size: PxSize,
        start_pos: PxPosition,
        _scene_texture_view: &wgpu::TextureView,
    ) {
        // Create a new, temporary TextRenderer for each draw call.
        // This is necessary to avoid state conflicts when rendering multiple
        // text elements interleaved with other components. It correctly
        // isolates the `prepare` call for each text block.
        let mut text_renderer = glyphon::TextRenderer::new(&mut self.atlas, gpu, self.msaa, None);

        self.viewport.update(
            gpu_queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );

        let text_areas = std::iter::once(command.data.text_area(start_pos));

        text_renderer
            .prepare(
                gpu,
                gpu_queue,
                &mut write_font_system(),
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .unwrap();

        text_renderer
            .render(&self.atlas, &self.viewport, render_pass)
            .unwrap();
    }
}

/// Text data for rendering, including buffer and size.
///
/// # Fields
/// - `text_buffer`: The glyphon text buffer.
/// - `size`: The size of the text area [width, height].
///
/// # Example
///
/// ```rust,ignore
/// use tessera_ui_basic_components::pipelines::text::TextData;
///
/// let data = TextData::new("Hello".to_string(), color, 16.0, 1.2, constraint);
/// ```
#[derive(Debug, Clone)]
pub struct TextData {
    /// glyphon text buffer
    text_buffer: glyphon::Buffer,
    /// text area size
    pub size: [u32; 2],
}

impl TextData {
    /// Prepares text data for rendering.
    ///
    /// # Parameters
    /// - `text`: The text string.
    /// - `color`: The text color.
    /// - `size`: Font size.
    /// - `line_height`: Line height.
    /// - `constraint`: Text constraint for layout.
    pub fn new(
        text: String,
        color: Color,
        size: f32,
        line_height: f32,
        constraint: TextConstraint,
    ) -> TextData {
        // Create text buffer
        let mut text_buffer = glyphon::Buffer::new(
            &mut write_font_system(),
            glyphon::Metrics::new(size, line_height),
        );
        let color = glyphon::Color::rgba(
            (color.r * 255.0) as u8,
            (color.g * 255.0) as u8,
            (color.b * 255.0) as u8,
            (color.a * 255.0) as u8,
        );
        text_buffer.set_wrap(&mut write_font_system(), glyphon::Wrap::Glyph);
        text_buffer.set_size(
            &mut write_font_system(),
            constraint.max_width,
            constraint.max_height,
        );
        text_buffer.set_text(
            &mut write_font_system(),
            &text,
            &glyphon::Attrs::new()
                .family(fontdb::Family::SansSerif)
                .color(color),
            glyphon::Shaping::Advanced,
        );
        text_buffer.shape_until_scroll(&mut write_font_system(), false);
        // Calculate text bounds
        // Get the layout runs
        let mut run_width: f32 = 0.0;
        // Calculate the line height based on the number of lines
        let line_height =
            text_buffer.layout_runs().count() as f32 * text_buffer.metrics().line_height;
        for run in text_buffer.layout_runs() {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
        }
        // build text data
        Self {
            text_buffer,
            size: [run_width as u32, line_height as u32],
        }
    }

    pub fn from_buffer(text_buffer: glyphon::Buffer) -> Self {
        // Calculate text bounds
        // Get the layout runs
        let mut run_width: f32 = 0.0;
        // Calculate the line height based on the number of lines
        let line_height =
            text_buffer.layout_runs().count() as f32 * text_buffer.metrics().line_height;
        for run in text_buffer.layout_runs() {
            // Take the max. width of all lines.
            run_width = run_width.max(run.line_w);
        }
        // build text data
        Self {
            text_buffer,
            size: [run_width as u32, line_height as u32],
        }
    }

    /// Get the glyphon text area from the text data
    fn text_area(&'_ self, start_pos: PxPosition) -> glyphon::TextArea<'_> {
        let bounds = glyphon::TextBounds {
            left: start_pos.x.raw(),
            top: start_pos.y.raw(),
            right: start_pos.x.raw() + self.size[0] as i32,
            bottom: start_pos.y.raw() + self.size[1] as i32,
        };
        glyphon::TextArea {
            buffer: &self.text_buffer,
            left: start_pos.x.to_f32(),
            top: start_pos.y.to_f32(),
            scale: 1.0,
            bounds,
            default_color: glyphon::Color::rgb(0, 0, 0), // Black by default
            custom_glyphs: &[],
        }
    }
}
