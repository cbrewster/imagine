use imagine::{BoxConstraint, LayoutContext, Position, Size, Widget, WidgetId};
use std::any::Any;

#[derive(Copy, Clone)]
pub enum FlexDirection {
    Horizontal,
    Vertical,
}

pub enum FlexEvent {
    AddChild(FlexItem),
    RemoveChild,
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

impl FlexItem {
    pub fn widget(self) -> WidgetId {
        match self {
            FlexItem::Flex(widget, _) => widget,
            FlexItem::NonFlex(widget) => widget,
        }
    }
}

pub struct Flex {
    children: Vec<FlexItem>,
    flex_direction: FlexDirection,
    flex_align: FlexAlign,
}

impl Flex {
    pub fn new(
        children: Vec<FlexItem>,
        flex_direction: FlexDirection,
        flex_align: FlexAlign,
    ) -> Flex {
        Flex {
            children,
            flex_direction,
            flex_align,
        }
    }
}

impl Widget for Flex {
    fn layout(
        &self,
        _id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let mut flex_widgets = vec![];
        let mut non_flex_widgets = vec![];
        let mut total_flex = 0;
        let mut non_flex_major = 0.0;
        let mut max_minor: f32 = 0.0;

        // Separate out flex and non-flex children
        for child in &self.children {
            match child {
                FlexItem::Flex(widget, flex) => {
                    total_flex += flex;
                    flex_widgets.push((widget, flex));
                }
                FlexItem::NonFlex(widget) => {
                    non_flex_widgets.push(widget);
                }
            }
        }

        // Layout non-flex children first
        for widget in non_flex_widgets {
            let child_constraint = match self.flex_direction {
                FlexDirection::Horizontal => BoxConstraint::new(
                    Size::new(0.0, box_constraint.min.height),
                    Size::new(std::f32::INFINITY, box_constraint.max.height),
                ),
                FlexDirection::Vertical => BoxConstraint::new(
                    Size::new(box_constraint.min.width, 0.0),
                    Size::new(box_constraint.max.width, std::f32::INFINITY),
                ),
            };
            let child_size = layout_context.layout_widget(*widget, child_constraint);
            max_minor = max_minor.max(self.flex_direction.minor_axis(child_size));
            non_flex_major += self.flex_direction.major_axis(child_size);
        }

        // Layout flex children
        for (widget, flex) in flex_widgets {
            let total_major = self.flex_direction.major_axis(box_constraint.max);
            let remaining = (total_major - non_flex_major).max(0.0);
            let major = remaining * (*flex as f32 / total_flex as f32);

            let child_constraint = match self.flex_direction {
                FlexDirection::Horizontal => BoxConstraint::new(
                    Size::new(major, box_constraint.min.height),
                    Size::new(major, box_constraint.max.height),
                ),
                FlexDirection::Vertical => BoxConstraint::new(
                    Size::new(box_constraint.min.width, major),
                    Size::new(box_constraint.max.width, major),
                ),
            };

            let child_size = layout_context.layout_widget(*widget, child_constraint);
            max_minor = max_minor.max(self.flex_direction.minor_axis(child_size));
        }

        let mut current_major = 0.0;
        for child in self.children.iter().map(|item| item.widget()) {
            let size = layout_context.get_size(child);
            let minor = match self.flex_align {
                FlexAlign::Top => 0.0,
                FlexAlign::Middle => (max_minor - self.flex_direction.minor_axis(size)) / 2.0,
                FlexAlign::Baseline => max_minor - self.flex_direction.minor_axis(size),
            };

            let position = self
                .flex_direction
                .major_minor_to_position(current_major, minor);

            layout_context.set_position(child, position);

            current_major += self.flex_direction.major_axis(size);
        }

        self.flex_direction
            .major_minor_to_size(current_major, max_minor)
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

    fn update(&mut self, event: Box<dyn Any>) -> Option<Vec<WidgetId>> {
        if let Ok(event) = event.downcast::<FlexEvent>() {
            match *event {
                FlexEvent::AddChild(child) => {
                    self.children.push(child);
                }
                FlexEvent::RemoveChild => {
                    return self
                        .children
                        .pop()
                        .map(|item| match item {
                            FlexItem::NonFlex(id) => id,
                            FlexItem::Flex(id, _) => id,
                        })
                        .map(|item| vec![item]);
                }
            }
        }
        None
    }
}
