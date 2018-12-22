use imagine::{BoxConstraint, LayoutContext, LayoutResult, Position, Size, Widget, WidgetId};

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
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => LayoutResult::RequestChildSize(self.child, box_constraint),
            Some(size) => {
                let xdiff = box_constraint.max.width - size.width;
                let ydiff = box_constraint.max.height - size.height;
                layout_context.set_position(self.child, Position::new(xdiff / 2.0, ydiff / 2.0));
                LayoutResult::Size(box_constraint.max)
            }
        }
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.child]
    }
}
