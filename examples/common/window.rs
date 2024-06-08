use std::time::Duration;
use derive_builder::Builder;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::Span;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, BorderType, Clear};
use ratatui::widgets::Widget;
use tachyonfx::{Effect, FilterMode, IntoEffect, Shader};


fn open_window(
    title: &'static str,
    border_style: Style,
    title_style: Style,
    content_style: Style,
    open_fx: Effect,
    content_fx: Effect,
) -> OpenWindow {
    let title = Line::from(vec![
        Span::from("┫").style(border_style),
        Span::from(" ").style(title_style),
        Span::from(title).style(title_style),
        Span::from(" ").style(title_style),
        Span::from("┣").style(border_style),
    ]);

    OpenWindow::builder()
        .title(title)
        .border_style(border_style)
        .border_type(BorderType::Rounded)
        .background(content_style)
        .pre_render_fx(open_fx)
        .content_fx(content_fx)
        .build()
        .unwrap()
}

#[derive(Builder, Clone)]
#[builder(pattern = "owned")]
pub struct OpenWindow {
    title: Line<'static>,
    #[builder(default, setter(strip_option))]
    pre_render_fx: Option<Effect>, // for setting up geometry etc
    #[builder(default, setter(strip_option))]
    parent_window_fx: Option<Effect>, // applied to whole buffer
    #[builder(default, setter(strip_option))]
    content_fx: Option<Effect>, // applied to content area
    title_style: Style,
    border_style: Style,
    border_type: BorderType,
    background: Style,
}

impl From<OpenWindowBuilder> for Effect {
    fn from(value: OpenWindowBuilder) -> Self {
        value.build().unwrap().into_effect()
    }
}

impl OpenWindow {
    pub fn builder() -> OpenWindowBuilder {
        OpenWindowBuilder::default()
    }

    pub fn screen_area(&mut self, area: Rect) {
        if let Some(fx) = self.parent_window_fx.as_mut() {
            fx.set_area(area);
        }
    }

    fn window_block(&self) -> Block {
        Block::new()
            .borders(Borders::ALL)
            .title_style(self.border_style)
            .title(self.title.clone())
            .border_style(self.border_style)
            .border_type(self.border_type)
            .style(self.background)
    }

    pub fn processing_content_fx(&mut self, duration: Duration, buf: &mut Buffer, area: Rect) {
        if let Some(fx) = self.content_fx.as_mut() {
            if fx.running() {
                fx.process(duration, buf, area);
            }
        }
    }
}

impl Shader for OpenWindow {
    fn process(
        &mut self,
        duration: Duration,
        buf: &mut Buffer,
        area: Rect
    ) -> Option<Duration> {
        if let Some(parent_window_fx) = self.parent_window_fx.as_mut() {
            parent_window_fx.process(duration, buf, area);
            if parent_window_fx.done() {
                self.parent_window_fx = None;
            }
        }

        let overflow = match self.pre_render_fx.as_mut() {
            Some(fx) if fx.running() => fx.process(duration, buf, area),
            _                        => Some(duration)
        };

        let area = self.pre_render_fx.as_ref()
            .map(Effect::area)
            .flatten()
            .map(|area| area.clamp(buf.area))
            .unwrap_or(area);

        if let Some(content_fx) = self.content_fx.as_mut() {
            content_fx.set_area(area)
        }

        Clear.render(area, buf);
        self.window_block().render(area, buf);

        overflow
    }


    fn done(&self) -> bool {
        self.pre_render_fx.is_none()
            || self.pre_render_fx.as_ref().is_some_and(Effect::done)
    }

    fn clone_box(&self) -> Box<dyn Shader> {
        Box::new(self.clone())
    }

    fn area(&self) -> Option<Rect> {
        self.pre_render_fx.as_ref()
            .map(Effect::area)
            .unwrap_or(None)
    }

    fn set_area(&mut self, area: Rect) {
        if let Some(open_window_fx) = self.pre_render_fx.as_mut() {
            open_window_fx.set_area(area);
        }
    }

    fn cell_selection(&mut self, _strategy: FilterMode) {
        todo!()
    }
}