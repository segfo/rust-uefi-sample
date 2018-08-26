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
mod memory;
use memory::{PAGE_SIZE};
use memory::frame_allocator::{memtranse,PageMemoryManager};

fn show_memmap(memory_map:&uefi::MemoryDescriptor,memory_map_size:usize,descriptor_size:usize){
    let mut w = SerialWriter::new();
    let memory_maps = memory_map as * const uefi::MemoryDescriptor as usize;
    let available_types=[
        uefi::MemoryType::BootServicesCode,
        uefi::MemoryType::BootServicesData,
        uefi::MemoryType::Conventional];
    let mut memory_size=0;
    let mut available_size=0;

    for i in 0..memory_map_size/descriptor_size{
        let memory_map=unsafe{
            mem::transmute::<*const uefi::MemoryDescriptor, &'static uefi::MemoryDescriptor>
                ((memory_maps+(i*descriptor_size))as *const uefi::MemoryDescriptor)
        };
        let pages=memory_map.number_of_pages();
        let memtype = memory_map.type_of_memory();
        if pages != 0 {
            for available in &available_types{
                if *available==memtype{
                    let phys_start = memory_map.physical_start();
                    let virt_start = memory_map.virtual_start();
                    write!(w,"{:?}",memtype);
                    write!(w,"|p:{:x}",phys_start);
                    write!(w,"-{:x}",phys_start+pages*4096);//virt_start);
                    write!(w,"|pages:{},({} kb)",pages,pages*4);
                    write!(w,"|attr:{:x}\r\n",memory_map.attribute());
                    available_size+=pages*4096;
                }
            }
            memory_size+=pages*4096;
        }else{
            break;
        }
    }
    write!(w,"memmap size : {}\r\n",memory_map_size);
    write!(w,"descriptor size : {}\r\n",descriptor_size);
}

fn init_sysphys_pagememory(pmm:&mut PageMemoryManager,memory_map:&uefi::MemoryDescriptor,memory_map_size:usize,descriptor_size:usize){
    let mut w = SerialWriter::new();
    let memory_maps = memory_map as * const uefi::MemoryDescriptor as usize;
    let available_types=[
        uefi::MemoryType::BootServicesCode,
        uefi::MemoryType::BootServicesData,
        uefi::MemoryType::Conventional];
    let mut memory_size=0;
    let mut available_size=0;

    for i in 0..memory_map_size/descriptor_size{
        let memory_map=unsafe{
            mem::transmute::<*const uefi::MemoryDescriptor, &'static uefi::MemoryDescriptor>
                ((memory_maps+(i*descriptor_size))as *const uefi::MemoryDescriptor)
        };
        let pages = memory_map.number_of_pages();
        let memtype = memory_map.type_of_memory();
        if pages != 0 {
            for available in &available_types{
                if *available==memtype{
                    // 有効なメモリエリアのみここに入る
                    // ページメモリはここで登録
                    unsafe{pmm.free_frames(memtranse(memory_map.physical_start(),pages*PAGE_SIZE));}
                    available_size+=pages*PAGE_SIZE;
                }
            }
            memory_size+=pages*PAGE_SIZE;
        }else{
            break;
        }
    }
    // システムメモリの総容量を登録する
    unsafe{pmm.set_system_memory_total_capacity(memory_size);}
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
    // グラフィックの初期化
    let mut mode: u32 = 0;
    for i in 0..gop.get_max_mode() {
        let info = gop.query_mode(i).unwrap();
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
    //
    let tm = rs.get_time().unwrap();
    let info = gop.query_mode(mode).unwrap();
    let resolution_w : usize = info.horizontal_resolution as usize;
    let resolution_h : usize = info.vertical_resolution as usize;

    let AREA : usize = resolution_h * resolution_w;
    let bitmap = bs.allocate_pool::<Pixel>(mem::size_of::<Pixel>() * AREA).unwrap();

    // メモリ周りの初期化
    // メモリマップを取って、EFI BootServicesを抜ける
    // 抜けたあとに、仮想メモリマップを登録する。このあたりのことは下のコードを参考にした
    // https://github.com/CumulusNetworks/linux-apd/blob/master/drivers/firmware/efi/libstub/fdt.c#L275
    // https://github.com/CumulusNetworks/linux-apd/blob/master/drivers/firmware/efi/libstub/fdt.c#L286
    let (memory_map, memory_map_size, map_key, descriptor_size, descriptor_version) = uefi::lib_memory_map();
    bs.exit_boot_services(&hdl, &map_key);
    rs.set_virtual_address_map(&memory_map_size, &descriptor_size, &descriptor_version, memory_map);
    let mut pmm = PageMemoryManager::new();
    init_sysphys_pagememory(&mut pmm,memory_map,memory_map_size,descriptor_size);
    // カーネルをキック
    run_kernel(&mut pmm);
    loop {
        unsafe{io_hlt();}
    }
    uefi::Status::Success
}

// 他のファイルにぶち込もうな。
fn run_kernel(pmm:&mut PageMemoryManager){
    let mut w = SerialWriter::new();
    let mega=1024*1024;
    write!(w,"system memory info: {} MB / {} MB(physical memory)\r\n",pmm.get_freearea_bytes()/mega,pmm.get_memory_capacity()/mega);
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
