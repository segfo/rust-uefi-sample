#[derive(Clone)]
pub struct BootInfo{
    pub graphic:Option<Graphics>
}
#[derive(Clone)]
pub struct Graphics{
    pub framebuffer_base: *mut u8,
    pub framebuffer_size:usize
}

pub struct BootInfoBuilder{
    info:BootInfo
}

impl BootInfoBuilder{
    pub fn set_graphics(mut self,graphic_info:Graphics)->Self{
        self.info.graphic=Some(graphic_info);
        self
    }
    pub fn build(self)->BootInfo{
        self.info
    }
}

impl BootInfo{
    pub fn new()->BootInfoBuilder{
        BootInfoBuilder{
            info:BootInfo{
                graphic:None,
            }
        }
    }
}