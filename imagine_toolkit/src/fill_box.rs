use imagine::{
    BoxConstraint, Geometry, InteractiveState, LayoutContext, LayoutResult, Position,
    RenderContext, RenderTreeBuilder, Size, Widget, WidgetId,
};
use webrender::api::*;

pub struct FillBox {
    pub size: Size,
    pub color: (f32, f32, f32, f32),
    hovered: bool,
    widget: Option<WidgetId>,
}

impl FillBox {
    pub fn new(size: Size, color: (f32, f32, f32, f32), widget: Option<WidgetId>) -> FillBox {
        FillBox {
            size,
            color,
            hovered: false,
            widget,
        }
    }
}

impl Widget for FillBox {
    fn create(self, builder: &mut RenderTreeBuilder) -> WidgetId {
        match self.widget {
            Some(child) => builder.create(self, &[child]),
            None => builder.create(self, &[]),
        }
    }

    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        interactive_state: InteractiveState,
        size: Option<Size>,
    ) -> LayoutResult {
        self.hovered = interactive_state.hovered;
        if let Some(size) = size {
            if let Some(widget) = self.widget {
                layout_context.set_position(widget, Position::zero());
            }
            LayoutResult::Size(box_constraint.constrain(size))
        } else {
            if let Some(widget) = self.widget {
                LayoutResult::RequestChildSize(widget, box_constraint)
            } else {
                LayoutResult::Size(box_constraint.constrain(self.size))
            }
        }
    }

    fn render(&self, geometry: Geometry, render_context: &mut RenderContext) -> Option<u64> {
        let mut info = LayoutPrimitiveInfo::new(LayoutRect::new(
            LayoutPoint::new(geometry.position.x, geometry.position.y),
            LayoutSize::new(geometry.size.width, geometry.size.height),
        ));
        let identifier = render_context.next_tag_identifier();
        info.tag = Some((identifier, 0));

        let border_radius = BorderRadius::uniform(4.0);

        let clip_id = render_context.builder.define_clip(
            info.rect,
            vec![ComplexClipRegion::new(
                info.rect,
                border_radius,
                ClipMode::Clip,
            )],
            None,
        );

        let (r, g, b, a) = self.color;

        render_context.builder.push_clip_id(clip_id);

        render_context
            .builder
            .push_rect(&info, ColorF::new(r, g, b, a));

        render_context.builder.pop_clip_id();

        if self.hovered {
            render_context.builder.push_box_shadow(
                &LayoutPrimitiveInfo::new(info.rect),
                info.rect,
                LayoutVector2D::new(0.0, 3.0),
                ColorF::new(0.0, 0.0, 0.0, 0.2),
                4.0,
                0.0,
                border_radius,
                BoxShadowClipMode::Inset,
            );
        }

        Some(identifier)
    }
}
