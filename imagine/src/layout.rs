use crate::WidgetId;
use specs::{Component, DenseVecStorage, WriteStorage};

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

impl Component for Position {
    type Storage = DenseVecStorage<Self>;
}

impl Component for Size {
    type Storage = DenseVecStorage<Self>;
}

pub struct LayoutContext<'a, 'b> {
    positions: &'a mut WriteStorage<'b, Position>,
    sizes: &'a mut WriteStorage<'b, Size>,
    hovered_tags: &'a [u64],
}

impl<'a, 'b> LayoutContext<'a, 'b> {
    pub(crate) fn new(
        positions: &'a mut WriteStorage<'b, Position>,
        sizes: &'a mut WriteStorage<'b, Size>,
        hovered_tags: &'a [u64],
    ) -> LayoutContext<'a, 'b> {
        LayoutContext {
            positions,
            sizes,
            hovered_tags,
        }
    }

    pub fn set_position(&mut self, widget: WidgetId, position: Position) {
        self.positions.insert(widget.0, position).ok();
    }

    pub fn get_size(&mut self, widget: WidgetId) -> Size {
        *self.sizes.get(widget.0).unwrap()
    }

    pub fn is_hovered(&self, tag: u64) -> bool {
        self.hovered_tags.contains(&tag)
    }
}
