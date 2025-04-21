use crate::{config::PAGE_SIZE, storage::tuple::Tuple};

// TODO
// - Representing null values

pub type PageId = u32;

/// 16bit offset + 16bit length
const SLOTTED_PAGE_SLOT_SIZE: usize = 4;
const SLOTTED_PAGE_HEADER_SIZE: usize = 0;

pub struct SlottedPage<'a> {
    // slice of bytes representing the page
    data: &'a mut [u8; PAGE_SIZE],
}

impl<'a> SlottedPage<'a> {
    /// Lookups the slot array for the given slot number
    pub fn get_n_tuple(self, n: usize) -> Tuple<'a> {
        let slot_offset = SLOTTED_PAGE_HEADER_SIZE + n * SLOTTED_PAGE_SLOT_SIZE;
        let slot = &self.data[slot_offset..slot_offset + SLOTTED_PAGE_SLOT_SIZE];

        assert!(slot.len() == SLOTTED_PAGE_SLOT_SIZE);

        let offset = u16::from_be_bytes([slot[0], slot[1]]) as usize;
        let length = u16::from_be_bytes([slot[2], slot[3]]) as usize;

        assert!(
            offset + length <= PAGE_SIZE,
            "Page slot reported an invalid offset"
        );

        Tuple::from(&self.data[offset..offset + length])
    }
}
