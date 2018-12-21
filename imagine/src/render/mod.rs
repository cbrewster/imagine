use crate::{Interaction, Widget, WidgetComponent, WidgetId};
use specs::{Component, DenseVecStorage, Entities, WriteStorage};
use std::any::Any;
use webrender::api::DisplayListBuilder;

pub struct RenderContext<'a> {
    pub builder: &'a mut DisplayListBuilder,
    next_tag_identifier: u64,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(builder: &'a mut DisplayListBuilder) -> RenderContext<'a> {
        RenderContext {
            builder,
            next_tag_identifier: 0,
        }
    }

    pub fn next_tag_identifier(&mut self) -> u64 {
        let identifier = self.next_tag_identifier;
        self.next_tag_identifier += 1;
        identifier
    }
}

pub struct InteractionContext<'a, 'b> {
    pub(crate) entities: &'a Entities<'b>,
    pub(crate) widgets: &'a mut WriteStorage<'b, WidgetComponent>,
}

impl InteractionContext<'_, '_> {
    pub fn send_message<M: Any>(&mut self, widget_id: WidgetId, message: M) {
        if let Some(widget) = self.widgets.get_mut(widget_id.0) {
            widget.update(Box::new(message));
        }
    }

    pub fn create_widget<W: Widget + 'static>(&mut self, widget: W) -> WidgetId {
        WidgetId(
            self.entities
                .build_entity()
                .with(
                    WidgetComponent {
                        inner: Box::new(widget),
                    },
                    self.widgets,
                )
                .build(),
        )
    }
}

pub struct ClickListener {
    pub(crate) on_click: Box<dyn Fn(&mut InteractionContext) -> () + Send + Sync + 'static>,
}

impl ClickListener {
    pub fn new<F>(click: F) -> ClickListener
    where
        F: Fn(&mut InteractionContext) -> () + Send + Sync + 'static,
    {
        ClickListener {
            on_click: Box::new(click),
        }
    }
}

impl Component for ClickListener {
    type Storage = DenseVecStorage<Self>;
}

pub(crate) struct Event {
    pub(crate) event: Interaction,
}

impl Event {
    pub(crate) fn new(event: Interaction) -> Event {
        Event { event }
    }
}

impl Component for Event {
    type Storage = DenseVecStorage<Self>;
}

pub(crate) struct Interactive {
    pub(crate) tag: u64,
}

impl Interactive {
    pub(crate) fn new(tag: u64) -> Interactive {
        Interactive { tag }
    }
}

impl Component for Interactive {
    type Storage = DenseVecStorage<Self>;
}
