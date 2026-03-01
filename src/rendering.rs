use crate::{config::ConfigData, rendering::layout::Widget, searching::SearchResults};
use std::{
    cell::Cell,
    rc::Rc,
    sync::mpsc::{Receiver, Sender},
    time::Instant,
};

use skia_safe::{
    BlendMode, Canvas, Color4f, ColorSpace, Font, FontMgr, ImageInfo, Paint, Rect,
    textlayout::FontCollection,
};
use smithay_client_toolkit::{
    reexports::client::{QueueHandle, protocol::wl_shm},
    seat::keyboard::{KeyEvent, KeyboardHandler},
    shell::WaylandSurface,
};
use tween::{CubicOut, Tween, Tweener};

use crate::window::HUDWindow;

pub mod layout;
pub mod text;

const BACKGROUND_ALPHA: f32 = 0.5;

pub type FadeTweenType = CubicOut;
pub fn create_app_fade_tween(start: f32, end: f32) -> Tweener<f32, f64, FadeTweenType> {
    Tweener::new(start, end, 0.2, FadeTweenType::new())
}

pub type AnimatedValue = Rc<Cell<f32>>;
struct Animation {
    val: AnimatedValue,
    tweener: Tweener<f32, f64, Box<dyn Tween<f32>>>,
}

pub struct AppContext {
    animations: Vec<Animation>,
    config_data: ConfigData,
    search_results: SearchResults,
    query_send: Sender<String>,
    key_handler: Option<Box<dyn Fn(KeyEvent)>>,
    font_collection: FontCollection,
}

impl AppContext {
    pub fn new(query_send: Sender<String>) -> Self {
        let mut font_collection = FontCollection::new();
        font_collection.set_default_font_manager(FontMgr::new(), None);
        Self {
            animations: Vec::new(),
            config_data: ConfigData::default(),
            key_handler: None,
            search_results: SearchResults::default(),
            query_send,
            font_collection,
        }
    }
    pub fn update_animations(&mut self, delta: f64) {
        // retains only animations which are currently ongoing.
        self.animations.retain_mut(|a| {
            a.val.set(a.tweener.move_by(delta));
            !a.tweener.is_finished()
        });
    }

    pub fn add_animation(
        &mut self,
        val: AnimatedValue,
        tweener: Tweener<f32, f64, Box<dyn Tween<f32>>>,
    ) {
        val.set(tweener.initial_value());
        if let Some(a) = self.animations.iter_mut().find(|a| a.val == val) {
            a.tweener = tweener;
        } else {
            self.animations.push(Animation { val, tweener });
        }
    }

    pub fn set_key_handler(&mut self, handler: Box<dyn Fn(KeyEvent)>) {
        self.key_handler = Some(handler);
    }

    pub fn handle_key_press(&mut self, key: KeyEvent) {
        if let Some(h) = self.key_handler.as_ref() {
            h(key)
        }
    }

    pub fn default_font(&mut self, size: Option<f32>) -> Font {
        Font::from_typeface(self.font_collection.default_fallback().unwrap(), size)
    }
}

impl HUDWindow {
    pub fn start_closing_animation(&mut self) {
        if self.in_closing_animation {
            return;
        }

        self.in_closing_animation = true;
        self.app_fade_tweener = create_app_fade_tween(self.app_fade_pos, 0.);
        // will be committed next redraw
        self.layer_surface.set_keyboard_interactivity(
            smithay_client_toolkit::shell::wlr_layer::KeyboardInteractivity::None,
        );
    }

    pub fn draw(&mut self, qh: &QueueHandle<Self>) {
        // Checks for updated results before redrawing.
        if let Ok(v) = self.search_results_receiver.try_recv() {
            self.app_context.search_results = v;
        }

        // Update animation states before drawing
        let now = Instant::now();
        let frame_delta = now.duration_since(self.last_frame_time).as_secs_f64();
        self.last_frame_time = now;
        self.app_fade_pos = self.app_fade_tweener.move_by(frame_delta);
        if self.in_closing_animation && self.app_fade_tweener.is_finished() {
            self.should_close = true;
            return;
        }
        self.app_context.update_animations(frame_delta);

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(
                    self.width as i32,
                    self.height as i32,
                    self.width as i32 * 4,
                    wl_shm::Format::Argb8888,
                )
                .unwrap()
                .0
        });

        {
            let canvas = match self.pool.canvas(buffer) {
                Some(canvas) => canvas,
                None => {
                    // double-buffer if needed
                    let (second_buffer, canvas) = self
                        .pool
                        .create_buffer(
                            self.width as i32,
                            self.height as i32,
                            self.width as i32 * 4,
                            wl_shm::Format::Argb8888,
                        )
                        .unwrap();
                    *buffer = second_buffer;
                    canvas
                }
            };
            let img_info = ImageInfo::new(
                (self.width as i32, self.height as i32),
                skia_safe::ColorType::BGRA8888,
                skia_safe::AlphaType::Premul,
                ColorSpace::new_srgb(),
            );
            let sk_canvas = Canvas::from_raster_direct(&img_info, canvas, None, None).unwrap();
            let mut paint = Paint::default();
            paint.set_anti_alias(true);

            // draw...
            sk_canvas.clear(Color4f::new(0., 0., 0., BACKGROUND_ALPHA));
            self.app_layout.draw(
                &sk_canvas,
                &mut paint,
                Rect::from_wh(self.width as f32, self.height as f32),
                &mut self.app_context,
            );

            // Multiplies by the fade animation amount.
            sk_canvas.draw_color(
                Color4f::new(1., 1., 1., self.app_fade_pos),
                Some(BlendMode::Modulate),
            );
        }

        let surface = self.layer_surface.wl_surface();

        surface.damage_buffer(0, 0, self.width as i32, self.height as i32);
        // requests another frame for constant redraws
        surface.frame(qh, surface.clone());
        buffer.attach_to(surface).unwrap();
        self.layer_surface.commit();
    }
}
