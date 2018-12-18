use imagine::{Imagine, Size, WidgetId};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexItem, Padding};

fn main() {
    let mut imagine = Imagine::default();

    let root = flex_row(&mut imagine);

    imagine.create_window("Basic Demo!", root, Size::new(1024.0, 768.0));
    imagine.run();
}

fn flex_row(imagine: &mut Imagine) -> WidgetId {
    let a = imagine.create_widget(FillBox::new(Size::new(30.0, 30.0), (1.0, 0.0, 0.0, 1.0)));
    let b = imagine.create_widget(FillBox::new(Size::new(30.0, 30.0), (0.0, 1.0, 0.0, 1.0)));
    let c = imagine.create_widget(FillBox::new(Size::new(30.0, 30.0), (0.0, 0.0, 1.0, 1.0)));

    let fill1 = imagine.create_widget(FillBox::new(Size::new(30.0, 30.0), (0.0, 1.0, 1.0, 1.0)));
    let fill2 = imagine.create_widget(FillBox::new(Size::new(30.0, 30.0), (1.0, 0.0, 1.0, 1.0)));

    let a_padded = imagine.create_widget(Padding::new(5.0, 5.0, 5.0, 5.0, a));
    let b_padded = imagine.create_widget(Padding::new(5.0, 5.0, 5.0, 5.0, b));
    let c_padded = imagine.create_widget(Padding::new(5.0, 5.0, 5.0, 5.0, c));

    imagine.create_widget(Flex::new(
        vec![
            FlexItem::NonFlex(a_padded),
            FlexItem::Flex(fill1, 1),
            FlexItem::NonFlex(b_padded),
            FlexItem::Flex(fill2, 2),
            FlexItem::NonFlex(c_padded),
        ],
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ))
}
