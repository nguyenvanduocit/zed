#![cfg_attr(target_family = "wasm", no_main)]

use std::sync::Arc;

use gpui::{
    App, Bounds, Context, Corners, RenderImage, SpriteBlend, Window, WindowBounds, WindowOptions,
    canvas, fill, point, prelude::*, px, rgb, size,
};
use gpui_platform::application;
use image::{Frame, ImageBuffer, Rgba};

const GLOW_SIZE: u32 = 64;

/// Paints a solid `#004000` image additively, half over a red square and half over the grey
/// background, so the blended pixels can be read back: `#ff4000` over red, `#408040` over grey.
struct AdditiveTest {
    glow: Arc<RenderImage>,
}

impl AdditiveTest {
    fn new() -> Self {
        // RenderImage expects BGRA; with zero red and blue the bytes match RGBA.
        let buffer = ImageBuffer::from_pixel(GLOW_SIZE, GLOW_SIZE, Rgba([0x00, 0x40, 0x00, 0xff]));
        Self {
            glow: Arc::new(RenderImage::new([Frame::new(buffer)])),
        }
    }
}

impl Render for AdditiveTest {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let glow = self.glow.clone();
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                window.paint_quad(fill(bounds, rgb(0x404040)));

                let red_square = Bounds::new(point(px(100.), px(100.)), size(px(128.), px(128.)));
                window.paint_quad(fill(red_square, rgb(0xff0000)));

                let glow_bounds = Bounds::new(
                    point(px(196.), px(132.)),
                    size(px(GLOW_SIZE as f32), px(GLOW_SIZE as f32)),
                );
                if let Err(error) = window.paint_image_blended(
                    glow_bounds,
                    glow_bounds,
                    Corners::default(),
                    glow,
                    0,
                    false,
                    SpriteBlend::Additive,
                ) {
                    log::error!("failed to paint additive image: {error:#}");
                }
            },
        )
        .size_full()
    }
}

fn run_example() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(400.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(|_| AdditiveTest::new()),
        )
        .expect("open additive test window");
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example();
}
