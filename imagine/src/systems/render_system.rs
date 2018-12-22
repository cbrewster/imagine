use crate::{
    Geometry, Interactive, Position, RenderContext, Size, WidgetComponent, WidgetId,
    WindowComponent,
};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};
use webrender::api::*;

pub(crate) struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, WidgetComponent>,
        WriteStorage<'a, WindowComponent>,
        WriteStorage<'a, Interactive>,
    );

    fn run(
        &mut self,
        (entities, sizes, positions, widgets, mut windows, mut interactive): Self::SystemData,
    ) {
        for window in (&mut windows).join() {
            if !window.dirty() {
                continue;
            }

            let mut builder = DisplayListBuilder::new(window.pipeline_id, window.layout_size());

            let bounds = LayoutRect::new(LayoutPoint::zero(), builder.content_size());

            let info = LayoutPrimitiveInfo::new(bounds);

            builder.push_stacking_context(
                &info,
                None,
                TransformStyle::Flat,
                MixBlendMode::Normal,
                &[],
                RasterSpace::Screen,
            );

            let mut render_context = RenderContext::new(&mut builder);

            fn render_entities(
                children: &[WidgetId],
                data: &(
                    &ReadStorage<Position>,
                    &ReadStorage<Size>,
                    &ReadStorage<WidgetComponent>,
                ),
                interactive: &mut WriteStorage<Interactive>,
                render_context: &mut RenderContext,
                entities: &Entities,
                offset: Position,
            ) {
                for widget_id in children {
                    let (position, size, widget) = data.join().get(widget_id.0, entities).unwrap();
                    let new_position = Position::new(offset.x + position.x, offset.y + position.y);

                    let box_size = Geometry::new(new_position, *size);

                    match widget.render(box_size, render_context) {
                        Some(tag) => {
                            interactive.insert(widget_id.0, Interactive::new(tag)).ok();
                        }
                        None => {
                            interactive.remove(widget_id.0);
                        }
                    }

                    render_entities(
                        &widget.children(),
                        data,
                        interactive,
                        render_context,
                        entities,
                        new_position,
                    );
                }
            }

            render_entities(
                &[window.root],
                &(&positions, &sizes, &widgets),
                &mut interactive,
                &mut render_context,
                &entities,
                Position::zero(),
            );

            builder.pop_stacking_context();

            window.display_list_builder = Some(builder);
        }
    }
}
