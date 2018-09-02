use memory::frame_allocator::PageMemoryManager;
use core::fmt::Write;
use baselib::{graphic::RGB,serial::SerialWriter};
use memory::{paging,FrameAllocator};
use boot::BootInfo;

pub fn kernel_main(bootinfo:BootInfo){
    let mut pmm = unsafe{PageMemoryManager::get_instance()};    //ページアロケータのインスタンスを得る
    let mut w = SerialWriter::new();
    let mega=1024*1024;
    // ページアロケータが持つメタ情報の出力とページメモリの確保と開放
    write!(w,"system memory info: {} MB / {} MB(physical memory)\r\n",pmm.get_freearea_bytes()/mega,pmm.get_memory_capacity()/mega);
    let frame=pmm.allocate_frame().unwrap();
    write!(w,"alloc page area : {:?}({} pages)\r\n",frame.page_frame.memory(),frame.page_frame.pages());
    let frame1=pmm.allocate_frame().unwrap();
    write!(w,"alloc page area : {:?}({} pages)\r\n",frame1.page_frame.memory(),frame.page_frame.pages());
    pmm.deallocate_frame(frame);
    pmm.deallocate_frame(frame1);
    let frame=pmm.allocate_frame().unwrap();
    write!(w,"alloc page area : {:?}({} pages)\r\n",frame.page_frame.memory(),frame.page_frame.pages());
    let base=bootinfo.clone().graphic.unwrap().framebuffer_base;
    let size=bootinfo.clone().graphic.unwrap().framebuffer_size;
    write!(w,"frame buffer info : {:x}-{:x}",(base as usize),(base as usize)+size);
}
