use imagine::{Imagine, RenderTreeBuilder, Size, Ui, Widget, WidgetId};
use imagine_toolkit::{FillBox, Flex, FlexAlign, FlexDirection, FlexItem, List, Padding};

struct Basic {
    count: usize,
}

impl Ui for Basic {
    fn render(&self, builder: &mut RenderTreeBuilder) -> WidgetId {
        let children = (0..5)
            .map(|i| row(self.count, 1.0 * (i as f32 / 5.0 as f32), builder))
            .collect();

        List::new(children).create(builder)
    }
}

fn row(count: usize, blue: f32, builder: &mut RenderTreeBuilder) -> WidgetId {
    let children = (0..count)
        .map(|i| {
            let red = 1.0 * (i as f32 / count as f32);
            let item =
                FillBox::new(Size::new(100.0, 100.0), (red, 0.0, blue, 1.0), None).create(builder);
            FlexItem::Flex(item, 1)
        })
        .collect();

    Flex::new(children, FlexDirection::Horizontal, FlexAlign::Middle).create(builder)
}

fn main() {
    Imagine::new(Basic { count: 10 }, "Basic").run()
}
