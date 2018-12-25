use crate::{
    interactive::{Event, Interaction},
    ClickListener, Message, MessageQueue, WidgetComponent,
};
use specs::{Entities, Join, System, Write, WriteStorage, ReadStorage};
use std::marker::PhantomData;

pub(crate) struct InteractionSystem<M: Message> {
    phantom: PhantomData<M>,
}

impl<M: Message> Default for InteractionSystem<M> {
    fn default() -> InteractionSystem<M> {
        InteractionSystem {
            phantom: PhantomData,
        }
    }
}

impl<'a, M: Message> System<'a> for InteractionSystem<M> {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Event>,
        WriteStorage<'a, WidgetComponent>,
        ReadStorage<'a, ClickListener<M>>,
        Write<'a, MessageQueue<M>>,
    );

    fn run(
        &mut self,
        (entities, mut events, mut widgets, listeners, mut queue): Self::SystemData,
    ) {
        for (event, widget) in (&mut events, &mut widgets).join() {
            widget.handle_interaction(event.event);
        }

        for (entity, event) in (&entities, &mut events).join() {
            if let Interaction::MouseDown = event.event {
                if let Some(listener) = listeners.get(entity) {
                    queue.0.push((listener.on_click)());
                }
            }
        }

        events.clear();
    }
}
