use crate::{Label, Padding};
use imagine::{
    BoxConstraint, Geometry, Imagine, Interaction, LayoutContext, LayoutResult, Position,
    RenderContext, Size, Widget, WidgetId,
};
use webrender::api::*;

pub struct Button {
    pub color: (f32, f32, f32, f32),
    hovered: bool,
    down: bool,
    padding: WidgetId,
}

impl Button {
    pub fn new(imagine: &mut Imagine, color: (f32, f32, f32, f32), text: &str) -> Button {
        let label = imagine.create_widget(Label::new(text));
        let padding = imagine.create_widget(Padding::new(10.0, 10.0, 10.0, 10.0, label));

        Button {
            color,
            down: false,
            hovered: false,
            padding,
        }
    }
}

impl Widget for Button {
    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => {
                layout_context.set_position(self.padding, Position::zero());
                LayoutResult::RequestChildSize(self.padding, box_constraint)
            }
            Some(size) => LayoutResult::Size(box_constraint.constrain(size)),
        }
    }

    fn handle_interaction(&mut self, interaction: Interaction) {
        match interaction {
            Interaction::Hovered(hovered) => self.hovered = hovered,
            Interaction::MouseDown => self.down = true,
            Interaction::MouseUp => self.down = false,
        }
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.padding]
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
