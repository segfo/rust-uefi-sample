#![no_std]
#![feature(asm)]
#![feature(intrinsics)]
#![feature(lang_items)]
#![feature(compiler_builtins_lib)]

extern crate uefi;
extern crate rlibc;
extern crate compiler_builtins;

use uefi::SimpleTextOutput;
use uefi::graphics::{PixelFormat,Pixel};
use core::mem;
use core::fmt::Write;

struct RGB{
    r:u8,
    g:u8,
    b:u8
}

impl RGB{
    fn new()->Self{
        Self{
            r:0,
            g:0,
            b:0
        }
    }

    fn hsv2rgb(&mut self,h:u8,s:u8,v:u8){
        let h = h as f64 /255.0;
        let s = s as f64 /255.0;
        let v = v as f64 /255.0;
        let mut r = v;
        let mut g = v;
        let mut b = v;

        let mut h=h;
        if s > 0.0 {
            h *= 6.0;
            let  i = h as u32;
            let f = h - (i as f64);
            match i{
                0=>{g *= 1.0 - s * (1.0 - f); b *= 1.0 - s;},
                1=>{r *= 1.0 - s * f; b *= 1.0 - s;},
                2=>{r *= 1.0 - s; b *= 1.0 - s * (1.0 - f);},
                3=>{r *= 1.0 - s;g *= 1.0 - s * f;},
                4=>{r *= 1.0 - s * (1.0 - f);g *= 1.0 - s;},
                5=>{g *= 1.0 - s;b *= 1.0 - s * f;},
                _=>{}
            }
        }
        self.r=(r*255.0) as u8;
        self.g=(g*255.0) as u8;
        self.b=(b*255.0) as u8;
    }
}

pub struct Writer;

impl core::fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uefi::get_system_table().console().write(s);
        Ok(())
    }
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
    for i in 0..gop.get_max_mode() {
        let info = gop.query_mode(i).unwrap();
        if info.pixel_format != PixelFormat::RedGreenBlue
            && info.pixel_format != PixelFormat::BlueGreenRed { continue; }
        if info.horizontal_resolution > 1920 && info.vertical_resolution > 1080 { continue; }
        if info.horizontal_resolution == 1920 && info.vertical_resolution == 1080 { mode = i; break; }
        mode = i;
    };

    gop.set_mode(mode);

    uefi::get_system_table().console().write("UEFI vendor: ");
    uefi::get_system_table().console().write_raw(uefi::get_system_table().vendor());
    uefi::get_system_table().console().write("\n\r");

    let tm = rs.get_time().unwrap();
    let info = gop.query_mode(mode).unwrap();
    let resolutin_w : usize = info.horizontal_resolution as usize;
    let resolutin_h : usize = info.vertical_resolution as usize;
    const AREA : usize = 800 * 600;
    
    let bitmap = bs.allocate_pool::<Pixel>(mem::size_of::<Pixel>() * AREA).unwrap();
    let mut c = RGB::new();
    loop {
        for x in 0..255{
            c.hsv2rgb(x,255,255);
            let px = Pixel::new(c.r,c.g,c.b);
            
            let mut count = 0;
            while count < AREA {
                unsafe{
                    *bitmap.offset(count as isize) = px.clone();
                };
                count += 1;
            }
            gop.draw(bitmap, resolutin_w/2-400, resolutin_h/2-300, 800, 600);
            bs.stall(100000);
        }
    }

    let (memory_map, memory_map_size, map_key, descriptor_size, descriptor_version) = uefi::lib_memory_map();
    bs.exit_boot_services(&hdl, &map_key);
    rs.set_virtual_address_map(&memory_map_size, &descriptor_size, &descriptor_version, memory_map);

    loop {
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

#[lang = "panic_fmt"]
#[no_mangle]
pub extern fn rust_begin_panic(_msg: core::fmt::Arguments,
                               _file: &'static str,
                               _line: u32) -> ! {
    uefi::get_system_table().console().write("panic!");
    loop {}
}
