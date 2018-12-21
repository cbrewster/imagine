pub mod layout;
pub mod render;
pub mod systems;
pub mod widget;

use self::{
    render::{Event, Interactive},
    systems::{InteractionSystem, LayoutSystem, RenderSystem},
    widget::WidgetComponent,
};
use gleam::gl;
use glutin::GlContext;
use glutin::{EventsLoop, WindowBuilder};
use specs::{
    Builder, Component, DenseVecStorage, Dispatcher, DispatcherBuilder, Entity, Join, World,
};
use std::collections::HashMap;
use webrender::api::*;

pub use self::layout::{BoxConstraint, Geometry, LayoutContext, LayoutResult, Position, Size};
pub use self::render::{ClickListener, InteractionContext, RenderContext};
pub use self::widget::{Interaction, Widget, WidgetId};

pub struct Imagine<'a, 'b> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    events_loop: EventsLoop,
    windows: HashMap<glutin::WindowId, RenderWindow>,
    renderers: Vec<webrender::Renderer>,
}

impl<'a, 'b> Default for Imagine<'a, 'b> {
    fn default() -> Imagine<'a, 'b> {
        let mut world = World::new();
        world.register::<WidgetComponent>();
        world.register::<Position>();
        world.register::<Size>();
        world.register::<Interactive>();

        let mut dispatcher = DispatcherBuilder::new()
            .with(InteractionSystem, "interaction", &[])
            .with(LayoutSystem, "layout", &["interaction"])
            .with(RenderSystem, "render", &["interaction", "layout"])
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
}

impl<'a, 'b> Imagine<'a, 'b> {
    pub fn create_window(&mut self, title: &str, root: WidgetId, size: Size) {
        // TODO: Generate Unique PipelineIds per Window.
        let pipeline_id = PipelineId(0, 0);
        let window_entity = self
            .world
            .create_entity()
            .with(WindowComponent {
                root,
                layout_size: LayoutSize::zero(),
                dirty: true,
                pipeline_id,
                display_list_builder: None,
                hovered: None,
                clicked: None,
            })
            .build();
        let render_window =
            RenderWindow::new(title, &self.events_loop, window_entity, size, pipeline_id).unwrap();
        self.windows
            .insert(render_window.window.id(), render_window);
    }

    pub fn create_widget<W: Widget + 'static>(&mut self, widget: W) -> WidgetId {
        WidgetId(
            self.world
                .create_entity()
                .with(WidgetComponent {
                    inner: Box::new(widget),
                })
                .build(),
        )
    }

    pub fn add_click_listener(&mut self, widget_id: WidgetId, listener: ClickListener) {
        let mut listeners = self.world.write_storage::<ClickListener>();
        listeners.insert(widget_id.0, listener).ok();
    }

    pub fn run(self) {
        let Imagine {
            mut events_loop,
            mut dispatcher,
            mut windows,
            world,
            mut renderers,
        } = self;

        events_loop.run_forever(|event| {
            if let glutin::Event::WindowEvent { event, window_id } = event {
                let mut response = EventResponse::Continue;
                if let Some(window) = windows.get_mut(&window_id) {
                    response = window.handle_event(event, &world);
                }
                match response {
                    EventResponse::Quit => {
                        if let Some(window) = windows.remove(&window_id) {
                            renderers.push(window.renderer);
                        }
                    }
                    EventResponse::Dirty => {
                        if let Some(window) = windows.get(&window_id) {
                            let mut window_components = world.write_storage::<WindowComponent>();

                            let window_component = window_components
                                .get_mut(window.entity)
                                .expect("Could not find window component");
                            window_component.set_dirty(true);
                        }
                    }
                    EventResponse::Continue => {}
                }
            }

            for window in windows.values() {
                let mut window_components = world.write_storage::<WindowComponent>();
                let window_component = window_components
                    .get_mut(window.entity)
                    .expect("Could not find window component");

                if !window_component.dirty() {
                    continue;
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

                let layout_size: LayoutSize =
                    framebuffer_size.to_f32() / euclid::TypedScale::new(hidpi_factor as f32);

                window_component.layout_size = layout_size;
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

                    if let Some(builder) = window_component.display_list_builder.take() {
                        let mut txn = Transaction::new();

                        txn.set_display_list(
                            window.epoch,
                            None,
                            window_component.layout_size,
                            builder.finalize(),
                            true,
                        );
                        txn.set_root_pipeline(window_component.pipeline_id);
                        txn.generate_frame();
                        window.api.send_transaction(window.document_id, txn);
                    }
                }

                window.renderer.update();
                window.renderer.render(framebuffer_size).unwrap();
                window.window.swap_buffers().ok();
            }

            if windows.is_empty() {
                glutin::ControlFlow::Break
            } else {
                glutin::ControlFlow::Continue
            }
        });

        for renderer in renderers {
            renderer.deinit();
        }
    }
}

pub(crate) struct WindowComponent {
    root: WidgetId,
    layout_size: LayoutSize,
    dirty: bool,
    pipeline_id: PipelineId,
    hovered: Option<Entity>,
    clicked: Option<Entity>,
    pub(crate) display_list_builder: Option<DisplayListBuilder>,
}

impl WindowComponent {
    pub fn layout_size(&self) -> LayoutSize {
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
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    entity: Entity,
    pipeline_id: PipelineId,
}

impl RenderWindow {
    pub fn new(
        title: &str,
        events_loop: &EventsLoop,
        entity: Entity,
        size: Size,
        pipeline_id: PipelineId,
    ) -> Result<RenderWindow, glutin::CreationError> {
        let window_builder = WindowBuilder::new()
            .with_title(title)
            .with_dimensions((f64::from(size.width), f64::from(size.height)).into());
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
            debug_flags: webrender::DebugFlags::PROFILER_DBG,
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

        Ok(RenderWindow {
            window,
            renderer,
            api,
            epoch,
            document_id,
            entity,
            pipeline_id,
        })
    }

    fn handle_event(&mut self, event: glutin::WindowEvent, world: &World) -> EventResponse {
        let mut window_components = world.write_storage::<WindowComponent>();
        let mut window_component = window_components.get_mut(self.entity).unwrap();

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
                let hidpi_factor = self.window.get_hidpi_factor();
                self.window.resize(size.to_physical(hidpi_factor));
                let framebuffer_size = {
                    let size = self
                        .window
                        .get_inner_size()
                        .unwrap()
                        .to_physical(hidpi_factor);
                    DeviceIntSize::new(size.width as i32, size.height as i32)
                };

                self.api.set_window_parameters(
                    self.document_id,
                    framebuffer_size,
                    DeviceIntRect::new(DeviceIntPoint::zero(), framebuffer_size),
                    hidpi_factor as f32,
                );
                EventResponse::Dirty
            }
            glutin::WindowEvent::KeyboardInput {
                input:
                    glutin::KeyboardInput {
                        state: glutin::ElementState::Pressed,
                        virtual_keycode: Some(glutin::VirtualKeyCode::P),
                        ..
                    },
                ..
            } => {
                self.renderer
                    .toggle_debug_flags(webrender::DebugFlags::PROFILER_DBG);
                EventResponse::Continue
            }
            glutin::WindowEvent::CursorMoved { position, .. } => {
                if window_component.clicked.is_some() {
                    return EventResponse::Continue;
                }
                let world_position = WorldPoint::new(position.x as f32, position.y as f32);
                let results = self.api.hit_test(
                    self.document_id,
                    Some(self.pipeline_id),
                    world_position,
                    HitTestFlags::empty(),
                );
                let interactive = world.read_storage::<Interactive>();
                let entities = world.entities();
                let mut events = world.write_storage::<Event>();
                let hit = results
                    .items
                    .iter()
                    .map(|item| item.tag.0)
                    .next()
                    .and_then(|id| {
                        (&entities, &interactive)
                            .join()
                            .find(|(_, i)| i.tag == id)
                            .map(|(e, _)| e)
                    });

                let changed = hit != window_component.hovered;

                if changed {
                    match (hit, window_component.hovered) {
                        (Some(new), Some(old)) => {
                            events
                                .insert(old, Event::new(Interaction::Hovered(false)))
                                .ok();
                            events
                                .insert(new, Event::new(Interaction::Hovered(true)))
                                .ok();
                        }
                        (Some(new), None) => {
                            events
                                .insert(new, Event::new(Interaction::Hovered(true)))
                                .ok();
                        }
                        (None, Some(old)) => {
                            events
                                .insert(old, Event::new(Interaction::Hovered(false)))
                                .ok();
                        }
                        _ => {}
                    }
                }

                window_component.hovered = hit;

                if changed {
                    EventResponse::Dirty
                } else {
                    EventResponse::Continue
                }
            }
            glutin::WindowEvent::MouseInput {
                button: glutin::MouseButton::Left,
                state,
                ..
            } => {
                match state {
                    glutin::ElementState::Pressed => {
                        if let Some(entity) = window_component.hovered {
                            let mut events = world.write_storage::<Event>();
                            events
                                .insert(entity, Event::new(Interaction::MouseDown))
                                .ok();
                            window_component.clicked = Some(entity);
                        }
                    }
                    glutin::ElementState::Released => {
                        if let Some(entity) = window_component.clicked.take() {
                            let mut events = world.write_storage::<Event>();
                            events.insert(entity, Event::new(Interaction::MouseUp)).ok();
                        }
                    }
                }

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
