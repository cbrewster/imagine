use imagine::{Application, ClickListener, Imagine, Size, WidgetContext, WidgetId};
use imagine_toolkit::{
    Button, FillBox, Flex, FlexAlign, FlexDirection, FlexEvent, FlexItem, Label, Padding,
};

enum BasicMessage {
    Add,
    Remove,
}

struct Basic {
    counter: usize,
    flex: Option<WidgetId>,
}

impl Application for Basic {
    type Message = BasicMessage;

    fn build(&mut self, context: &mut WidgetContext<Self::Message>) -> WidgetId {
        let rows = (0..2)
            .map(|_| FlexItem::Flex(flex_row(context), 1))
            .collect();
        let flex =
            context.create_widget(Flex::new(rows, FlexDirection::Vertical, FlexAlign::Middle));
        self.flex = Some(flex);

        let add_button = Button::new(context, (0.0, 1.0, 0.0, 1.0), "Add");
        let add_button = context.create_widget(add_button);

        let remove_button = Button::new(context, (1.0, 0.0, 0.0, 1.0), "Remove");
        let remove_button = context.create_widget(remove_button);

        context.add_click_listener(add_button, ClickListener::new(|| BasicMessage::Add));

        context.add_click_listener(remove_button, ClickListener::new(|| BasicMessage::Remove));

        let buttons = context.create_widget(Flex::new(
            vec![
                FlexItem::Flex(add_button, 1),
                FlexItem::Flex(remove_button, 1),
            ],
            FlexDirection::Horizontal,
            FlexAlign::Middle,
        ));

        context.create_widget(Flex::new(
            vec![FlexItem::Flex(flex, 9), FlexItem::Flex(buttons, 1)],
            FlexDirection::Vertical,
            FlexAlign::Middle,
        ))
    }

    fn handle_message(
        &mut self,
        message: Self::Message,
        context: &mut WidgetContext<Self::Message>,
    ) {
        if let Some(flex) = self.flex {
            match message {
                BasicMessage::Add => {
                    self.counter += 1;
                    let new_row =
                        context.create_widget(Label::new(format!("New Item {}", self.counter)));
                    let padded = context.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, new_row));
                    context.send_message(flex, FlexEvent::AddChild(FlexItem::Flex(padded, 1)))
                }
                BasicMessage::Remove => {
                    context.send_message(flex, FlexEvent::RemoveChild);
                }
            }
        }
    }
}

fn main() {
    let mut imagine = Imagine::new(Basic {
        counter: 0,
        flex: None,
    });

    imagine.create_window("Basic Demo!", Size::new(1024.0, 768.0));
    imagine.run();
}

fn flex_row(context: &mut WidgetContext<BasicMessage>) -> WidgetId {
    let children = (0..5)
        .map(|_| {
            let block =
                context.create_widget(FillBox::new(Size::new(20.0, 20.0), (1.0, 0.0, 0.0, 1.0)));
            FlexItem::Flex(
                context.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, block)),
                1,
            )
        })
        .collect();

    context.create_widget(Flex::new(
        children,
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ))
}
