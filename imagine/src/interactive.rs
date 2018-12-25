use crate::{Widget, WidgetComponent, WidgetId};
use specs::{Component, DenseVecStorage, Entities, WriteStorage};
use std::any::Any;

pub trait Message: Any + Send + Sync {}

impl<T> Message for T where T: Any + Send + Sync {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Interaction {
    Hovered(bool),
    MouseDown,
    MouseUp,
}

pub struct WidgetContext<'a, 'b, M: Message> {
    pub(crate) entities: &'a Entities<'b>,
    pub(crate) widgets: &'a mut WriteStorage<'b, WidgetComponent>,
    pub(crate) click_listeners: &'a mut WriteStorage<'b, ClickListener<M>>,
}

impl<'a, 'b, M: Message> WidgetContext<'a, 'b, M> {
    pub(crate) fn new(
        entities: &'a Entities<'b>,
        widgets: &'a mut WriteStorage<'b, WidgetComponent>,
        click_listeners: &'a mut WriteStorage<'b, ClickListener<M>>,
    ) -> WidgetContext<'a, 'b, M> {
        WidgetContext {
            entities,
            widgets,
            click_listeners,
        }
    }

    pub fn send_message<T: Any>(&mut self, widget_id: WidgetId, message: T) {
        let removed = if let Some(widget) = self.widgets.get_mut(widget_id.0) {
            widget.update(Box::new(message))
        } else {
            None
        };

        fn remove_widgets<'a, 'b>(
            entities: &'a Entities<'b>,
            widgets: &'a mut WriteStorage<'b, WidgetComponent>,
            ids: &'a [WidgetId],
        ) {
            for id in ids {
                let widget = widgets.get(id.0).unwrap();
                remove_widgets(entities, widgets, &widget.children());
                entities.delete(id.0).ok();
            }
        }

        if let Some(removed) = removed {
            remove_widgets(self.entities, self.widgets, &removed);
        }
    }

    pub fn add_click_listener(&mut self, widget_id: WidgetId, listener: ClickListener<M>) {
        self.click_listeners.insert(widget_id.0, listener).ok();
    }

    pub fn remove_widget(&mut self, widget_id: WidgetId) {
        self.entities.delete(widget_id.0).ok();
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

pub struct ClickListener<M: Message> {
    pub(crate) on_click: Box<dyn Fn() -> M + Send + Sync + 'static>,
}

impl<M: Message> ClickListener<M> {
    pub fn new<F>(click: F) -> ClickListener<M>
    where
        F: Fn() -> M + Send + Sync + 'static,
    {
        ClickListener {
            on_click: Box::new(click),
        }
    }
}

impl<M: Message> Component for ClickListener<M> {
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
