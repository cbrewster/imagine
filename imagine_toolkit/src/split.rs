use imagine::Entity;
use imagine::{BoxConstraint, LayoutResult, Position, SetPosition, Size, Widget};

pub struct Split {
    children: [Entity; 2],
    value: f32,
    finished_left: bool,
}

impl Split {
    pub fn new(left: Entity, right: Entity, value: f32) -> Split {
        Split {
            children: [left, right],
            value: value.max(0.0).min(1.0),
            finished_left: false,
        }
    }
}

impl Widget for Split {
    fn layout(
        &mut self,
        mut set_position: SetPosition,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        let [left, right] = self.children;
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
                LayoutResult::RequestChildSize(left, constraint)
            }
            Some(_) => {
                if self.finished_left {
                    set_position.set_position(left, Position::zero());
                    set_position.set_position(
                        right,
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
                    LayoutResult::RequestChildSize(right, constraint)
                }
            }
        }
    }

    fn children(&self) -> &[Entity] {
        &self.children
    }
}
