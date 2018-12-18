use crate::{
    layout::{BoxConstraint, LayoutResult, Position, SetPosition, Size},
    widget::WidgetComponent,
    WindowComponent,
};
use specs::{Entity, Join, ReadStorage, System, WriteStorage};

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
            let root = match window.root {
                Some(root) => root,
                None => continue,
            };

            let constraint = BoxConstraint::new(Size::zero(), window.layout_size());
            request_layout(&mut sizes, &mut positions, &mut widgets, constraint, root);
            positions.insert(root, Position::zero()).ok();
        }
    }
}

fn request_layout<'a>(
    sizes: &mut WriteStorage<'a, Size>,
    positions: &mut WriteStorage<'a, Position>,
    widgets: &mut WriteStorage<'a, WidgetComponent>,
    constraint: BoxConstraint,
    node: Entity,
) -> Size {
    let mut size_prev_child = None;
    loop {
        let result = widgets.get_mut(node).unwrap().layout(
            SetPosition::new(positions),
            constraint,
            size_prev_child,
        );
        match result {
            LayoutResult::Size(size) => {
                sizes.insert(node, size).ok();
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
