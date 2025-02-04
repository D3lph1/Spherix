use spherix_math::vector::Vector3;
use spherix_world::block::state::BlockState;
use std::sync::Arc;

pub trait BlockStateFiller {
    fn calculate(&self, pos: &Vector3) -> Option<Arc<BlockState>>;
}

pub struct MaterialRuleList(Vec<Box<dyn BlockStateFiller>>);

impl MaterialRuleList {
    #[inline]
    pub fn new(fillers: Vec<Box<dyn BlockStateFiller>>) -> Self {
        Self(fillers)
    }
}

impl BlockStateFiller for MaterialRuleList {
    fn calculate(&self, pos: &Vector3) -> Option<Arc<BlockState>> {
        for filler in self.0.iter() {
            let mb_state = filler.calculate(pos);
            if mb_state.is_some() {
                return Some(mb_state.unwrap())
            }
        }

        None
    }
}
