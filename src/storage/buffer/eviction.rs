use super::frame::FrameId;

#[allow(unused)]
pub enum AccessType {
    Lookup,
    Scan,
    Index,
}

pub trait EvictionPolicy {
    fn evict(&self) -> Option<FrameId>;

    fn record_access(&self, frame_id: FrameId, access_type: AccessType);

    fn set_evictable(&self, frame_id: FrameId, is_evictable: bool);

    fn remove(&self, frame_id: FrameId);
}
