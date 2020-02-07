//! # What storage do we need?
//!
//!  - Keep most recent version of a data kind ("value")
//!  - Keep a bunch of a kind of data, appending to it. Garbage collect so it behaves as a circular
//!    buffer ("log")
//!  - Provide GC variations to allow rating "importance" of piece of data
//!  - Need to be able to commit changes to multiple independent values safely.
//!
//! # Flash system primitives
//!
//! Typical Flash based storage systems provide the following operations with the restrictions
//! noted below.
//!
//!  - erase sector
//!    - sectors have fixed locations
//!    - systems have more than 1 sector
//!    - sectors may be different sizes (or the same size, called "pages" in that case)
//!    - after erase, sector may either be 0 or 1
//!  - write erased
//!    - can write any data after erase occurs
//!    - has a minumum write size (and alignment. think about sectors as split into blocks of with
//!      a size equal to the minimum write size)
//!    - has some set of supported write sizes
//!  - write after write
//!    - may be able to write _some_ values with a subsequent write
//!    - restricted to one of:
//!      - making more bits 0
//!      - making more bits 1
//!      - making all bits 0
//!      - making all bits 1
//!
//! By using more flexible "write after write" options, we can emit smaller writes than the minimum
//! write size.
//!
//!  - writing to flash may require code to be running from a seperate device
//!    - some ram region
//!    - different flash device
//!    - different flash bank (device split into seperate parts)
//!
//! We're interested in what design is necessary to support the use of a flash bank system when in
//! use as a ping-pong update system without the execution of any code from ram.
//!
//! # Specific Devices
//!
//! ## STM32F0xx
//!  - writes of 16 bit or 32 bit size
//!  - pages (fixed size sectors), 512B each
//!  - 
//!
//! ## STM32F1
//!
//! ## STM32F205/STM32F207
//!  - sectors (4x16K, 1x64K, 7x128K)
//!  - on erase, bits = 1
//!  - write after write toward 0 allowed
//!
//! ## STM32F3
//! 
//! ## STM32F4
//!
//! ## STM32F7
//!  - dual bank
//!  - sectors
//!     - per bank, 1M: 4x16K, 1x64K, 3x128K
//!     - per bank, 2M: 4x16K, 1x64K, 7x128K
//!  - write sizes: 8, 16, 32, 64 _bytes_
//!
//! ## STM32L1
//!
//! - 2 banks
//!
//! ### flash interface
//!  - sectors
//!    - which contain pages
//!    - specifics vary per device
//!    - cat1 & 2:
//!     - each sector 4K in size
//!     - sector 0 split into pages: 4x256, 3x1K
//!  - 32 bit writes
//!  - erase to 0
//!  - write after write is unreliable (due to ecc?)
//!
//! also has a EEPROM
//!
//! ## STM32L4
//!
//! - 2 banks
//! - erase to 1
//! - 72 bit writes (8 bit ecc included)
//! - write after read not mentioned in docs
//!  
//! ## NRF52
//!
//! Called NVM, and includes an additional restriction:
//!
//! - erase to 1
//! - after 181 writes to addresses in the same block, must be erased
//!   - 512/181 ~= 2.8287
//! - write to 0
//! - write after write allowed, but keep in mind 181 write limit.
//! - blocks are 512 bytes
//! - write size is 4 bytes (32 bits)
//!
//! ## NRF51
//!
//!
//! ## TI cc26xx/cc13xx
//!
//! ## TI Hercules
//!
//! ## ATMEL SAM3
//!
//!
//!
//! # MTXN data layout
//!
//! ## sector header
//!
//!   magic:    u16
//!   version:  u16
//!   erase_ct: u32
//!   sequence: u32
//!
//! ## transaction header
//!
//!   magic:   u16
//!   version: u16
//!   length:  u16
//!   
//! ## item
//!
//!   magic: u16
//!   kind:  u32
//!
//!  2 kinds of magic: 
//!   - log
//!   - value

pub struct SectorSpec {
    /// base address of this sector
    pub addr: usize,
    /// bytes in this sector
    pub len:  usize,
}

pub enum ProgramError {
    /// Attempt to move a bit back to it's erased state
    BitUnsetAttempt,
    /// In some cases, writes before erase is limited
    TooManyWrites,
    /// Some flash types forbid writing over data at all, and will emit this error
    WriteAfterWrite,
    /// Many flash devices require that writes be aligned (at least to their size)
    WriteUnaligned,
}

pub enum FlashOpKind {
    Erase { sector: usize },
    Program { sector: usize, addr: usize, data: &[u8] },
}

pub struct FlashOp {
    // XXX: need intrusive list

    kind: FlashOpKind,

    // XXX: in C, we presume that the callback can use `container_of` on the `FlashOp` parameter to
    // obtain a reference to their data. It might be reasonable to provide a field to contain it
    // instead
    //
    // XXX: consider if we can without-cost support a Fn type here instead of a basic function
    callback: fn(Pin<FlashOp>, &mut Flash, Result<(), ProgramError>),
}

/// Abstract flash API
pub trait Flash {
    // XXX: consider using an associated value? or maybe a marker type?
    /// Does this erase to 0 or 1?
    fn erases_to_zero(&self) -> bool;

    fn run_op(&mut self, op: Pin<FlashOp>);

    /// erase a given sector
    //
    // XXX: ASYNC!
    fn erase_sector(&mut self, sector: usize) -> Result<(),()>;

    /// program some piece of a sector
    //
    // XXX: ASYNC! we won't know result until later
    fn program(&mut self, sector: usize, addr: usize, data: &[u8]) -> Result<(), ProgramError>;
}

/// Mtxn - a transactional kv store
pub struct Mtxn<F: Flash> {
    flash: F, 

    //
}
