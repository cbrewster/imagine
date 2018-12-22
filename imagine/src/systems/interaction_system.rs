use crate::{
    interactive::{Event, Interaction},
    ClickListener, WidgetComponent, WidgetContext,
};
use specs::{Entities, Join, System, WriteStorage};

pub(crate) struct InteractionSystem;

impl<'a> System<'a> for InteractionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, Event>,
        WriteStorage<'a, WidgetComponent>,
        WriteStorage<'a, ClickListener>,
    );

    fn run(&mut self, (entities, mut events, mut widgets, mut listeners): Self::SystemData) {
        for (event, widget) in (&mut events, &mut widgets).join() {
            widget.handle_interaction(event.event);
        }

        for (entity, event) in (&entities, &mut events).join() {
            if let Interaction::MouseDown = event.event {
                if let Some(listener) = listeners.get_mut(entity) {
                    let mut context = WidgetContext {
                        entities: &entities,
                        widgets: &mut widgets,
                    };
                    (listener.on_click)(&mut context);
                }
            }
        }

        events.clear();
    }
}
