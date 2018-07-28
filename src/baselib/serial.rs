extern crate uefi;
use core;

use uefi::SimpleTextOutput;
use uefi::graphics::{PixelFormat,Pixel};

pub struct SerialWriter;

impl SerialWriter{
    pub fn new()->Self{
        SerialWriter{}
    }
}

impl core::fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        uefi::get_system_table().console().write(s);
        Ok(())
    }
}
