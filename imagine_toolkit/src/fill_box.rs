use imagine::{BoxConstraint, Entity, Geometry, LayoutResult, SetPosition, Size, Widget};
use webrender::api::*;

pub struct FillBox {
    pub size: Size,
    pub color: (f32, f32, f32, f32),
}

impl FillBox {
    pub fn new(size: Size, color: (f32, f32, f32, f32)) -> FillBox {
        FillBox { size, color }
    }
}

impl Widget for FillBox {
    fn layout(
        &mut self,
        _set_position: SetPosition,
        box_constraint: BoxConstraint,
        _size: Option<Size>,
    ) -> LayoutResult {
        LayoutResult::Size(box_constraint.constrain(self.size))
    }

    fn children(&self) -> Vec<Entity> {
        vec![]
    }

    fn render(&self, geometry: Geometry, builder: &mut DisplayListBuilder) {
        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(geometry.position.x, geometry.position.y),
            LayoutSize::new(geometry.size.width, geometry.size.height),
        ));
        let (r, g, b, a) = self.color;
        builder.push_rect(&info, ColorF::new(r, g, b, a));
    }
}
