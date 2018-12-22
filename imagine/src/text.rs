use rusttype::{point, Font, Scale};
use webrender::api::*;

#[derive(Debug)]
pub struct FinalText {
    glyphs: Vec<GlyphInstance>,
    width: f32,
}

impl FinalText {
    pub(crate) fn new(font: &Font, text: &str) -> FinalText {
        let height: f32 = 20.0;

        let scale = Scale {
            x: height * 2.0,
            y: height,
        };
        let v_metrics = font.v_metrics(scale);
        let offset = point(0.0, v_metrics.ascent + 10.0);

        let (glyphs, width) = font.layout(text, scale, offset).fold(
            (Vec::new(), 0.0f32),
            |(mut glyphs, max_width), glyph| {
                let position = glyph.position();
                glyphs.push(GlyphInstance {
                    index: glyph.id().0,
                    point: LayoutPoint::new(position.x, position.y),
                });
                (
                    glyphs,
                    max_width.max(position.x + glyph.unpositioned().h_metrics().advance_width),
                )
            },
        );

        FinalText { glyphs, width }
    }

    pub fn render(
        &self,
        info: &LayoutPrimitiveInfo,
        origin: LayoutPoint,
        builder: &mut DisplayListBuilder,
        font_instance_key: FontInstanceKey,
    ) {
        let glyphs = self
            .glyphs
            .iter()
            .map(|glyph| GlyphInstance {
                index: glyph.index,
                point: LayoutPoint::new(glyph.point.x + origin.x, glyph.point.y + origin.y),
            })
            .collect::<Vec<_>>();
        builder.push_text(info, &glyphs, font_instance_key, ColorF::BLACK, None);
    }

    pub fn width(&self) -> f32 {
        self.width
    }
}
