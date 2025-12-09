#[derive(Copy, Clone, Default, Debug)]
pub struct Cache {
    /// Packed value: `[tag_type: 4 bits][parent_offset: 60 bits]`
    pub general_parent_offset: u64,
    /// Current position in list iteration
    pub list_current_length: u32,
    /// Total number of elements in the list
    pub list_total_length: u32,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Store {
    /// Distance to the next mark in the flat array (number of marks to skip including nested ones)
    pub flat_next_mark: u64,
    /// Pointer to the end of this structure
    pub end_pointer: *const u8,
}

#[derive(Copy, Clone)]
pub union Mark {
    pub cache: Cache,
    pub store: Store,
}

unsafe impl Send for Mark {}
unsafe impl Sync for Mark {}
