use webrender::api::DisplayListBuilder;

pub struct RenderContext<'a> {
    pub builder: &'a mut DisplayListBuilder,
    next_tag_identifier: u64,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(builder: &'a mut DisplayListBuilder) -> RenderContext<'a> {
        RenderContext {
            builder,
            next_tag_identifier: 0,
        }
    }

    pub fn next_tag_identifier(&mut self) -> u64 {
        let identifier = self.next_tag_identifier;
        self.next_tag_identifier += 1;
        identifier
    }
}
