use imagine::{
    BoxConstraint, InteractiveState, LayoutContext, LayoutResult, Position, Size, Widget, WidgetId,
};

pub struct List {
    widgets: Vec<WidgetId>,
    current_index: usize,
}

impl List {
    pub fn new(widgets: Vec<WidgetId>) -> List {
        List {
            widgets,
            current_index: 0,
        }
    }
}

impl Widget for List {
    fn children(&self) -> Vec<WidgetId> {
        self.widgets.clone()
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
            }
            Some(_) => {
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
        let child_constraint =
            BoxConstraint::new(Size::new(box_constraint.max.width, 0.0), box_constraint.max);
        LayoutResult::RequestChildSize(self.widgets[self.current_index], child_constraint)
    }
}
