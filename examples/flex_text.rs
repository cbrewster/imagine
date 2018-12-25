use imagine::{Application, Imagine, Size, WidgetContext, WidgetId};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexItem, Label};

struct FlexText;

impl Application for FlexText {
    type Message = ();

    fn build(&mut self, context: &mut WidgetContext<Self::Message>) -> WidgetId {
        let left = context.create_widget(Label::new("Left Text"));
        let right = context.create_widget(Label::new("Right Text"));
        let fill = context.create_widget(FillBox::new(Size::new(10.0, 10.0), (0.0, 0.0, 0.0, 0.0)));

        context.create_widget(Flex::new(
            vec![
                FlexItem::NonFlex(left),
                FlexItem::Flex(fill, 1),
                FlexItem::NonFlex(right),
            ],
            FlexDirection::Horizontal,
            FlexAlign::Middle,
        ))
    }
}

fn main() {
    let mut imagine = Imagine::new(FlexText);
    imagine.create_window("Flex Text", Size::new(1024.0, 768.0));
    imagine.run();
}
