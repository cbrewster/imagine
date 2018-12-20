use imagine::{
    BoxConstraint, InteractiveState, LayoutContext, LayoutResult, Position, Size, Widget, WidgetId,
};

#[derive(Copy, Clone, Eq, PartialEq)]
enum FlexPhase {
    NonFlex,
    Flex,
}

#[derive(Copy, Clone)]
pub enum FlexDirection {
    Horizontal,
    Vertical,
}

impl FlexDirection {
    pub fn major_axis(self, size: Size) -> f32 {
        match self {
            FlexDirection::Horizontal => size.width,
            FlexDirection::Vertical => size.height,
        }
    }

    pub fn minor_axis(self, size: Size) -> f32 {
        match self {
            FlexDirection::Horizontal => size.height,
            FlexDirection::Vertical => size.width,
        }
    }

    pub fn major_minor_to_position(self, major: f32, minor: f32) -> Position {
        match self {
            FlexDirection::Horizontal => Position::new(major, minor),
            FlexDirection::Vertical => Position::new(minor, major),
        }
    }

    pub fn major_minor_to_size(self, major: f32, minor: f32) -> Size {
        match self {
            FlexDirection::Horizontal => Size::new(major, minor),
            FlexDirection::Vertical => Size::new(minor, major),
        }
    }
}

#[derive(Copy, Clone)]
pub enum FlexAlign {
    Top,
    Middle,
    Baseline,
}

#[derive(Copy, Clone, Debug)]
pub enum FlexItem {
    NonFlex(WidgetId),
    Flex(WidgetId, usize),
}

pub struct Flex {
    children: Vec<FlexItem>,
    index: usize,
    phase: FlexPhase,
    flex_direction: FlexDirection,
    flex_align: FlexAlign,
    non_flex_major: f32,
    total_flex: usize,
    minor: f32,
}

impl Flex {
    pub fn new(
        children: Vec<FlexItem>,
        flex_direction: FlexDirection,
        flex_align: FlexAlign,
    ) -> Flex {
        Flex {
            children,
            index: 0,
            flex_direction,
            flex_align,
            phase: FlexPhase::NonFlex,
            non_flex_major: 0.0,
            total_flex: 0,
            minor: 0.0,
        }
    }
}

impl Flex {
    fn get_next(&self, start: usize) -> Option<usize> {
        for index in start..self.children.len() {
            match (self.phase, self.children[index]) {
                (FlexPhase::Flex, FlexItem::Flex(..)) => return Some(index),
                (FlexPhase::NonFlex, FlexItem::NonFlex(..)) => return Some(index),
                _ => continue,
            }
        }
        None
    }

    fn position_children(
        &self,
        box_constraint: BoxConstraint,
        layout_context: &mut LayoutContext,
    ) -> LayoutResult {
        let mut major = 0.0;
        for child in &self.children {
            let child = match child {
                FlexItem::Flex(child, _) => *child,
                FlexItem::NonFlex(child) => *child,
            };
            let child_size = layout_context.get_size(child);

            let minor = match self.flex_align {
                FlexAlign::Top => 0.0,
                FlexAlign::Middle => {
                    (self.minor - self.flex_direction.minor_axis(child_size)) / 2.0
                }
                FlexAlign::Baseline => self.minor - self.flex_direction.minor_axis(child_size),
            };

            layout_context.set_position(
                child,
                self.flex_direction.major_minor_to_position(major, minor),
            );

            major += self.flex_direction.major_axis(child_size);
        }
        let total_major = self.flex_direction.major_axis(box_constraint.max);

        LayoutResult::Size(
            self.flex_direction
                .major_minor_to_size(total_major, self.minor),
        )
    }
}

impl Widget for Flex {
    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        _interactive_state: InteractiveState,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => {
                if self.children.is_empty() {
                    return LayoutResult::Size(box_constraint.min);
                }
                self.phase = FlexPhase::NonFlex;
                self.non_flex_major = 0.0;
                self.minor = 0.0;
                self.total_flex = self
                    .children
                    .iter()
                    .map(|child| match child {
                        FlexItem::NonFlex(..) => 0,
                        FlexItem::Flex(_, flex) => *flex,
                    })
                    .sum();

                if let Some(index) = self.get_next(0) {
                    self.index = index;
                } else {
                    self.phase = FlexPhase::Flex;
                    self.index = 0;
                }
            }
            Some(size) => {
                self.minor = self.minor.max(self.flex_direction.minor_axis(size));

                if let FlexItem::NonFlex(_) = self.children[self.index] {
                    self.non_flex_major += self.flex_direction.major_axis(size);
                }

                if let Some(index) = self.get_next(self.index + 1) {
                    self.index = index;
                } else if self.phase == FlexPhase::NonFlex {
                    self.phase = FlexPhase::Flex;
                    if let Some(index) = self.get_next(0) {
                        self.index = index;
                    } else {
                        return self.position_children(box_constraint, layout_context);
                    }
                } else {
                    return self.position_children(box_constraint, layout_context);
                }
            }
        }
        let (min_major, max_major, child) = match self.children[self.index] {
            FlexItem::NonFlex(child) => (0.0, std::f32::INFINITY, child),
            FlexItem::Flex(child, flex) => {
                let total_major = self.flex_direction.major_axis(box_constraint.max);
                let remaining = (total_major - self.non_flex_major).max(0.0);
                let major = remaining * (flex as f32 / self.total_flex as f32);

                (major, major, child)
            }
        };
        let child_constraint = match self.flex_direction {
            FlexDirection::Horizontal => BoxConstraint::new(
                Size::new(min_major, box_constraint.min.height),
                Size::new(max_major, box_constraint.max.height),
            ),
            FlexDirection::Vertical => BoxConstraint::new(
                Size::new(box_constraint.min.width, min_major),
                Size::new(box_constraint.max.width, max_major),
            ),
        };

        LayoutResult::RequestChildSize(child, child_constraint)
    }

    fn children(&self) -> Vec<WidgetId> {
        self.children
            .iter()
            .map(|item| match item {
                FlexItem::NonFlex(child) => *child,
                FlexItem::Flex(child, _) => *child,
            })
            .collect()
    }
}
