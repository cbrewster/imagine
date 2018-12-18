use crate::{BoxConstraint, Entity, Geometry, LayoutResult, SetPosition, Size};
use specs::{Component, DenseVecStorage};
use webrender::api::DisplayListBuilder;

pub trait Widget: Send + Sync {
    fn layout(
        &mut self,
        set_position: SetPosition,
        box_constraint: BoxConstraint,
        size: Option<Size>,
    ) -> LayoutResult;

    fn children(&self) -> Vec<Entity>;

    fn render(&self, _geomtery: Geometry, _builder: &mut DisplayListBuilder) {}
}

pub(crate) struct WidgetComponent {
    pub(crate) inner: Box<dyn Widget>,
}

impl std::ops::Deref for WidgetComponent {
    type Target = Widget;

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
