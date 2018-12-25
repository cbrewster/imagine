use imagine::{Application, ClickListener, Imagine, Size, WidgetContext, WidgetId};
use imagine_toolkit::{
    Button, Flex, FlexAlign, FlexDirection, FlexItem, Label, LabelMessage, Padding,
};

#[derive(Clone, Debug)]
enum CalcMessage {
    Num(f32),
    Op(Op),
    Decimal,
    Negative,
    Eq,
    Clear,
}

#[derive(Clone, Debug)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

struct Calculator {
    current: f32,
    operator: Option<(f32, Op)>,
    decimal: Option<f32>,
    display: Option<WidgetId>,
}

impl Calculator {
    fn new() -> Calculator {
        Calculator {
            current: 0.0,
            operator: None,
            decimal: None,
            display: None,
        }
    }
}

impl Application for Calculator {
    type Message = CalcMessage;

    fn build(&mut self, context: &mut WidgetContext<Self::Message>) -> WidgetId {
        let result = context.create_widget(Label::new("0"));

        let display = context.create_widget(Padding::new(5.0, 5.0, 5.0, 5.0, result));
        self.display = Some(result);

        let rows = vec![
            FlexItem::NonFlex(display),
            FlexItem::Flex(
                calc_row(
                    context,
                    vec![
                        ("C", 1, CalcMessage::Clear),
                        ("+/-", 1, CalcMessage::Negative),
                        ("%", 1, CalcMessage::Op(Op::Mod)),
                        ("/", 1, CalcMessage::Op(Op::Div)),
                    ],
                ),
                1,
            ),
            FlexItem::Flex(
                calc_row(
                    context,
                    vec![
                        ("7", 1, CalcMessage::Num(7.0)),
                        ("8", 1, CalcMessage::Num(8.0)),
                        ("9", 1, CalcMessage::Num(9.0)),
                        ("*", 1, CalcMessage::Op(Op::Mul)),
                    ],
                ),
                1,
            ),
            FlexItem::Flex(
                calc_row(
                    context,
                    vec![
                        ("4", 1, CalcMessage::Num(4.0)),
                        ("5", 1, CalcMessage::Num(5.0)),
                        ("6", 1, CalcMessage::Num(6.0)),
                        ("-", 1, CalcMessage::Op(Op::Sub)),
                    ],
                ),
                1,
            ),
            FlexItem::Flex(
                calc_row(
                    context,
                    vec![
                        ("1", 1, CalcMessage::Num(1.0)),
                        ("2", 1, CalcMessage::Num(2.0)),
                        ("3", 1, CalcMessage::Num(3.0)),
                        ("+", 1, CalcMessage::Op(Op::Add)),
                    ],
                ),
                1,
            ),
            FlexItem::Flex(
                calc_row(
                    context,
                    vec![
                        ("0", 2, CalcMessage::Num(0.0)),
                        (".", 1, CalcMessage::Decimal),
                        ("=", 1, CalcMessage::Eq),
                    ],
                ),
                1,
            ),
        ];

        context.create_widget(Flex::new(
            rows,
            FlexDirection::Vertical,
            FlexAlign::Baseline,
        ))
    }

    fn handle_message(&mut self, message: CalcMessage, context: &mut WidgetContext<Self::Message>) {
        println!("Received Message: {:?}", message);
        match message {
            CalcMessage::Num(number) => {
                if let Some(decimal) = self.decimal {
                    self.current += decimal * number;
                    self.decimal = Some(decimal / 10.0);
                } else {
                    self.current *= 10.0;
                    self.current += number;
                }
            }
            CalcMessage::Eq => {
                if let Some((prev, op)) = self.operator.take() {
                    self.current = match op {
                        Op::Add => prev + self.current,
                        Op::Sub => prev - self.current,
                        Op::Mul => prev * self.current,
                        Op::Div => prev / self.current,
                        Op::Mod => prev % self.current,
                    }
                }
            }
            CalcMessage::Negative => {
                self.current *= -1.0;
            }
            CalcMessage::Op(op) => match &self.operator {
                None => {
                    self.operator = Some((self.current, op));
                    self.current = 0.0;
                    self.decimal = None;
                }
                Some((prev, old_op)) => {
                    self.current = match old_op {
                        Op::Add => prev + self.current,
                        Op::Sub => prev - self.current,
                        Op::Mul => prev * self.current,
                        Op::Div => prev / self.current,
                        Op::Mod => prev % self.current,
                    };
                    self.operator = Some((self.current, op));
                    self.current = 0.0;
                    self.decimal = None;
                }
            },
            CalcMessage::Clear => {
                self.current = 0.0;
                self.operator = None;
                self.decimal = None;
            }
            CalcMessage::Decimal => {
                if self.decimal.is_none() {
                    self.decimal = Some(0.1);
                }
            }
        }

        let current = format!("{}", self.current);
        if let Some(display) = self.display {
            context.send_message(display, LabelMessage::SetText(current));
        }
    }
}

fn main() {
    let mut imagine = Imagine::new(Calculator::new());

    imagine.create_window("Calculator!", Size::new(280.0, 350.0));
    imagine.run();
}

fn calc_row(
    context: &mut WidgetContext<CalcMessage>,
    items: Vec<(&str, usize, CalcMessage)>,
) -> WidgetId {
    let children = items
        .into_iter()
        .map(|(item, flex, message)| {
            let button = Button::new(context, (0.957, 0.586, 0.16, 1.0), item);
            let button = context.create_widget(button);
            context.add_click_listener(button, ClickListener::new(move || message.clone()));
            FlexItem::Flex(
                context.create_widget(Padding::new(2.0, 2.0, 2.0, 2.0, button)),
                flex,
            )
        })
        .collect();

    context.create_widget(Flex::new(
        children,
        FlexDirection::Horizontal,
        FlexAlign::Middle,
    ))
}
