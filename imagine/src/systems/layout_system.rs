use crate::{
    layout::{BoxConstraint, LayoutContext, Position, Size},
    text::FinalText,
    widget::WidgetComponent,
    WindowComponent,
};
use specs::{Join, ReadStorage, System, WriteStorage};

pub(crate) struct LayoutSystem;

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        WriteStorage<'a, Size>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, FinalText>,
        ReadStorage<'a, WidgetComponent>,
        ReadStorage<'a, WindowComponent>,
    );

    fn run(&mut self, (mut sizes, mut positions, mut text, widgets, windows): Self::SystemData) {
        for window in windows.join() {
            if !window.dirty() {
                continue;
            }
            let layout_size = window.layout_size();
            let constraint = BoxConstraint::new(
                Size::zero(),
                Size::new(layout_size.width, layout_size.height),
            );

            let mut layout_context = LayoutContext::new(
                &mut positions,
                &mut sizes,
                &mut text,
                &widgets,
                &window.font,
            );
            layout_context.layout_widget(window.root, constraint);
            positions.insert(window.root.0, Position::zero()).ok();
        }
    }
}
