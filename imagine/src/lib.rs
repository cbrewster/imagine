mod interactive;
mod layout;
mod render;
mod systems;
pub mod text;
mod widget;

use self::{
    interactive::{Event, Interactive},
    systems::{InteractionSystem, LayoutSystem, RenderSystem},
    widget::WidgetComponent,
};
use app_units::Au;
use gleam::gl;
use glutin::GlContext;
use glutin::{EventsLoop, WindowBuilder};
use rusttype::Font;
use specs::{
    Builder, Component, DenseVecStorage, Dispatcher, DispatcherBuilder, Entity, Join, World,
};
use std::collections::HashMap;
use std::mem;
use webrender::api::*;
use webrender::api::units::*;

pub use self::{
    interactive::{ClickListener, Interaction, Message, WidgetContext},
    layout::{BoxConstraint, Geometry, LayoutContext, Position, Size},
    render::RenderContext,
    widget::{Widget, WidgetId},
};

const FONT_DATA: &[u8] = include_bytes!("../resources/FreeSans.ttf");

pub trait Application {
    type Message: Message;

    fn build(&mut self, context: &mut WidgetContext<Self::Message>) -> WidgetId;

    fn handle_message(
        &mut self,
        _message: Self::Message,
        _context: &mut WidgetContext<Self::Message>,
    ) {
    }
}

pub(crate) struct MessageQueue<M: Message>(Vec<M>);

impl<M: Message> Default for MessageQueue<M> {
    fn default() -> MessageQueue<M> {
        MessageQueue(Vec::new())
    }
}

pub struct Imagine<'a, 'b, A: Application> {
    world: World,
    dispatcher: Dispatcher<'a, 'b>,
    events_loop: EventsLoop,
    windows: HashMap<glutin::WindowId, RenderWindow>,
    renderers: Vec<webrender::Renderer>,
    application: A,
}

impl<'a, 'b, A: Application> Imagine<'a, 'b, A> {
    pub fn new(application: A) -> Imagine<'a, 'b, A> {
        let mut world = World::new();
        world.add_resource(MessageQueue::<A::Message>(Vec::new()));
        let mut dispatcher = DispatcherBuilder::new()
            .with(
                InteractionSystem::<A::Message>::default(),
                "interaction",
                &[],
            )
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
            application,
        }
    }

    pub fn create_window(&mut self, title: &str, size: Size) {
        // TODO: Generate Unique PipelineIds per Window.
        let pipeline_id = PipelineId(0, 0);
        let window_entity = self.world.create_entity().build();
        let render_window =
            RenderWindow::new(title, &self.events_loop, window_entity, size, pipeline_id).unwrap();
        let mut windows = self.world.write_storage::<WindowComponent>();
        let entities = self.world.entities();
        let mut widgets = self.world.write_storage::<WidgetComponent>();
        let mut click_listeners = self.world.write_storage::<ClickListener<A::Message>>();
        let mut context = WidgetContext::new(&entities, &mut widgets, &mut click_listeners);
        let root = self.application.build(&mut context);
        windows
            .insert(
                window_entity,
                WindowComponent {
                    root,
                    layout_size: LayoutSize::zero(),
                    dirty: true,
                    pipeline_id,
                    display_list_builder: None,
                    hovered: None,
                    clicked: None,
                    font_instance_key: render_window.font_instance_key,
                    font: Font::from_bytes(FONT_DATA).unwrap(),
                },
            )
            .ok();
        self.windows
            .insert(render_window.window.id(), render_window);
    }

    pub fn run(self) {
        let Imagine {
            mut events_loop,
            mut dispatcher,
            mut windows,
            mut world,
            mut renderers,
            mut application,
        } = self;

        events_loop.run_forever(|event| {
            if let glutin::Event::WindowEvent { event, window_id } = event {
                let mut response = EventResponse::Continue;
                if let Some(window) = windows.get_mut(&window_id) {
                    response = window.handle_event::<A::Message>(event, &world);
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
                    framebuffer_size.to_f32() / euclid::Scale::new(hidpi_factor as f32);

                window_component.layout_size = layout_size;
            }

            dispatcher.dispatch(&world.res);
            world.maintain();

            let entities = world.entities();
            let mut widgets = world.write_storage::<WidgetComponent>();
            let mut click_listeners = world.write_storage::<ClickListener<A::Message>>();
            let mut context = WidgetContext::new(&entities, &mut widgets, &mut click_listeners);

            let mut message_queue = world.write_resource::<MessageQueue<A::Message>>();
            let messages = mem::replace(&mut *message_queue, MessageQueue::default());

            for message in messages.0 {
                application.handle_message(message, &mut context);
            }

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
    pub(crate) font_instance_key: FontInstanceKey,
    pub(crate) font: Font<'static>,
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
    font_instance_key: FontInstanceKey,
    show_profiler: bool,
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
            debug_flags: webrender::DebugFlags::empty(),
            ..webrender::RendererOptions::default()
        };

        let framebuffer_size = {
            let size = window.get_inner_size().unwrap().to_physical(hidpi_factor);
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };
        let notifier = Box::new(Notifier::new(events_loop.create_proxy()));
        let (renderer, sender) =
            webrender::Renderer::new(gl.clone(), notifier, opts, None, framebuffer_size).unwrap();
        let api = sender.create_api();
        let document_id = api.add_document(framebuffer_size, 0);
        let epoch = Epoch(0);

        let mut txn = Transaction::new();

        let font_key = api.generate_font_key();
        txn.add_raw_font(font_key, Vec::from(FONT_DATA), 0);

        let font_instance_key = api.generate_font_instance_key();
        txn.add_font_instance(
            font_instance_key,
            font_key,
            Au::from_px(32),
            None,
            None,
            Vec::new(),
        );

        api.send_transaction(document_id, txn);

        Ok(RenderWindow {
            window,
            renderer,
            api,
            epoch,
            document_id,
            entity,
            pipeline_id,
            font_instance_key,
            show_profiler: false,
        })
    }

    fn handle_event<T: 'static + Send + Sync>(
        &mut self,
        event: glutin::WindowEvent,
        world: &World,
    ) -> EventResponse {
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

                self.api.set_document_view(
                    self.document_id,
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
                if !self.show_profiler {
                    self.renderer
                        .set_debug_flags(webrender::DebugFlags::PROFILER_DBG);
                } else {
                    self.renderer
                        .set_debug_flags(webrender::DebugFlags::empty());
                }
                self.show_profiler = !self.show_profiler;
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
                            .filter(|(e, _)| entities.is_alive(*e))
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
    fn clone(&self) -> Box<dyn RenderNotifier> {
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
