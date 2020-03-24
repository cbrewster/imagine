use webrender::api::{DisplayListBuilder, FontInstanceKey, SpaceAndClipInfo, PipelineId};

pub struct RenderContext<'a> {
    pub builder: &'a mut DisplayListBuilder,
    pub current_space_and_clip: SpaceAndClipInfo,
    next_tag_identifier: u64,
    font_instance_key: FontInstanceKey,
}

impl<'a> RenderContext<'a> {
    pub(crate) fn new(
        builder: &'a mut DisplayListBuilder,
        font_instance_key: FontInstanceKey,
        pipeline_id: PipelineId,
    ) -> RenderContext<'a> {
        RenderContext {
            builder,
            next_tag_identifier: 0,
            font_instance_key,
            current_space_and_clip: SpaceAndClipInfo::root_scroll(pipeline_id),
        }
    }

    pub fn next_tag_identifier(&mut self) -> u64 {
        let identifier = self.next_tag_identifier;
        self.next_tag_identifier += 1;
        identifier
    }

    pub fn font_instance_key(&self) -> FontInstanceKey {
        self.font_instance_key
    }
}
