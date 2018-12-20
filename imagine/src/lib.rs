pub mod layout;
pub mod render;
pub mod widget;

use gleam::gl;
use glutin::GlContext;
use glutin::{EventsLoop, WindowBuilder};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use webrender::api::*;

pub use self::layout::{BoxConstraint, Geometry, LayoutContext, LayoutResult, Position, Size};
pub use self::render::RenderContext;
pub use self::widget::{InteractiveState, Widget, WidgetId};

pub struct RenderTreeBuilder {
    widgets: Vec<Box<dyn Widget + 'static>>,
    children: Vec<Vec<WidgetId>>,
}

impl RenderTreeBuilder {
    fn new() -> RenderTreeBuilder {
        RenderTreeBuilder {
            widgets: Vec::new(),
            children: Vec::new(),
        }
    }

    pub fn create<T: Widget + 'static>(&mut self, widget: T, children: &[WidgetId]) -> WidgetId {
        self.widgets.push(Box::new(widget));
        let id = self.widgets.len() - 1;
        self.children.push(children.into());
        WidgetId(id)
    }

    fn root(self, root: WidgetId) -> RenderTree {
        RenderTree {
            widgets: self.widgets,
            children: self.children,
            root,
            positions: HashMap::new(),
            sizes: HashMap::new(),
        }
    }
}

pub struct RenderTree {
    widgets: Vec<Box<dyn Widget + 'static>>,
    children: Vec<Vec<WidgetId>>,
    positions: HashMap<WidgetId, Position>,
    sizes: HashMap<WidgetId, Size>,
    root: WidgetId,
}

pub trait Ui {
    fn render(&self, builder: &mut RenderTreeBuilder) -> WidgetId;
}

pub struct Imagine<T: Ui> {
    events_loop: EventsLoop,
    window: RenderWindow,
    ui: T,
}

impl<T: Ui> Imagine<T> {
    pub fn new(ui: T, title: &str) -> Imagine<T> {
        let events_loop = EventsLoop::new();
        let pipeline_id = PipelineId(0, 0);

        let window =
            RenderWindow::new(title, &events_loop, Size::new(800.0, 600.0), pipeline_id).unwrap();

        Imagine {
            events_loop,
            window,
            ui,
        }
    }

    pub fn run(self) {
        let Imagine {
            mut events_loop,
            mut window,
            mut ui,
        } = self;

        loop {
            let mut hovered = Vec::new();
            let mut exit = false;
            events_loop.poll_events(|event| {
                if let glutin::Event::WindowEvent { event, .. } = event {
                    let response = window.handle_event(event);
                    match response {
                        EventResponse::Quit => exit = true,
                        EventResponse::Dirty => {}
                        EventResponse::Hit(hit) => hovered = hit,
                        EventResponse::Continue => {}
                    }
                }
            });
            if exit {
                break;
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

            let start = Instant::now();

            let mut tree_builder = RenderTreeBuilder::new();
            let root = ui.render(&mut tree_builder);
            let mut tree = tree_builder.root(root);

            let mut layout_context =
                LayoutContext::new(&mut tree.positions, &mut tree.sizes, &hovered);

            fn request_layout(
                layout_context: &mut LayoutContext,
                widgets: &mut Vec<Box<dyn Widget + 'static>>,
                constraint: BoxConstraint,
                widget: WidgetId,
            ) -> Size {
                let mut size_prev_child = None;
                let interactive_state = InteractiveState::new(false, false);

                loop {
                    let result = widgets[widget.0].layout(
                        layout_context,
                        constraint,
                        interactive_state,
                        size_prev_child,
                    );
                    match result {
                        LayoutResult::Size(size) => {
                            layout_context.set_size(widget, size);
                            return size;
                        }
                        LayoutResult::RequestChildSize(child, child_constraint) => {
                            size_prev_child = Some(request_layout(
                                layout_context,
                                widgets,
                                child_constraint,
                                child,
                            ));
                        }
                    }
                }
            }

            let constraint = BoxConstraint::new(
                Size::zero(),
                Size::new(layout_size.width, layout_size.height),
            );
            request_layout(
                &mut layout_context,
                &mut tree.widgets,
                constraint,
                tree.root,
            );
            layout_context.set_position(tree.root, Position::zero());

            let end = Instant::now();

            // println!("Layout took {:?}", end.duration_since(start));

            let start = Instant::now();

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

            let mut render_context = RenderContext::new(&mut builder);

            fn render_entities(
                children: &[WidgetId],
                tree: &RenderTree,
                render_context: &mut RenderContext,
                offset: Position,
            ) {
                for widget_id in children {
                    let (position, size, widget) = (
                        tree.positions[widget_id],
                        tree.sizes[widget_id],
                        &tree.widgets[widget_id.0],
                    );
                    let new_position = Position::new(offset.x + position.x, offset.y + position.y);

                    let box_size = Geometry::new(new_position, size);

                    widget.render(box_size, render_context);

                    render_entities(
                        &tree.children[widget_id.0],
                        tree,
                        render_context,
                        new_position,
                    );
                }
            }

            render_entities(&[tree.root], &tree, &mut render_context, Position::zero());

            builder.pop_stacking_context();

            txn.set_display_list(window.epoch, None, layout_size, builder.finalize(), true);
            txn.set_root_pipeline(window.pipeline_id);
            txn.generate_frame();
            window.api.send_transaction(window.document_id, txn);

            window.renderer.update();
            window.renderer.render(framebuffer_size).unwrap();
            window.window.swap_buffers().ok();

            let end = Instant::now();

            // println!("Render took {:?}", end.duration_since(start));
        }

        window.renderer.deinit();
    }
}

pub(crate) struct WindowComponent {
    root: WidgetId,
    layout_size: LayoutSize,
    dirty: bool,
    pipeline_id: PipelineId,
    pub(crate) display_list_builder: Option<DisplayListBuilder>,
    pub(crate) hovered_tags: Vec<u64>,
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

pub struct RenderWindow {
    window: glutin::GlWindow,
    renderer: webrender::Renderer,
    document_id: DocumentId,
    epoch: Epoch,
    api: RenderApi,
    pipeline_id: PipelineId,
}

impl RenderWindow {
    pub fn new(
        title: &str,
        events_loop: &EventsLoop,
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
            pipeline_id,
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
                let world_position = WorldPoint::new(position.x as f32, position.y as f32);
                let results = self.api.hit_test(
                    self.document_id,
                    Some(self.pipeline_id),
                    world_position,
                    HitTestFlags::all(),
                );
                let hit = results
                    .items
                    .iter()
                    .map(|item| item.tag.0)
                    .take(1)
                    .collect();
                EventResponse::Hit(hit)
            }
            _ => EventResponse::Continue,
        }
    }
}

enum EventResponse {
    Continue,
    Quit,
    Dirty,
    Hit(Vec<u64>),
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
