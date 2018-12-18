pub mod layout;
pub mod systems;
pub mod widget;

use self::{systems::LayoutSystem, widget::WidgetComponent};
use gleam::gl;
use glutin::GlContext;
use glutin::{EventsLoop, WindowBuilder};
use specs::{Builder, Component, DenseVecStorage, Dispatcher, DispatcherBuilder, World};
use std::collections::HashMap;
use webrender::api::*;

pub use self::layout::{BoxConstraint, Geometry, LayoutResult, Position, SetPosition, Size};
pub use self::widget::Widget;
pub use specs::Entity;

pub struct Imagine<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    events_loop: EventsLoop,
    windows: HashMap<glutin::WindowId, RenderWindow>,
    renderers: Vec<webrender::Renderer>,
}

impl<'a, 'b> Imagine<'a, 'b> {
    pub fn new() -> Imagine<'a, 'b> {
        let mut world = World::new();
        world.register::<WidgetComponent>();
        world.register::<Position>();
        world.register::<Size>();
        let mut dispatcher = DispatcherBuilder::new()
            .with(LayoutSystem, "layout", &[])
            .build();

        dispatcher.setup(&mut world.res);

        let events_loop = EventsLoop::new();

        Imagine {
            world,
            dispatcher,
            events_loop,
            windows: HashMap::new(),
            renderers: Vec::new(),
        }
    }

    pub fn create_window(&mut self, title: &str, root: Entity, size: Size) {
        let window_entity = self
            .world
            .create_entity()
            .with(WindowComponent {
                root: root,
                layout_size: size,
                dirty: true,
            })
            .build();
        let render_window =
            RenderWindow::new(title, &self.events_loop, window_entity, size).unwrap();
        self.windows
            .insert(render_window.window.id(), render_window);
    }

    pub fn add_widget<W: Widget + 'static>(&mut self, widget: W) -> Entity {
        self.world
            .create_entity()
            .with(WidgetComponent {
                inner: Box::new(widget),
            })
            .build()
    }

    pub fn run(self) {
        let Imagine {
            mut events_loop,
            mut dispatcher,
            mut windows,
            world,
            mut renderers,
        } = self;
        while !windows.is_empty() {
            events_loop.poll_events(|event| {
                if let glutin::Event::WindowEvent { event, window_id } = event {
                    let mut response = EventResponse::Continue;
                    if let Some(window) = windows.get_mut(&window_id) {
                        response = window.handle_event(event);
                    }
                    match response {
                        EventResponse::Quit => {
                            if let Some(window) = windows.remove(&window_id) {
                                renderers.push(window.renderer);
                            }
                        }
                        EventResponse::Dirty => {
                            if let Some(window) = windows.get(&window_id) {
                                let mut window_components =
                                    world.write_storage::<WindowComponent>();

                                let window_component = window_components
                                    .get_mut(window.entity)
                                    .expect("Could not find window component");
                                window_component.set_dirty(true);
                            }
                        }
                        EventResponse::Continue => {}
                    }
                }
            });

            for window in windows.values() {
                let mut window_components = world.write_storage::<WindowComponent>();
                let window_component = window_components
                    .get_mut(window.entity)
                    .expect("Could not find window component");

                if !window_component.dirty() {
                    continue;
                }

                let window_size = window.window.get_inner_size().unwrap();
                window_component.layout_size =
                    Size::new(window_size.width as f32, window_size.height as f32);
            }

            dispatcher.dispatch(&world.res);

            let mut window_components = world.write_storage::<WindowComponent>();

            for window in windows.values_mut() {
                let window_component = window_components
                    .get_mut(window.entity)
                    .expect("Could not find window component");

                unsafe {
                    window.window.make_current().ok();
                }

                let hidpi_factor = window.window.get_hidpi_factor();
                let framebuffer_size = {
                    let size = window
                        .window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(hidpi_factor);
                    DeviceIntSize::new(size.width as i32, size.height as i32)
                };

                if window_component.dirty() {
                    window_component.set_dirty(false);
                    let layout_size =
                        framebuffer_size.to_f32() / euclid::TypedScale::new(hidpi_factor as f32);

                    let mut txn = Transaction::new();
                    let mut builder = DisplayListBuilder::new(window.pipeline_id, layout_size);

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

                    fn render_entities(
                        entities: &[Entity],
                        world: &World,
                        builder: &mut DisplayListBuilder,
                        offset: Position,
                    ) {
                        let positions = world.read_storage::<Position>();
                        let sizes = world.read_storage::<Size>();
                        let widget_components = world.read_storage::<WidgetComponent>();

                        for entity in entities {
                            let position = positions.get(*entity).unwrap();
                            let size = sizes.get(*entity).unwrap();

                            let new_position =
                                Position::new(offset.x + position.x, offset.y + position.y);

                            let box_size = Geometry::new(new_position, *size);

                            let widget = widget_components.get(*entity).unwrap();
                            widget.render(box_size, builder);
                            render_entities(&widget.children(), world, builder, new_position);
                        }
                    }

                    render_entities(
                        &[window_component.root],
                        &world,
                        &mut builder,
                        Position::zero(),
                    );

                    builder.pop_stacking_context();

                    txn.set_display_list(window.epoch, None, layout_size, builder.finalize(), true);
                    txn.set_root_pipeline(window.pipeline_id);
                    txn.generate_frame();
                    window.api.send_transaction(window.document_id, txn);
                }

                window.renderer.update();
                window.renderer.render(framebuffer_size).unwrap();
                window.window.swap_buffers().ok();
            }
        }

        for renderer in renderers {
            renderer.deinit();
        }
    }
}

pub struct WindowComponent {
    root: Entity,
    layout_size: Size,
    dirty: bool,
}

impl WindowComponent {
    pub fn layout_size(&self) -> Size {
        self.layout_size
    }

    pub fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn set_dirty(&mut self, dirty: bool) {
        self.dirty = dirty
    }
}

impl Component for WindowComponent {
    type Storage = DenseVecStorage<Self>;
}

pub struct RenderWindow {
    window: glutin::GlWindow,
    renderer: webrender::Renderer,
    pipeline_id: PipelineId,
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    entity: Entity,
}

impl RenderWindow {
    pub fn new(
        title: &str,
        events_loop: &EventsLoop,
        entity: Entity,
        size: Size,
    ) -> Result<RenderWindow, glutin::CreationError> {
        let window_builder =
            WindowBuilder::new()
                .with_title(title)
                .with_dimensions(glutin::dpi::LogicalSize::new(
                    size.width as f64,
                    size.height as f64,
                ));
        let context = glutin::ContextBuilder::new();
        let window = glutin::GlWindow::new(window_builder, context, events_loop)?;

        unsafe {
            window.make_current().ok();
        }

        let gl = match window.get_api() {
            glutin::Api::OpenGl => unsafe {
                gl::GlFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::OpenGlEs => unsafe {
                gl::GlesFns::load_with(|symbol| window.get_proc_address(symbol) as *const _)
            },
            glutin::Api::WebGl => unimplemented!(),
        };

        let hidpi_factor = window.get_hidpi_factor();

        let opts = webrender::RendererOptions {
            device_pixel_ratio: hidpi_factor as f32,
            clear_color: Some(ColorF::new(0.98, 0.98, 0.98, 1.0)),
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let size = window.get_inner_size().unwrap().to_physical(hidpi_factor);
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };
        let notifier = Box::new(Notifier::new(events_loop.create_proxy()));
        let (renderer, sender) =
            webrender::Renderer::new(gl.clone(), notifier, opts, None).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        Ok(RenderWindow {
            window,
            renderer,
            api,
            pipeline_id,
            epoch,
            document_id,
            entity,
        })
    }

    fn handle_event(&mut self, event: glutin::WindowEvent) -> EventResponse {
        match event {
            glutin::WindowEvent::CloseRequested
            | glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        virtual_keycode: Some(glutin::VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => EventResponse::Quit,
            glutin::WindowEvent::Resized(size) => {
                let dpi_factor = self.window.get_hidpi_factor();
                self.window.resize(size.to_physical(dpi_factor));
                EventResponse::Dirty
            }
            _ => EventResponse::Continue,
        }
    }
}

enum EventResponse {
    Continue,
    Quit,
    Dirty,
}

struct Notifier {
    events_proxy: glutin::EventsLoopProxy,
}

impl Notifier {
    fn new(events_proxy: glutin::EventsLoopProxy) -> Notifier {
        Notifier { events_proxy }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<RenderNotifier> {
        Box::new(Notifier {
            events_proxy: self.events_proxy.clone(),
        })
    }

    fn wake_up(&self) {
        let _ = self.events_proxy.wakeup();
    }

    fn new_frame_ready(
        &self,
        _: DocumentId,
        _scrolled: bool,
        _composite_needed: bool,
        _render_time: Option<u64>,
    ) {
        self.wake_up();
    }
}
