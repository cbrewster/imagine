use crate::{
    interactive::Interaction, text::FinalText, BoxConstraint, Geometry, LayoutContext, Message,
    RenderContext, Size, WidgetContext,
};
use specs::{Component, DenseVecStorage, Entity};
use std::any::Any;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WidgetId(pub(crate) Entity);

pub trait WidgetBuilder {
    fn build<T: Message>(self, context: &mut WidgetContext<T>) -> WidgetId;
}

pub trait Widget: Send + Sync {
    fn layout(
        &self,
        id: WidgetId,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
    ) -> Size;

    fn children(&self) -> Vec<WidgetId>;

    fn render(
        &self,
        _id: WidgetId,
        _geometry: Geometry,
        _text: Option<&FinalText>,
        _render_context: &mut RenderContext,
    ) -> Option<u64> {
        None
    }

    fn handle_interaction(&mut self, _interaction: Interaction) {}

    fn update(&mut self, _event: Box<dyn Any>) -> Option<Vec<WidgetId>> {
        None
    }
}

pub(crate) struct WidgetComponent {
    pub(crate) inner: Box<dyn Widget>,
}

impl std::ops::Deref for WidgetComponent {
    type Target = dyn Widget;

    fn deref(&self) -> &(dyn Widget + 'static) {
        self.inner.deref()
    }
}

impl std::ops::DerefMut for WidgetComponent {
    fn deref_mut(&mut self) -> &mut (dyn Widget + 'static) {
        self.inner.deref_mut()
    }
}

impl Component for WidgetComponent {
    type Storage = DenseVecStorage<Self>;
}
