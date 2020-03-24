use crate::{
    text::FinalText, Geometry, Interactive, Position, RenderContext, Size, WidgetComponent,
    WidgetId, WindowComponent,
};
use specs::{Entities, Join, ReadStorage, System, WriteStorage};
use webrender::api::*;
use webrender::api::units::*;

pub(crate) struct RenderSystem;

impl<'a> System<'a> for RenderSystem {
    type SystemData = (
        Entities<'a>,
        ReadStorage<'a, Size>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, WidgetComponent>,
        ReadStorage<'a, FinalText>,
        WriteStorage<'a, WindowComponent>,
        WriteStorage<'a, Interactive>,
    );

    fn run(
        &mut self,
        (entities, sizes, positions, widgets, text, mut windows, mut interactive): Self::SystemData,
    ) {
        for window in (&mut windows).join() {
            if !window.dirty() {
                continue;
            }

            let mut builder = DisplayListBuilder::new(window.pipeline_id, window.layout_size());

            builder.push_stacking_context(
                LayoutPoint::zero(),
                SpatialId::root_reference_frame(window.pipeline_id),
                PrimitiveFlags::empty(),
                None,
                TransformStyle::Flat,
                MixBlendMode::Normal,
                &[],
                &[],
                &[],
                RasterSpace::Screen,
                false,
                false,
            );

            let mut render_context = RenderContext::new(&mut builder, window.font_instance_key, window.pipeline_id);

            fn render_entities(
                children: &[WidgetId],
                data: &(
                    &ReadStorage<Position>,
                    &ReadStorage<Size>,
                    &ReadStorage<WidgetComponent>,
                ),
                texts: &ReadStorage<FinalText>,
                interactive: &mut WriteStorage<Interactive>,
                render_context: &mut RenderContext,
                entities: &Entities,
                offset: Position,
            ) {
                for widget_id in children {
                    let (position, size, widget) = data.join().get(widget_id.0, entities).unwrap();
                    let new_position = Position::new(offset.x + position.x, offset.y + position.y);
                    let text = texts.get(widget_id.0);
                    let box_size = Geometry::new(new_position, *size);

                    match widget.render(*widget_id, box_size, text, render_context) {
                        Some(tag) => {
                            if let Some(interactive) = interactive.get_mut(widget_id.0) {
                                interactive.tag = tag;
                            } else {
                                interactive.insert(widget_id.0, Interactive::new(tag)).ok();
                            }
                        }
                        None => {
                            interactive.remove(widget_id.0);
                        }
                    }

                    render_entities(
                        &widget.children(),
                        data,
                        texts,
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
                &text,
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
