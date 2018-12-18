use crate::{BoxConstraint, Geometry, LayoutContext, LayoutResult, RenderContext, Size};
use specs::{Component, DenseVecStorage, Entity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InteractiveState {
    pub hovered: bool,
    pub clicked: bool,
}

impl InteractiveState {
    pub fn new(hovered: bool, clicked: bool) -> InteractiveState {
        InteractiveState { hovered, clicked }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct WidgetId(pub(crate) Entity);

pub trait Widget: Send + Sync {
    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        interactive_state: InteractiveState,
        size: Option<Size>,
    ) -> LayoutResult;

    fn children(&self) -> Vec<WidgetId>;

    fn render(&self, _geomtery: Geometry, _render_context: &mut RenderContext) -> Option<u64> {
        None
    }
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
