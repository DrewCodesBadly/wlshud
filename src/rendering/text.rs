use std::{
    cell::{Cell, Ref, RefCell},
    rc::Rc,
};

use skia_safe::{Color, utils::text_utils::Align};
use smithay_client_toolkit::seat::keyboard::{KeyEvent, Keysym};

use crate::rendering::layout::Widget;

/// Text input field.
pub struct TextField {
    text: Rc<RefCell<String>>,
    color: Color,
    text_size: f32,
}

impl Widget for TextField {
    fn draw(
        &mut self,
        canvas: &skia_safe::Canvas,
        paint: &mut skia_safe::Paint,
        area: skia_safe::Rect,
        context: &mut super::AppContext,
    ) {
        paint.set_color(self.color);
        canvas.draw_str_align(
            self.text.borrow().as_str(),
            area.bl(),
            &context.default_font(Some(self.text_size)),
            paint,
            Align::Left,
        );
    }
}

impl TextField {
    pub fn new(text_color: Color, text_size: f32) -> Self {
        Self {
            text: Default::default(),
            color: text_color,
            text_size,
        }
    }

    pub fn input_handler<F: Fn(Ref<String>) + 'static>(
        &self,
        on_changed: F,
    ) -> Box<dyn Fn(KeyEvent)> {
        let rc = self.text.clone();
        Box::new(move |event| {
            if event.keysym == Keysym::BackSpace {
                rc.borrow_mut().pop();
            } else if let Some(c) = event.keysym.key_char() {
                rc.borrow_mut().push(c);
            }
            on_changed(rc.borrow())
        })
    }

    pub fn text(&self) -> String {
        self.text.borrow().clone()
    }
}
