use imagine::{
    text::FinalText, BoxConstraint, Geometry, LayoutContext, RenderContext, Size, Widget, WidgetId,
};
use std::any::Any;
use webrender::api::*;

pub enum LabelMessage {
    SetText(String),
}

pub struct Label {
    text: String,
}

impl Label {
    pub fn new<T: Into<String>>(text: T) -> Label {
        Label { text: text.into() }
    }
}

impl Widget for Label {
    fn children(&self) -> Vec<WidgetId> {
        vec![]
    }

    fn layout(
        &self,
        id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size {
        let final_text = layout_context.layout_text(&self.text);
        let width = final_text.width();
        layout_context.set_text(id, final_text);
        box_constraint.constrain(Size::new(width, 32.0))
    }

    fn render(
        &self,
        _id: WidgetId,
        geometry: Geometry,
        text: Option<&FinalText>,
        render_context: &mut RenderContext,
    ) -> Option<u64> {
        let origin = LayoutPoint::new(geometry.position.x, geometry.position.y);
        let info = LayoutPrimitiveInfo::new(LayoutRect::new(
            origin,
            LayoutSize::new(geometry.size.width, geometry.size.height),
        ));

        if let Some(final_text) = text {
            final_text.render(
                &info,
                origin,
                render_context.builder,
                render_context.font_instance_key(),
            );
        }
        None
    }

    fn update(&mut self, event: Box<dyn Any>) -> Option<Vec<WidgetId>> {
        if let Ok(event) = event.downcast::<LabelMessage>() {
            match *event {
                LabelMessage::SetText(text) => self.text = text,
            }
        }
        None
    }
}
