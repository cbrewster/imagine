use imagine::{
    BoxConstraint, Geometry, InteractiveState, LayoutContext, LayoutResult, RenderContext, Size,
    Widget, WidgetId,
};
use webrender::api::*;

pub struct FillBox {
    pub size: Size,
    pub color: (f32, f32, f32, f32),
    hovered: bool,
}

impl FillBox {
    pub fn new(size: Size, color: (f32, f32, f32, f32)) -> FillBox {
        FillBox {
            size,
            color,
            hovered: false,
        }
    }
}

impl Widget for FillBox {
    fn layout(
        &mut self,
        _layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        interactive_state: InteractiveState,
        _size: Option<Size>,
    ) -> LayoutResult {
        self.hovered = interactive_state.hovered;
        LayoutResult::Size(box_constraint.constrain(self.size))
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![]
    }

    fn render(&self, geometry: Geometry, render_context: &mut RenderContext) -> Option<u64> {
        let mut info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(geometry.position.x, geometry.position.y),
            LayoutSize::new(geometry.size.width, geometry.size.height),
        ));
        let identifier = render_context.next_tag_identifier();
        info.tag = Some((identifier, 0));

        let (r, g, b, a) = if self.hovered {
            (0.0, 0.0, 0.0, 1.0)
        } else {
            self.color
        };

        render_context
            .builder
            .push_rect(&info, ColorF::new(r, g, b, a));

        Some(identifier)
    }
}
