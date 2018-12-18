use imagine::{Imagine, Size};
use imagine_toolkit::{FillBox, Padding, Split};

fn main() {
    let mut imagine = Imagine::new();

    let left = imagine.add_widget(FillBox::new(
        Size::new(5000.0, 5000.0),
        (1.0, 0.0, 0.0, 1.0),
    ));
    let right = imagine.add_widget(FillBox::new(
        Size::new(5000.0, 5000.0),
        (1.0, 1.0, 0.0, 1.0),
    ));

    let left = imagine.add_widget(Padding::new(10.0, 10.0, 10.0, 10.0, left));
    let right = imagine.add_widget(Padding::new(20.0, 20.0, 20.0, 20.0, right));

    let root = imagine.add_widget(Split::new(left, right, 0.5));

    imagine.create_window("Test!", root, Size::new(1024.0, 768.0));

    let left = imagine.add_widget(FillBox::new(
        Size::new(5000.0, 5000.0),
        (1.0, 0.0, 0.0, 1.0),
    ));
    let right = imagine.add_widget(FillBox::new(
        Size::new(5000.0, 5000.0),
        (1.0, 1.0, 0.0, 1.0),
    ));

    let left = imagine.add_widget(Padding::new(10.0, 10.0, 10.0, 10.0, left));
    let right = imagine.add_widget(Padding::new(20.0, 20.0, 20.0, 20.0, right));

    let root = imagine.add_widget(Split::new(left, right, 0.5));

    imagine.create_window("Test2", root, Size::new(1024.0, 768.0));

    imagine.run();
}
