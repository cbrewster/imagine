use imagine::{
    text::FinalText, BoxConstraint, Geometry, Interaction, LayoutContext, RenderContext, Size,
    Widget, WidgetId,
};
use webrender::api::*;

pub struct FillBox {
    pub size: Size,
    pub color: (f32, f32, f32, f32),
    hovered: bool,
    down: bool,
}

impl FillBox {
    pub fn new(size: Size, color: (f32, f32, f32, f32)) -> FillBox {
        FillBox {
            size,
            color,
            down: false,
            hovered: false,
        }
    }
}

impl Widget for FillBox {
    fn layout(
        &self,
        _id: WidgetId,
        _layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        box_constraint.constrain(self.size)
    }

    fn handle_interaction(&mut self, interaction: Interaction) {
        match interaction {
            Interaction::Hovered(hovered) => self.hovered = hovered,
            Interaction::MouseDown => self.down = true,
            Interaction::MouseUp => self.down = false,
        }
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![]
    }

    fn render(
        &self,
        _id: WidgetId,
        geometry: Geometry,
        _text: Option<&FinalText>,
        render_context: &mut RenderContext,
    ) -> Option<u64> {
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

        let (r, g, b, mut a) = self.color;

        if self.down {
            a = 0.5;
        }

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
