//! Provides a highly customizable and interactive button component for Tessera UI.
//!
//! This module defines the [`button`] component and its configuration via [`ButtonArgs`].
//! The button supports custom colors, shapes, padding, border, ripple effects, and hover states.
//! It is designed to wrap arbitrary child content and handle user interactions such as clicks
//! with visual feedback. Typical use cases include triggering actions, submitting forms, or
//! serving as a core interactive element in user interfaces.
//!
//! The API offers builder patterns and convenience constructors for common button styles
//! (primary, secondary, success, danger), making it easy to create consistent and accessible
//! buttons throughout your application.
//!
//! Example usage and customization patterns are provided in the [`button`] documentation.
//!
//! # Features
//! - Customizable appearance: color, shape, border, padding, ripple, hover
//! - Flexible sizing: explicit width/height or content-based
//! - Event handling: on_click callback
//! - Composable: can wrap any child component
//! - Builder and fluent APIs for ergonomic usage
//!
//! See [`button`] and [`ButtonArgs`] for details.
use std::sync::Arc;

use derive_builder::Builder;
use tessera_ui::{Color, DimensionValue, Dp};
use tessera_ui_macros::tessera;

use crate::{
    ripple_state::RippleState,
    shape_def::Shape,
    surface::{SurfaceArgsBuilder, surface},
};

/// Arguments for the `button` component.
#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct ButtonArgs {
    /// The fill color of the button (RGBA).
    #[builder(default = "Color::new(0.2, 0.5, 0.8, 1.0)")]
    pub color: Color,
    /// The hover color of the button (RGBA). If None, no hover effect is applied.
    #[builder(default)]
    pub hover_color: Option<Color>,
    /// The shape of the button.
    #[builder(default = "Shape::RoundedRectangle { corner_radius: 25.0, g2_k_value: 3.0 }")]
    pub shape: Shape,
    /// The padding of the button.
    #[builder(default = "Dp(12.0)")]
    pub padding: Dp,
    /// Optional explicit width behavior for the button.
    #[builder(default, setter(strip_option))]
    pub width: Option<DimensionValue>,
    /// Optional explicit height behavior for the button.
    #[builder(default, setter(strip_option))]
    pub height: Option<DimensionValue>,
    /// The click callback function
    pub on_click: Arc<dyn Fn() + Send + Sync>,
    /// The ripple color (RGB) for the button.
    #[builder(default = "Color::from_rgb(1.0, 1.0, 1.0)")]
    pub ripple_color: Color,
    /// Width of the border. If > 0, an outline will be drawn.
    #[builder(default = "0.0")]
    pub border_width: f32,
    /// Optional color for the border (RGBA). If None and border_width > 0, `color` will be used.
    #[builder(default)]
    pub border_color: Option<Color>,
}

impl std::fmt::Debug for ButtonArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ButtonArgs")
            .field("color", &self.color)
            .field("hover_color", &self.hover_color)
            .field("shape", &self.shape)
            .field("padding", &self.padding)
            .field("width", &self.width)
            .field("height", &self.height)
            .field("on_click", &"<callback>")
            .field("ripple_color", &self.ripple_color)
            .field("border_width", &self.border_width)
            .field("border_color", &self.border_color)
            .finish()
    }
}

impl Default for ButtonArgs {
    fn default() -> Self {
        ButtonArgsBuilder::default()
            .on_click(Arc::new(|| {}))
            .build()
            .unwrap()
    }
}

/// Creates an interactive button component that can wrap any custom child content.
///
/// The `button` component provides a clickable surface with a ripple effect,
/// customizable appearance, and event handling. It's built on top of the `surface`
/// component and handles user interactions like clicks and hover states.
///
/// # Parameters
///
/// - `args`: An instance of `ButtonArgs` or `ButtonArgsBuilder` that defines the button's
///   properties, such as color, shape, padding, and the `on_click` callback.
/// - `ripple_state`: An `Arc<RippleState>` that manages the visual state of the ripple
///   effect. This should be created and managed by the parent component to persist
///   the ripple animation state across recompositions.
/// - `child`: A closure that defines the content to be displayed inside the button.
///   This can be any other component, such as `text`, `image`, or a combination of them.
///
/// # Example
///
/// ```
/// # use std::sync::Arc;
/// # use tessera_ui::{Color, Dp};
/// # use tessera_ui_basic_components::{
/// #     button::{button, ButtonArgsBuilder},
/// #     ripple_state::RippleState,
/// #     text::{text, TextArgsBuilder},
/// # };
/// #
/// // 1. Create a ripple state to manage the effect.
/// let ripple_state = Arc::new(RippleState::new());
///
/// // 2. Define the button's properties using the builder pattern.
/// let args = ButtonArgsBuilder::default()
///     .color(Color::new(0.2, 0.5, 0.8, 1.0)) // A nice blue
///     .padding(Dp(12.0))
///     .on_click(Arc::new(|| {
///         println!("Button was clicked!");
///     }))
///     .build()
///     .unwrap();
///
/// // 3. Call the button component, passing the args, state, and a child content closure.
/// button(args, ripple_state, || {
///     text(
///         TextArgsBuilder::default()
///             .text("Click Me".to_string())
///             .color(Color::WHITE)
///             .build()
///             .unwrap(),
///     );
/// });
/// ```
#[tessera]
pub fn button(args: impl Into<ButtonArgs>, ripple_state: Arc<RippleState>, child: impl FnOnce()) {
    let button_args: ButtonArgs = args.into();

    // Create interactive surface for button
    surface(create_surface_args(&button_args), Some(ripple_state), child);
}

/// Create surface arguments based on button configuration
fn create_surface_args(args: &ButtonArgs) -> crate::surface::SurfaceArgs {
    let mut builder = SurfaceArgsBuilder::default();

    // Set width if available
    if let Some(width) = args.width {
        builder = builder.width(width);
    }

    // Set height if available
    if let Some(height) = args.height {
        builder = builder.height(height);
    }

    builder
        .color(args.color)
        .hover_color(args.hover_color)
        .shape(args.shape)
        .padding(args.padding)
        .border_width(args.border_width)
        .border_color(args.border_color)
        .ripple_color(args.ripple_color)
        .on_click(Some(args.on_click.clone()))
        .build()
        .unwrap()
}

/// Convenience constructors for common button styles
impl ButtonArgs {
    /// Create a primary button with default blue styling
    pub fn primary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.2, 0.5, 0.8, 1.0)) // Blue
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a secondary button with gray styling
    pub fn secondary(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.6, 0.6, 0.6, 1.0)) // Gray
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a success button with green styling
    pub fn success(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.1, 0.7, 0.3, 1.0)) // Green
            .on_click(on_click)
            .build()
            .unwrap()
    }

    /// Create a danger button with red styling
    pub fn danger(on_click: Arc<dyn Fn() + Send + Sync>) -> Self {
        ButtonArgsBuilder::default()
            .color(Color::new(0.8, 0.2, 0.2, 1.0)) // Red
            .on_click(on_click)
            .build()
            .unwrap()
    }
}

/// Builder methods for fluent API
impl ButtonArgs {
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn with_hover_color(mut self, hover_color: Color) -> Self {
        self.hover_color = Some(hover_color);
        self
    }

    pub fn with_padding(mut self, padding: Dp) -> Self {
        self.padding = padding;
        self
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }

    pub fn with_width(mut self, width: DimensionValue) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: DimensionValue) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_ripple_color(mut self, ripple_color: Color) -> Self {
        self.ripple_color = ripple_color;
        self
    }

    pub fn with_border(mut self, width: f32, color: Option<Color>) -> Self {
        self.border_width = width;
        self.border_color = color;
        self
    }
}
