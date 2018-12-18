use imagine::{BoxConstraint, LayoutContext, LayoutResult, Position, Size, Widget, WidgetId};

pub struct Split {
    left: WidgetId,
    right: WidgetId,
    value: f32,
    finished_left: bool,
}

impl Split {
    pub fn new(left: WidgetId, right: WidgetId, value: f32) -> Split {
        Split {
            left,
            right,
            value: value.max(0.0).min(1.0),
            finished_left: false,
        }
    }
}

impl Widget for Split {
    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => {
                self.finished_left = false;
                let constraint = BoxConstraint::new(
                    Size::zero(),
                    Size::new(
                        box_constraint.max.width * self.value,
                        box_constraint.max.height,
                    ),
                );
                LayoutResult::RequestChildSize(self.left, constraint)
            }
            Some(_) => {
                if self.finished_left {
                    layout_context.set_position(self.left, Position::zero());
                    layout_context.set_position(
                        self.right,
                        Position::new(box_constraint.max.width * self.value, 0.0),
                    );

                    LayoutResult::Size(Size::new(
                        box_constraint.max.width,
                        box_constraint.max.height,
                    ))
                } else {
                    self.finished_left = true;
                    let constraint = BoxConstraint::new(
                        Size::zero(),
                        Size::new(
                            box_constraint.max.width * (1.0 - self.value),
                            box_constraint.max.height,
                        ),
                    );
                    LayoutResult::RequestChildSize(self.right, constraint)
                }
            }
        }
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.left, self.right]
    }
}
