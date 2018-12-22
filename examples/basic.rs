use imagine::{ClickListener, Imagine, Size, WidgetId};
use imagine_toolkit::{
    Button, FillBox, Flex, FlexAlign, FlexDirection, FlexEvent, FlexItem, Label, Padding,
};

fn main() {
    let mut imagine = Imagine::default();

    let rows = (0..2)
        .map(|_| FlexItem::Flex(flex_row(&mut imagine), 1))
        .collect();
    let flex = imagine.create_widget(Flex::new(rows, FlexDirection::Vertical, FlexAlign::Middle));

    let add_button = Button::new(&mut imagine, (0.0, 1.0, 0.0, 1.0), "Add");
    let add_button = imagine.create_widget(add_button);

    let remove_button = Button::new(&mut imagine, (1.0, 0.0, 0.0, 1.0), "Remove");
    let remove_button = imagine.create_widget(remove_button);

    imagine.add_click_listener(
        add_button,
        ClickListener::new(move |context| {
            let new_row = context.create_widget(Label::new("New Item"));
            let padded = context.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, new_row));
            context.send_message(flex, FlexEvent::AddChild(FlexItem::Flex(padded, 1)))
        }),
    );

    imagine.add_click_listener(
        remove_button,
        ClickListener::new(move |context| context.send_message(flex, FlexEvent::RemoveChild)),
    );

    let buttons = imagine.create_widget(Flex::new(
        vec![
            FlexItem::Flex(add_button, 1),
            FlexItem::Flex(remove_button, 1),
        ],
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ));

    let root = imagine.create_widget(Flex::new(
        vec![FlexItem::Flex(flex, 9), FlexItem::Flex(buttons, 1)],
        FlexDirection::Vertical,
        FlexAlign::Middle,
    ));

    imagine.create_window("Basic Demo!", root, Size::new(1024.0, 768.0));
    imagine.run();
}

fn flex_row(imagine: &mut Imagine) -> WidgetId {
    let children = (0..5)
        .map(|_| {
            let block =
                imagine.create_widget(FillBox::new(Size::new(20.0, 20.0), (1.0, 0.0, 0.0, 1.0)));
            FlexItem::Flex(
                imagine.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, block)),
                1,
            )
        })
        .collect();

    imagine.create_widget(Flex::new(
        children,
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ))
}
