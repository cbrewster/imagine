use imagine::{
    BoxConstraint, InteractiveState, LayoutContext, LayoutResult, Position, RenderTreeBuilder,
    Size, Widget, WidgetId,
};

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
    fn create(self, builder: &mut RenderTreeBuilder) -> WidgetId {
        let children = &[self.child];
        builder.create(self, children)
    }

    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        _interactive_state: InteractiveState,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => {
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
                LayoutResult::RequestChildSize(self.child, child_constraint)
            }
            Some(size) => {
                layout_context.set_position(self.child, Position::new(self.top, self.left));
                LayoutResult::Size(Size::new(
                    size.width + (self.right + self.left),
                    size.height + (self.top + self.bottom),
                ))
            }
        }
    }
}
