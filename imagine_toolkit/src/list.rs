use imagine::{
    BoxConstraint, InteractiveState, LayoutContext, LayoutResult, Position, RenderTreeBuilder,
    Size, Widget, WidgetId,
};

pub struct List {
    widgets: Vec<WidgetId>,
    current_index: usize,
    remaining_height: f32,
}

impl List {
    pub fn new(widgets: Vec<WidgetId>) -> List {
        List {
            widgets,
            current_index: 0,
            remaining_height: 0.0,
        }
    }
}

impl Widget for List {
    fn create(self, builder: &mut RenderTreeBuilder) -> WidgetId {
        let children = self.widgets.clone();
        builder.create(self, &children)
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
                self.current_index = 0;
                self.remaining_height = box_constraint.max.height;
            }
            Some(size) => {
                self.remaining_height -= size.height;

                self.current_index += 1;

                if self.current_index >= self.widgets.len() {
                    let mut height = 0.0;

                    for widget in &self.widgets {
                        let size = layout_context.get_size(*widget);
                        layout_context.set_position(*widget, Position::new(0.0, height));
                        height += size.height;
                    }

                    return LayoutResult::Size(Size::new(box_constraint.max.width, height));
                }
            }
        }

        let child_constraint = BoxConstraint::new(
            Size::new(box_constraint.max.width, 0.0),
            Size::new(box_constraint.max.width, self.remaining_height.max(0.0)),
        );
        LayoutResult::RequestChildSize(self.widgets[self.current_index], child_constraint)
    }
}
