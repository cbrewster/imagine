use crate::{
    layout::{BoxConstraint, LayoutContext, LayoutResult, Position, Size},
    widget::WidgetComponent,
    WidgetId, WindowComponent,
};
use specs::{Join, ReadStorage, System, WriteStorage};

pub(crate) struct LayoutSystem;

impl<'a> System<'a> for LayoutSystem {
    type SystemData = (
        WriteStorage<'a, Size>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, WidgetComponent>,
        ReadStorage<'a, WindowComponent>,
    );

    fn run(&mut self, (mut sizes, mut positions, mut widgets, windows): Self::SystemData) {
        for window in windows.join() {
            if !window.dirty() {
                continue;
            }
            let layout_size = window.layout_size();
            let constraint = BoxConstraint::new(
                Size::zero(),
                Size::new(layout_size.width, layout_size.height),
            );
            request_layout(
                &mut sizes,
                &mut positions,
                &mut widgets,
                constraint,
                window.root,
            );
            positions.insert(window.root.0, Position::zero()).ok();
        }
    }
}

fn request_layout<'a>(
    sizes: &mut WriteStorage<'a, Size>,
    positions: &mut WriteStorage<'a, Position>,
    widgets: &mut WriteStorage<'a, WidgetComponent>,
    constraint: BoxConstraint,
    widget: WidgetId,
) -> Size {
    let mut size_prev_child = None;
    loop {
        let result = widgets.get_mut(widget.0).unwrap().layout(
            &mut LayoutContext::new(positions, sizes),
            constraint,
            size_prev_child,
        );
        match result {
            LayoutResult::Size(size) => {
                sizes.insert(widget.0, size).ok();
                return size;
            }
            LayoutResult::RequestChildSize(child, child_constraint) => {
                size_prev_child = Some(request_layout(
                    sizes,
                    positions,
                    widgets,
                    child_constraint,
                    child,
                ));
            }
        }
    }
}
