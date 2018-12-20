use crate::{
    BoxConstraint, Geometry, LayoutContext, LayoutResult, RenderContext, RenderTreeBuilder, Size,
};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct WidgetId(pub(crate) usize);

pub trait Widget: Send + Sync {
    fn layout(
        &mut self,
        layout_context: &mut LayoutContext,
        box_constraint: BoxConstraint,
        interactive_state: InteractiveState,
        size: Option<Size>,
    ) -> LayoutResult;

    fn render(&self, _geomtery: Geometry, _render_context: &mut RenderContext) -> Option<u64> {
        None
    }

    fn create(self, builder: &mut RenderTreeBuilder) -> WidgetId;
}
