#![cfg_attr(target_family = "wasm", no_main)]

use std::sync::Arc;

use gpui::{
    App, Application, Bounds, Context, Corners, RenderImage, SpriteBlend, SpriteFilter, Window,
    WindowBounds, WindowOptions, canvas, fill, point, prelude::*, px, rgb, size,
};
use image::{Frame, ImageBuffer, Rgba};

const GLOW_SIZE: u32 = 64;
/// Side of the checkerboard image, in texels.
const CHECKER_TEXELS: u32 = 4;
/// Logical pixels per checkerboard texel.
const CHECKER_SCALE: f32 = 16.;

/// Paints a solid `#004000` image additively, half over a red square and half over the grey
/// background, so the blended pixels can be read back: `#ff4000` over red, `#408040` over grey.
///
/// Below it, a 4x4 black and white checkerboard drawn 16 times larger twice: with
/// [`SpriteFilter::Nearest`] on the left, every texel a solid block with a one-pixel edge, and with
/// the default [`SpriteFilter::Linear`] on the right, whose edges blend through greys.
struct AdditiveTest {
    glow: Arc<RenderImage>,
    checker: Arc<RenderImage>,
}

impl AdditiveTest {
    fn new() -> Self {
        // RenderImage expects BGRA; with zero red and blue the bytes match RGBA.
        let buffer = ImageBuffer::from_pixel(GLOW_SIZE, GLOW_SIZE, Rgba([0x00, 0x40, 0x00, 0xff]));
        let checker = ImageBuffer::from_fn(CHECKER_TEXELS, CHECKER_TEXELS, |x, y| {
            let shade = if (x + y) % 2 == 0 { 0xff } else { 0x00 };
            Rgba([shade, shade, shade, 0xff])
        });
        Self {
            glow: Arc::new(RenderImage::new([Frame::new(buffer)])),
            checker: Arc::new(RenderImage::new([Frame::new(checker)])),
        }
    }
}

impl Render for AdditiveTest {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let glow = self.glow.clone();
        let checker = self.checker.clone();
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
                    glow.clone(),
                    0,
                    false,
                    SpriteBlend::Additive,
                    SpriteFilter::Linear,
                ) {
                    log::error!("failed to paint additive image: {error:#}");
                }

                let checker_side = px(CHECKER_TEXELS as f32 * CHECKER_SCALE);
                for (x, filter) in [
                    (px(40.), SpriteFilter::Nearest),
                    (px(200.), SpriteFilter::Linear),
                ] {
                    let checker_bounds =
                        Bounds::new(point(x, px(260.)), size(checker_side, checker_side));
                    if let Err(error) = window.paint_image_blended(
                        checker_bounds,
                        checker_bounds,
                        Corners::default(),
                        checker.clone(),
                        0,
                        false,
                        SpriteBlend::Normal,
                        filter,
                    ) {
                        log::error!("failed to paint checkerboard: {error:#}");
                    }
                }
            },
        )
        .size_full()
    }
}

fn run_example(application: Application) {
    application.run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(400.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                if let Some(specs) = window.gpu_specs() {
                    log::info!(
                        "additive-test GPU: {} / {} / {}",
                        specs.device_name,
                        specs.driver_name,
                        specs.driver_info
                    );
                }
                cx.new(|_| AdditiveTest::new())
            },
        )
        .expect("open additive test window");
        cx.activate(true);
    });
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run_example(gpui_platform::application());
}

/// `?backend=webgl` or `?backend=webgpu` forces a renderer so both paths can be checked.
#[cfg(target_family = "wasm")]
fn requested_backend() -> gpui_platform::WebBackendPreference {
    let search = web_sys::window()
        .and_then(|window| window.location().search().ok())
        .unwrap_or_default();
    let requested = |value: &str| {
        search
            .trim_start_matches('?')
            .split('&')
            .any(|parameter| parameter == value)
    };
    if requested("backend=webgpu") {
        gpui_platform::WebBackendPreference::WebGpu
    } else if requested("backend=webgl") {
        gpui_platform::WebBackendPreference::WebGl
    } else {
        gpui_platform::WebBackendPreference::Auto
    }
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    run_example(gpui_platform::application_with_web_backend(
        requested_backend(),
    ));
}
