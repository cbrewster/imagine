use imagine::{BoxConstraint, LayoutContext, Position, Size, Widget, WidgetId};

pub struct Padding {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    child: WidgetId,
}

impl Padding {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32, child: WidgetId) -> Padding {
        Padding {
            top,
            bottom,
            left,
            right,
            child,
        }
    }
}

impl Widget for Padding {
    fn layout(
        &self,
        _id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let child_constraint = BoxConstraint::new(
            Size::new(
                box_constraint.min.width - (self.right + self.left),
                box_constraint.min.height - (self.top + self.bottom),
            ),
            Size::new(
                box_constraint.max.width - (self.right + self.left),
                box_constraint.max.height - (self.top + self.bottom),
            ),
        );
        let child_size = layout_context.layout_widget(self.child, child_constraint);
        layout_context.set_position(self.child, Position::new(self.top, self.left));
        Size::new(
            child_size.width + (self.right + self.left),
            child_size.height + (self.top + self.bottom),
        )
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.child]
    }
}
