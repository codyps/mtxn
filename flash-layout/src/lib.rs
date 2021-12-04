//! Describe the layout of flash memory devices
//!
//! Flash devices are composed of erase blocks. Depending on the device, these can be fixed size or
//! varry in size, and may be numerous. `flash-layout` provides a consise way to describe the
//! location and size of erase blocks for a particular flash device.
//!
//! # Notes on naming
//!
//! Some manufacturers will use the term `sectors` to describe flash erase blocks (like STM).

#[derive(Debug, Clone, Copy)]
pub struct FlashLayout<'a> {
    pub regions: &'a [Region],
}

impl<'a> FlashLayout<'a> {
    pub fn new(regions: &'a [Region]) -> Self {
        let s = Self { regions };
        s.validate_regions();
        s
    }

    // NOTE: non-const because `const_for` is not yet stable
    fn validate_regions(&self) {
        // - regions must be ordered
        // - regions must _not_ overlap
        //
        if self.regions.is_empty() {
            panic!("at least 1 region required");
        }

        for region in self.regions.windows(2) {
            if region[0].addr_end() > region[1].addr_start() {
                panic!("region overlap or mis-ordering");
            }
        }
    }

    pub fn addr_start(&self) -> u64 {
        self.regions[0].addr_start()
    }

    pub fn addr_end(&self) -> u64 {
        self.regions[self.regions.len() - 1].addr_end()
    }

    pub fn len(&self) -> u64 {
        let mut s = 0;
        for r in self.regions {
            s += r.len();
        }

        s
    }

    pub fn find_eb_by_addr(&self, addr: u64) -> Option<(EraseBlock<'a>, u32)> {
        for (i, r) in self.regions.into_iter().enumerate() {
            if r.contains_addr(addr) {
                let addr_in_r = addr - r.addr_start();
                let eb_n = addr_in_r / r.eb_bytes as u64;
                let eb_r = addr_in_r % r.eb_bytes as u64;

                return Some((
                    EraseBlock {
                        layout: *self,
                        region_idx: i,
                        eb_offs_in_region: eb_n.try_into().unwrap(),
                    },
                    eb_r.try_into().unwrap(),
                ));
            }
        }

        None
    }

    pub fn find_eb_by_eb_num(&self, eb_num: u32) -> Option<EraseBlock<'a>> {
        let mut eb_a = 0;
        for (i, r) in self.regions.into_iter().enumerate() {
            let eb_n = r.eb_count + eb_a;
            if eb_num < eb_n {
                return Some(EraseBlock {
                    layout: *self,
                    region_idx: i,
                    eb_offs_in_region: eb_num - eb_a,
                });
            }

            eb_a = eb_n;
        }

        None
    }

    /*
    pub fn eb_range_from_addr_range(&self, addr_start: u64, addr_end: u64) -> Option<Range<'a>> {
        todo!()
    }
    */
}

/// A region within a flash device which contains a particular size and number of erase blocks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Region {
    /// base address of this region (address of the first erase block)
    pub addr: u64,

    /// Number of bytes per erase block in this region
    pub eb_bytes: u32,

    /// Number of erase blocks within this region
    pub eb_count: u32,
}

impl Region {
    pub fn addr_start(&self) -> u64 {
        self.addr
    }

    pub fn addr_end(&self) -> u64 {
        self.addr + self.len()
    }

    pub fn len(&self) -> u64 {
        self.eb_bytes as u64 * self.eb_count as u64
    }

    pub fn contains_addr(&self, addr: u64) -> bool {
        addr < self.addr_end() && addr >= self.addr_start()
    }
}

/// A single erase block within a Region
#[derive(Debug, Clone)]
pub struct EraseBlock<'a> {
    layout: FlashLayout<'a>,
    // TODO: if we're ok with some pointer comparisons, we can store the reference here directly.
    region_idx: usize,
    eb_offs_in_region: u32,
}

impl<'a> EraseBlock<'a> {
    pub fn region(&self) -> &'a Region {
        &self.layout.regions[self.region_idx]
    }

    /// Number of erase blocks from the start of the region to where this erase block is located
    pub fn eb_offs_in_region(&self) -> u32 {
        self.eb_offs_in_region
    }

    /// First address in the erase block
    ///
    /// `Self::addr_start()` and `Self::addr_end()` combined give a inclusive left, exclusive
    /// right, range.
    pub fn addr_start(&self) -> u64 {
        self.region().addr_start() + self.region().eb_bytes as u64 * self.eb_offs_in_region as u64
    }

    /// End address of this erase block, 1 byte past the last byte inside the erase block
    ///
    /// `Self::addr_start()` and `Self::addr_end()` combined give a inclusive left, exclusive
    /// right, range.
    pub fn addr_end(&self) -> u64 {
        self.addr_start() + self.region().eb_bytes as u64
    }

    /// Number of bytes in this erase block
    pub fn len(&self) -> u32 {
        self.region().eb_bytes
    }

    /*
    /// Erase block number (within the containing `Layout`) of this erase block
    pub fn eb_num(&self) -> u32 {
        todo!()
    }
    */
}

/// A flat sequence of erase blocks
#[derive(Debug, Clone)]
pub struct Range<'a> {
    layout: FlashLayout<'a>,
    first_eb: EraseBlock<'a>,
    addr_end: u64,
}

impl<'a> Range<'a> {
    pub fn addr_start(&self) -> u64 {
        let byte_ct = {
            let mut byte_ct = 0u64;
            for (i, r) in self.layout.regions.into_iter().enumerate() {
                if i == self.first_eb.region_idx {
                    break;
                }
                byte_ct += r.len();
            }

            byte_ct
                + self.first_eb.eb_offs_in_region as u64 * self.first_eb.region().eb_bytes as u64
        };
        self.layout.addr_start() + byte_ct
    }

    pub fn addr_end(&self) -> u64 {
        self.addr_end
    }
}

// FIXME: unclear what IntoIterator we need/want here
/*
impl<'a> IntoIterator for Range<'a> {
    type Item = EraseBlock<'a>;
    type IntoIter = Self;

    fn into_iter(self) -> Self::IntoIter {
        self
    }
}
*/

impl<'a> Iterator for Range<'a> {
    type Item = EraseBlock<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.first_eb.region_idx > self.layout.regions.len() {
            return None;
        }
        if self.first_eb.addr_end() >= self.addr_end {
            return None;
        }

        if self.first_eb.eb_offs_in_region >= self.first_eb.region().eb_count {
            let next_region = self.first_eb.region_idx + 1;
            if next_region > self.layout.regions.len() {
                // TODO: consider short circuiting here?
                return None;
            }
        }

        self.first_eb.eb_offs_in_region += 1;

        Some(self.first_eb.clone())
    }
}
