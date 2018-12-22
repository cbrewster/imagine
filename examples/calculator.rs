use imagine::{Imagine, Size, WidgetId};
use imagine_toolkit::{Button, Flex, FlexAlign, FlexDirection, FlexItem, Label, Padding};

fn main() {
    let mut imagine = Imagine::default();

    let result = imagine.create_widget(Label::new("0000"));

    let rows = vec![
        FlexItem::NonFlex(imagine.create_widget(Padding::new(5.0, 5.0, 5.0, 5.0, result))),
        FlexItem::Flex(
            calc_row(&mut imagine, &[("C", 1), ("+/-", 1), ("%", 1), ("/", 1)]),
            1,
        ),
        FlexItem::Flex(
            calc_row(&mut imagine, &[("7", 1), ("8", 1), ("9", 1), ("*", 1)]),
            1,
        ),
        FlexItem::Flex(
            calc_row(&mut imagine, &[("4", 1), ("5", 1), ("6", 1), ("-", 1)]),
            1,
        ),
        FlexItem::Flex(
            calc_row(&mut imagine, &[("1", 1), ("2", 1), ("3", 1), ("+", 1)]),
            1,
        ),
        FlexItem::Flex(calc_row(&mut imagine, &[("0", 2), (".", 1), ("=", 1)]), 1),
    ];

    let root = imagine.create_widget(Flex::new(
        rows,
        FlexDirection::Vertical,
        FlexAlign::Baseline,
    ));

    imagine.create_window("Calculator!", root, Size::new(280.0, 350.0));
    imagine.run();
}

fn calc_row(imagine: &mut Imagine, items: &[(&str, usize)]) -> WidgetId {
    let children = items
        .iter()
        .map(|(item, flex)| {
            let button = Button::new(imagine, (0.957, 0.586, 0.16, 1.0), item);
            let button = imagine.create_widget(button);
            FlexItem::Flex(
                imagine.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, button)),
                *flex,
            )
        })
        .collect();

    imagine.create_widget(Flex::new(
        children,
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ))
}
