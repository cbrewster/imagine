use imagine::{BoxConstraint, Entity, LayoutResult, Position, SetPosition, Size, Widget};

pub struct Padding {
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    children: [Entity; 1],
}

impl Padding {
    pub fn new(top: f32, bottom: f32, left: f32, right: f32, child: Entity) -> Padding {
        Padding {
            top,
            bottom,
            left,
            right,
            children: [child],
        }
    }
}

impl Widget for Padding {
    fn layout(
        &mut self,
        mut set_position: SetPosition,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        let [child] = self.children;
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
                LayoutResult::RequestChildSize(child, child_constraint)
            }
            Some(size) => {
                set_position.set_position(child, Position::new(self.top, self.left));
                LayoutResult::Size(Size::new(
                    size.width + (self.right + self.left),
                    size.height + (self.top + self.bottom),
                ))
            }
        }
    }

    fn children(&self) -> &[Entity] {
        &self.children
    }
}
