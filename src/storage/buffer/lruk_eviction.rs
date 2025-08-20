use std::collections::{HashMap, LinkedList};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

use crate::storage::buffer::eviction::{AccessType, EvictionPolicy};
use crate::storage::buffer::frame::FrameId;

const BACKWARD_DISTANCE_INF: u64 = u64::MAX;

pub struct LRUKNode {
    frame_id: FrameId,
    is_evictable: bool,
    history: LinkedList<u64>,
}

pub struct LRUKEvictionPolicy {
    /// The number of historical accesses to track.
    k: usize,
    /// The frames that are currently in the buffer pool.
    nodes_store: Arc<RwLock<HashMap<FrameId, LRUKNode>>>,
    /// An incremental counters that tracks the current timestamp starting from 0.
    current_timestamp: Arc<AtomicU64>,
}

impl LRUKEvictionPolicy {
    pub fn new(k: usize, max_size: usize) -> Self {
        assert!(k > 0, "k must be greater than 0");
        let frames = HashMap::with_capacity(max_size);

        LRUKEvictionPolicy {
            k,
            current_timestamp: Arc::new(AtomicU64::new(0)),
            nodes_store: Arc::new(RwLock::new(frames)),
        }
    }

    #[allow(unused)]
    /// The number of frames that can be evicted
    pub fn size(&self) -> usize {
        let frames = self.nodes_store.read().unwrap();
        frames.iter().filter(|(_, node)| node.is_evictable).count()
    }

    fn next_timestamp(&self) -> u64 {
        self.current_timestamp.fetch_add(1, Ordering::SeqCst)
    }

    fn get_timestamp(&self) -> u64 {
        self.current_timestamp.load(Ordering::SeqCst)
    }

    #[allow(unused)]
    fn debug_dump(&self) {
        let frames = self.nodes_store.read().unwrap();
        log::debug!("Current timestamp: {}", self.get_timestamp());
        for (frame_id, node) in frames.iter() {
            log::debug!(
                "FrameId: {}, Evictable: {}, History: {:?}",
                frame_id,
                node.is_evictable,
                node.history
            );
        }
    }
}

impl EvictionPolicy for LRUKEvictionPolicy {
    /// The LRU-K algorithm evicts a frame whose backward k-distance is the maximum of all frames
    /// in the replacer. Backward k-distance is computed as the difference in time between the
    /// current timestamp and the timestamp of kth previous access.
    ///
    /// A frame with fewer than k historical accesses is given +inf as its backward k-distance.
    /// If multiple frames have +inf backward k-distance, the replacer evicts the frame with
    /// the earliest overall timestamp (i.e., the frame whose least-recent recorded access is
    /// the overall least recent access).
    fn evict(&self) -> Option<FrameId> {
        let current_timestamp = self.get_timestamp();

        let mut frame_to_evict = Option::<FrameId>::None;
        let mut least_recent_access = 0_u64;
        let mut max_backward_distance = 0_u64;

        // We will search for all the frames and find which is the best candidate to evict.
        for frame in self.nodes_store.read().unwrap().values() {
            assert!(!frame.history.is_empty() && frame.history.len() <= self.k);

            if !frame.is_evictable {
                continue;
            }

            let last_access = frame.history.iter().last().unwrap();
            let less_than_k_accesses = frame.history.len() < self.k;

            // A frame with less than k historical accesses has been found
            if max_backward_distance == BACKWARD_DISTANCE_INF {
                if less_than_k_accesses && *last_access < least_recent_access {
                    least_recent_access = *last_access;
                    frame_to_evict = Some(frame.frame_id);
                }
            } else if less_than_k_accesses {
                max_backward_distance = BACKWARD_DISTANCE_INF;
                least_recent_access = *last_access;
                frame_to_evict = Some(frame.frame_id);
            } else {
                let backward_distance = current_timestamp - last_access;
                if backward_distance > max_backward_distance {
                    max_backward_distance = backward_distance;
                    frame_to_evict = Some(frame.frame_id);
                }
            }
        }

        match frame_to_evict {
            Some(frame_id) => {
                self.remove(frame_id);
                Some(frame_id)
            }
            _ => None,
        }
    }

    /// Record an access to a frame. This information will be used to compute the
    /// backward k-distance of the frame.
    ///
    /// The frame is initially marked as non-evictable. If the frame is not found,
    /// it will be inserted with the default values.
    fn record_access(&self, frame_id: FrameId, _access_type: AccessType) {
        let now = self.next_timestamp();

        let mut frames = self.nodes_store.write().unwrap();

        let node = frames.entry(frame_id).or_insert(LRUKNode {
            frame_id,
            is_evictable: false,
            history: LinkedList::new(),
        });

        if node.history.len() == self.k {
            node.history.pop_front();
        }
        node.history.push_back(now);
    }

    /// Whether the frame is evictable or not. Panics if the frame is not found.
    fn set_evictable(&self, frame_id: FrameId, is_evictable: bool) {
        let mut frames = self.nodes_store.write().unwrap();
        let node = frames
            .get_mut(&frame_id)
            .unwrap_or_else(|| panic!("set_evictable: Frame with frame_id={} not found", frame_id));
        node.is_evictable = is_evictable;
    }

    fn remove(&self, frame_id: FrameId) {
        self.nodes_store.write().unwrap().remove(&frame_id);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::panic;

    #[test]
    /// Corresponding CMU test suite
    /// https://github.com/cmu-db/bustub/blob/8de6f6b57fbb3511f21e581379464c2e182d751d/test/buffer/lru_k_replacer_test.cpp
    fn test_lruk_replacer() {
        let n_frames = 7;
        let k = 2;

        let lru_replacer = LRUKEvictionPolicy::new(k, n_frames);

        // Add six frames to the replacer. We now have frames [1, 2, 3, 4, 5]. We set frame 6 as non-evictable.
        lru_replacer.record_access(1, AccessType::Lookup);
        lru_replacer.record_access(2, AccessType::Lookup);
        lru_replacer.record_access(3, AccessType::Lookup);
        lru_replacer.record_access(4, AccessType::Lookup);
        lru_replacer.record_access(5, AccessType::Lookup);
        lru_replacer.record_access(6, AccessType::Lookup);
        lru_replacer.set_evictable(1, true);
        lru_replacer.set_evictable(2, true);
        lru_replacer.set_evictable(3, true);
        lru_replacer.set_evictable(4, true);
        lru_replacer.set_evictable(5, true);
        lru_replacer.set_evictable(6, false);

        // The size of the replacer is the number of frames that can be evicted, _not_ the total number of frames entered.
        assert_eq!(5, lru_replacer.size());

        // Record an access for frame 1. Now frame 1 has two accesses total.
        lru_replacer.record_access(1, AccessType::Lookup);
        // All other frames now share the maximum backward k-distance. Since we use timestamps to break ties, where the first
        // to be evicted is the frame with the oldest timestamp, the order of eviction should be [2, 3, 4, 5, 1].

        // Evict three pages from the replacer.
        // To break ties, we use LRU with respect to the oldest timestamp, or the least recently used frame.
        assert_eq!(Some(2), lru_replacer.evict());
        assert_eq!(Some(3), lru_replacer.evict());
        assert_eq!(Some(4), lru_replacer.evict());
        assert_eq!(2, lru_replacer.size());

        // Now the replacer has the frames [5, 1].

        // Insert new frames [3, 4], and update the access history for 5. Now, the ordering is [3, 1, 5, 4].
        lru_replacer.record_access(3, AccessType::Lookup);
        lru_replacer.record_access(4, AccessType::Lookup);
        lru_replacer.record_access(5, AccessType::Lookup);
        lru_replacer.record_access(4, AccessType::Lookup);
        lru_replacer.set_evictable(3, true);
        lru_replacer.set_evictable(4, true);
        assert_eq!(4, lru_replacer.size());

        // Look for a frame to evict. We expect frame 3 to be evicted next.
        assert_eq!(Some(3), lru_replacer.evict());
        assert_eq!(3, lru_replacer.size());

        // Set 6 to be evictable. 6 Should be evicted next since it has the maximum backward k-distance.
        lru_replacer.set_evictable(6, true);
        assert_eq!(4, lru_replacer.size());
        assert_eq!(Some(6), lru_replacer.evict());
        assert_eq!(3, lru_replacer.size());

        // Mark frame 1 as non-evictable. We now have [5, 4].
        lru_replacer.set_evictable(1, false);

        // We expect frame 5 to be evicted next.
        assert_eq!(2, lru_replacer.size());
        assert_eq!(Some(5), lru_replacer.evict());
        assert_eq!(1, lru_replacer.size());

        // Update the access history for frame 1 and make it evictable. Now we have [4, 1].
        lru_replacer.record_access(1, AccessType::Lookup);
        lru_replacer.record_access(1, AccessType::Lookup);
        lru_replacer.set_evictable(1, true);
        assert_eq!(2, lru_replacer.size());

        // Evict the last two frames.
        assert_eq!(Some(4), lru_replacer.evict());
        assert_eq!(1, lru_replacer.size());
        assert_eq!(Some(1), lru_replacer.evict());
        assert_eq!(0, lru_replacer.size());

        // Insert frame 1 again and mark it as non-evictable.
        lru_replacer.record_access(1, AccessType::Lookup);
        lru_replacer.set_evictable(1, false);
        assert_eq!(0, lru_replacer.size());

        // A failed eviction should not change the size of the replacer.
        let frame = lru_replacer.evict();
        assert_eq!(false, frame.is_some());

        // Mark frame 1 as evictable again and evict it.
        lru_replacer.set_evictable(1, true);
        assert_eq!(1, lru_replacer.size());
        assert_eq!(Some(1), lru_replacer.evict());
        assert_eq!(0, lru_replacer.size());

        // There is nothing left in the replacer, so make sure this doesn't do something strange.
        let frame = lru_replacer.evict();
        assert_eq!(false, frame.is_some());
        assert_eq!(0, lru_replacer.size());

        // Make sure that setting a non-existent frame as evictable or non-evictable doesn't do something strange.
        let result = panic::catch_unwind(|| {
            lru_replacer.set_evictable(6, false);
            lru_replacer.set_evictable(6, true);
        });
        assert!(result.is_err());
    }
}
