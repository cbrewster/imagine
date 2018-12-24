use crate::{Center, Label, Padding};
use imagine::{
    BoxConstraint, Geometry, Imagine, Interaction, LayoutContext, LayoutResult, Position,
    RenderContext, Size, Widget, WidgetBuilder, WidgetId,
};
use webrender::api::*;

pub struct Button {
    pub color: (f32, f32, f32, f32),
    hovered: bool,
    down: bool,
    child: WidgetId,
}

impl Button {
    pub fn new<T: Into<String>>(
        imagine: &mut Imagine,
        color: (f32, f32, f32, f32),
        text: T,
    ) -> Button {
        let label = imagine.create_widget(Label::new(text));
        let center = imagine.create_widget(Center::new(label));
        let child = imagine.create_widget(Padding::new(10.0, 10.0, 10.0, 10.0, center));

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
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult {
        match size {
            None => {
                layout_context.set_position(self.child, Position::zero());
                LayoutResult::RequestChildSize(
                    self.child,
                    BoxConstraint::new(Size::zero(), box_constraint.max),
                )
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
        vec![self.child]
    }

    fn render(&self, geometry: Geometry, render_context: &mut RenderContext) -> Option<u64> {
        let mut rect = LayoutRect::new(
            LayoutPoint::new(geometry.position.x, geometry.position.y),
            LayoutSize::new(geometry.size.width, geometry.size.height),
        );

        if self.down {
            rect = rect.inflate(-2.0, -2.0);
        }

        let mut info = LayoutPrimitiveInfo::new(rect);
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

        if self.hovered && !self.down {
            render_context.builder.push_box_shadow(
                &LayoutPrimitiveInfo::new(info.rect.inflate(4.0, 4.0)),
                info.rect,
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

#[derive(Default)]
pub struct ButtonBuilder {
    text: Option<String>,
    color: Option<(f32, f32, f32, f32)>,
}

impl ButtonBuilder {
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn with_color(mut self, color: (f32, f32, f32, f32)) -> Self {
        self.color = Some(color);
        self
    }
}

impl WidgetBuilder for ButtonBuilder {
    fn build(self, imagine: &mut Imagine) -> WidgetId {
        let color = self.color.unwrap_or((1.0, 1.0, 1.0, 1.0));
        let text = self.text.unwrap_or_else(|| "".into());

        let button = Button::new(imagine, color, text);
        imagine.create_widget(button)
    }
}
