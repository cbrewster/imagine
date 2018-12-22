use imagine::{
    text::FinalText, BoxConstraint, Geometry, LayoutContext, LayoutResult, RenderContext, Size,
    Widget, WidgetId,
};
use webrender::api::*;

pub struct Label {
    text: String,
    final_text: Option<FinalText>,
}

impl Label {
    pub fn new<T: Into<String>>(text: T) -> Label {
        Label {
            text: text.into(),
            final_text: None,
        }
    }
}

impl Widget for Label {
    fn children(&self) -> Vec<WidgetId> {
        vec![]
    }

    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        _size: Option<Size>,
    ) -> LayoutResult {
        let final_text = layout_context.layout_text(&self.text);
        let width = final_text.width();
        self.final_text = Some(final_text);
        LayoutResult::Size(box_constraint.constrain(Size::new(width, 32.0)))
    }

    fn render(&self, geometry: Geometry, render_context: &mut RenderContext) -> Option<u64> {
        let origin = LayoutPoint::new(geometry.position.x, geometry.position.y);
        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            origin,
            LayoutSize::new(geometry.size.width, geometry.size.height),
        ));

        if let Some(final_text) = &self.final_text {
            final_text.render(
                &info,
                origin,
                render_context.builder,
                render_context.font_instance_key(),
            );
        }
        None
    }
}
