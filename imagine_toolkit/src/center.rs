use imagine::{BoxConstraint, LayoutContext, Position, Size, Widget, WidgetId};

pub struct Center {
    child: WidgetId,
}

impl Center {
    pub fn new(child: WidgetId) -> Center {
        Center { child }
    }
}

impl Widget for Center {
    fn layout(
        &self,
        _id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let child_size = layout_context.layout_widget(self.child, box_constraint);
        let xdiff = box_constraint.max.width - child_size.width;
        let ydiff = box_constraint.max.height - child_size.height;
        layout_context.set_position(self.child, Position::new(xdiff / 2.0, ydiff / 2.0));
        box_constraint.max
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.child]
    }
}
