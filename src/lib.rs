#![no_std]
#![feature(asm)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(compiler_builtins_lib)]
#![feature(panic_implementation)]

extern crate uefi;
use core::fmt::Write;
use uefi::SimpleTextOutput;
use uefi::graphics::{PixelFormat,Pixel};
use core::mem;
use core::panic::PanicInfo;
mod baselib;
use baselib::{graphic::RGB,serial::SerialWriter};

fn show_memmap(memory_map:&uefi::MemoryDescriptor,memory_map_size:usize,descriptor_size:usize){
    let mut w = SerialWriter::new();
    let memory_maps = memory_map as * const uefi::MemoryDescriptor as usize;

    for i in 0..memory_map_size/descriptor_size{
        let memory_map=unsafe{
            mem::transmute::<*const uefi::MemoryDescriptor, &'static uefi::MemoryDescriptor>
                ((memory_maps+(i*descriptor_size))as *const uefi::MemoryDescriptor)
        };
        let pages=memory_map.number_of_pages();
        if pages != 0{
            let phys_start = memory_map.physical_start();
            let virt_start = memory_map.virtual_start();
            write!(w,"{:?}",memory_map.type_of_memory());
            write!(w,"|p:{:x}",phys_start);
            write!(w,"|v:{:x}",virt_start);
            write!(w,"|pages:{},({} kb)",pages,pages*4);
            write!(w,"|attr:{:x}\r\n",memory_map.attribute());
        }else{
            break;
        }
    }
    write!(w,"memmap size : {}\r\n",memory_map_size);
    write!(w,"descriptor size : {}",descriptor_size);
}

extern{
    fn io_hlt();
}

#[allow(unreachable_code)]
#[no_mangle]
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub extern "win64" fn efi_main(hdl: uefi::Handle, sys: uefi::SystemTable) -> uefi::Status {
    uefi::initialize_lib(&hdl, &sys);

    let bs = uefi::get_system_table().boot_services();
    let rs = uefi::get_system_table().runtime_services();
    let gop = uefi::graphics::GraphicsOutputProtocol::new().unwrap();

    let mut mode: u32 = 0;
    let mut w = SerialWriter::new();
    for i in 0..gop.get_max_mode() {
        let info = gop.query_mode(i).unwrap();
        write!(w,"w:{} h:{} pix_fmt:{:?}\r\n",info.horizontal_resolution,info.vertical_resolution,info.pixel_format);
        if info.pixel_format != PixelFormat::RedGreenBlue
            && info.pixel_format != PixelFormat::BlueGreenRed { 
                continue;
            }
        if info.horizontal_resolution == 1920 && info.vertical_resolution == 1080 { mode = i; break; }
        mode = i;
    };

    uefi::get_system_table().console().write("UEFI vendor: ");
    uefi::get_system_table().console().write_raw(uefi::get_system_table().vendor());
    uefi::get_system_table().console().write("\n\r\n\r");
    
    let tm = rs.get_time().unwrap();
    let info = gop.query_mode(mode).unwrap();
    let resolution_w : usize = info.horizontal_resolution as usize;
    let resolution_h : usize = info.vertical_resolution as usize;

    let AREA : usize = resolution_h * resolution_w;

    //　適当に描画
    let bitmap = bs.allocate_pool::<Pixel>(mem::size_of::<Pixel>() * AREA).unwrap();
/*
    let mut c = RGB::new();
//    loop {
        for x in 0..255{
            c.hsv2rgb(x,128,255);
            let px = Pixel::new(c.r,c.g,c.b);

            let mut count = 0;
            while count < AREA {
                unsafe{
                    *bitmap.offset(count as isize) = px.clone();
                };
                count += 1;
            }
            gop.draw(bitmap, 0, 0, resolution_w, resolution_h);
            bs.stall(1000);
        }
        */
//    }

    let (memory_map, memory_map_size, map_key, descriptor_size, descriptor_version) = uefi::lib_memory_map();
    bs.exit_boot_services(&hdl, &map_key);

        let mut c = RGB::new();
//    loop {
        for x in 0..255{
            c.hsv2rgb(x,128,255);
            let px = Pixel::new(c.r,c.g,c.b);

            let mut count = 0;
            while count < AREA {
                unsafe{
                    *bitmap.offset(count as isize) = px.clone();
                };
                count += 1;
            }
            gop.draw(bitmap, 0, 0, resolution_w, resolution_h);
            bs.stall(1000);
        }
    rs.set_virtual_address_map(&memory_map_size, &descriptor_size, &descriptor_version, memory_map);
    show_memmap(memory_map,memory_map_size,descriptor_size);
    loop {
        unsafe{io_hlt();}
    }
    uefi::Status::Success
}

#[no_mangle]
pub fn abort() -> ! {
    loop {}
}

#[no_mangle]
pub fn breakpoint() -> ! {
    loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
pub extern "C" fn _Unwind_Resume() -> ! {
    loop {}
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {}

#[panic_implementation]
#[no_mangle]
pub extern fn rust_begin_panic(info:&PanicInfo) -> ! {
    let mut w = SerialWriter::new();
    write!(w,"{}",info);
    loop {unsafe{io_hlt();}}
}
