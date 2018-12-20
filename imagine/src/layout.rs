use crate::WidgetId;
use std::collections::HashMap;

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Geometry {
    pub position: Position,
    pub size: Size,
}

impl Geometry {
    pub fn new(position: Position, size: Size) -> Geometry {
        Geometry { position, size }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    pub fn new(x: f32, y: f32) -> Position {
        Position { x, y }
    }

    pub fn zero() -> Position {
        Position { x: 0.0, y: 0.0 }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub fn new(width: f32, height: f32) -> Size {
        Size { width, height }
    }

    pub fn zero() -> Size {
        Size {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BoxConstraint {
    pub min: Size,
    pub max: Size,
}

impl BoxConstraint {
    pub fn new(min: Size, max: Size) -> BoxConstraint {
        BoxConstraint { min, max }
    }

    pub fn tight(size: Size) -> BoxConstraint {
        BoxConstraint {
            min: size,
            max: size,
        }
    }

    pub fn constrain(&self, size: Size) -> Size {
        Size::new(
            clamp(size.width, self.min.width, self.max.width),
            clamp(size.height, self.min.height, self.max.height),
        )
    }

    pub fn is_tight(&self) -> bool {
        (self.min.width - self.max.width).abs() < std::f32::EPSILON
            && (self.min.height - self.max.height).abs() < std::f32::EPSILON
    }
}

fn clamp(input: f32, min: f32, max: f32) -> f32 {
    if input < min {
        min
    } else if input > max {
        max
    } else {
        input
    }
}

pub enum LayoutResult {
    Size(Size),
    RequestChildSize(WidgetId, BoxConstraint),
}

pub struct LayoutContext<'a> {
    positions: &'a mut HashMap<WidgetId, Position>,
    sizes: &'a mut HashMap<WidgetId, Size>,
    hovered_tags: &'a Vec<u64>,
}

impl<'a> LayoutContext<'a> {
    pub(crate) fn new(
        positions: &'a mut HashMap<WidgetId, Position>,
        sizes: &'a mut HashMap<WidgetId, Size>,
        hovered_tags: &'a Vec<u64>,
    ) -> LayoutContext<'a> {
        LayoutContext {
            positions,
            sizes,
            hovered_tags,
        }
    }

    pub(crate) fn set_size(&mut self, widget: WidgetId, size: Size) {
        self.sizes.insert(widget, size);
    }

    pub fn set_position(&mut self, widget: WidgetId, position: Position) {
        self.positions.insert(widget, position);
    }

    pub fn get_size(&mut self, widget: WidgetId) -> Size {
        *self.sizes.get(&widget).unwrap()
    }

    pub fn is_hovered(&self, tag: u64) -> bool {
        self.hovered_tags.contains(&tag)
    }
}
