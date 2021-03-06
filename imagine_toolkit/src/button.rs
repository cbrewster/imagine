use crate::{Center, Label, Padding};
use imagine::{
    text::FinalText, BoxConstraint, Geometry, Interaction, LayoutContext, Position, RenderContext,
    Size, Widget, WidgetContext, WidgetId,
};
use webrender::api::*;
use webrender::api::units::*;

pub struct Button {
    pub color: (f32, f32, f32, f32),
    hovered: bool,
    down: bool,
    child: WidgetId,
}

impl Button {
    pub fn new<T: Into<String>, M: 'static + Send + Sync>(
        context: &mut WidgetContext<M>,
        color: (f32, f32, f32, f32),
        text: T,
    ) -> Button {
        let label = context.create_widget(Label::new(text));
        let center = context.create_widget(Center::new(label));
        let child = context.create_widget(Padding::new(10.0, 10.0, 10.0, 10.0, center));

        Button {
            color,
            down: false,
            hovered: false,
            child,
        }
    }
}

impl Widget for Button {
    fn layout(
        &self,
        _id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let child_size = layout_context.layout_widget(
            self.child,
            BoxConstraint::new(Size::zero(), box_constraint.max),
        );
        layout_context.set_position(self.child, Position::zero());
        box_constraint.constrain(child_size)
    }

    fn handle_interaction(&mut self, interaction: Interaction) {
        match interaction {
            Interaction::Hovered(hovered) => self.hovered = hovered,
            Interaction::MouseDown => self.down = true,
            Interaction::MouseUp => self.down = false,
        }
    }

    fn children(&self) -> Vec<WidgetId> {
        vec![self.child]
    }

    fn render(
        &self,
        _id: WidgetId,
        geometry: Geometry,
        _text: Option<&FinalText>,
        render_context: &mut RenderContext,
    ) -> Option<u64> {
        let mut rect = LayoutRect::new(
            LayoutPoint::new(geometry.position.x, geometry.position.y),
            LayoutSize::new(geometry.size.width, geometry.size.height),
        );

        if self.down {
            rect = rect.inflate(-2.0, -2.0);
        }

        let identifier = render_context.next_tag_identifier();

        let border_radius = BorderRadius::uniform(4.0);

        let clip_id = render_context.builder.define_clip(
            &render_context.current_space_and_clip,
            rect,
            vec![ComplexClipRegion::new(
                rect,
                border_radius,
                ClipMode::Clip,
            )],
            None,
        );

        let (r, g, b, a) = self.color;

        render_context
            .builder
            .push_rect(
                &CommonItemProperties {
                    clip_rect: rect,
                    clip_id,
                    spatial_id: render_context.current_space_and_clip.spatial_id,
                    hit_info: Some((identifier, 0)),
                    flags: PrimitiveFlags::empty(),
                },
                ColorF::new(r, g, b, a)
            );

        if self.hovered && !self.down {
            render_context.builder.push_box_shadow(
                &CommonItemProperties::new(rect.inflate(4.0, 4.0), render_context.current_space_and_clip),
                rect,
                LayoutVector2D::new(0.0, 3.0),
                ColorF::new(0.0, 0.0, 0.0, 0.2),
                4.0,
                0.0,
                border_radius,
                BoxShadowClipMode::Outset,
            );
        }

        Some(identifier)
    }
}
