use imagine::{ClickListener, Imagine, Size, WidgetId};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexEvent, FlexItem, Padding};

fn main() {
    let mut imagine = Imagine::default();

    let rows = (0..2)
        .map(|_| FlexItem::Flex(flex_row(&mut imagine), 1))
        .collect();
    let flex = imagine.create_widget(Flex::new(rows, FlexDirection::Vertical, FlexAlign::Middle));

    let add_button =
        imagine.create_widget(FillBox::new(Size::new(70.0, 200.0), (0.0, 1.0, 0.0, 1.0)));

    imagine.add_click_listener(
        add_button,
        ClickListener::new(move |context| {
            let new_row = context.create_widget(FillBox::new(
                Size::new(std::f32::INFINITY, 20.0),
                (1.0, 0.0, 0.0, 1.0),
            ));
            let padded = context.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, new_row));
            context.send_message(flex, FlexEvent::AddChild(FlexItem::Flex(padded, 1)))
        }),
    );

    let buttons = imagine.create_widget(Flex::new(
        vec![FlexItem::Flex(add_button, 1)],
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ));

    let root = imagine.create_widget(Flex::new(
        vec![FlexItem::Flex(flex, 1), FlexItem::NonFlex(buttons)],
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
