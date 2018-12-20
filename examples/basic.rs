use imagine::{Imagine, Size, WidgetId};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexItem, Padding};

fn main() {
    let mut imagine = Imagine::default();

    let rows = (0..20)
        .map(|_| FlexItem::Flex(flex_row(&mut imagine), 1))
        .collect();
    let root = imagine.create_widget(Flex::new(rows, FlexDirection::Vertical, FlexAlign::Middle));

    imagine.create_window("Basic Demo!", root, Size::new(1024.0, 768.0));
    imagine.run();
}

fn flex_row(imagine: &mut Imagine) -> WidgetId {
    let children = (0..50)
        .map(|_| {
            let block = imagine.create_widget(FillBox::new(
                Size::new(20.0, 20.0),
                (1.0, 0.0, 0.0, 1.0),
                None,
            ));
            FlexItem::Flex(
                imagine.create_widget(Padding::new(2.0, 2.0, 2.0, 1.0, block)),
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
