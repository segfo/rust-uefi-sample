pub mod frame_allocator;
pub mod paging;
use self::frame_allocator::PageMemory;
use self::paging::PhysicalAddress;
use self::frame_allocator::memtranse;
pub const PAGE_SIZE: usize = 4096;

pub trait FrameAllocator {
    fn allocate_frame(&mut self) -> Option<Frame>;
    fn deallocate_frame(&mut self, frame: Frame);
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frame {
    pub page_frame: PageMemory,
}

impl Frame {
    fn containing_address(address:usize)->Frame{
        Frame{ page_frame:memtranse(address,PAGE_SIZE) }
    }
    fn start_address(&self) -> PhysicalAddress {
        self.page_frame.memory() as usize
    }
}
