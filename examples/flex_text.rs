use imagine::{Imagine, Size};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexItem, Label};

fn main() {
    let mut imagine = Imagine::default();

    let left = imagine.create_widget(Label::new("Left Text"));
    let right = imagine.create_widget(Label::new("Right Text"));
    let fill = imagine.create_widget(FillBox::new(Size::new(10.0, 10.0), (0.0, 0.0, 0.0, 0.0)));

    let root = imagine.create_widget(Flex::new(
        vec![
            FlexItem::NonFlex(left),
            FlexItem::Flex(fill, 1),
            FlexItem::NonFlex(right),
        ],
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ));

    imagine.create_window("Flex Text", root, Size::new(1024.0, 768.0));
    imagine.run();
}
