use imagine::{BoxConstraint, LayoutContext, Position, Size, Widget, WidgetId};

pub struct List {
    widgets: Vec<WidgetId>,
}

impl List {
    pub fn new(widgets: Vec<WidgetId>) -> List {
        List { widgets }
    }
}

impl Widget for List {
    fn children(&self) -> Vec<WidgetId> {
        self.widgets.clone()
    }

    fn layout(
        &self,
        _id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let mut current_y = 0.0;
        for child in &self.widgets {
            let child_size = layout_context.layout_widget(
                *child,
                BoxConstraint::new(
                    Size::new(box_constraint.max.width, 0.0),
                    Size::new(box_constraint.max.width, std::f32::INFINITY),
                ),
            );

            layout_context.set_position(*child, Position::new(0.0, current_y));
            current_y += child_size.height;
        }
        box_constraint.constrain(Size::new(box_constraint.max.width, current_y))
    }
}
