mod boot;
mod cpu;
mod dmg;
mod io;

use std::path::Path;

use cpu::Cpu;
use io::{
    cartridge::Cartridge,
    ppu::{LCD_HEIGHT, LCD_WIDTH},
};
use log::error;
use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SCALE: f32 = 4.0;

fn main() {
    env_logger::init();

    let mut cpu =
        Cpu::new(Cartridge::from_path(Path::new("./tetris.gb")).expect("unable to load rom"));
    // loop {
    //     cpu.machine_cycle();
    // }

    let surface_size = LogicalSize::new(LCD_WIDTH as f32, LCD_HEIGHT as f32);
    let scaled_surface_size =
        LogicalSize::new(surface_size.width * SCALE, surface_size.height * SCALE);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Rust GameBoy")
        .with_inner_size(scaled_surface_size)
        .with_min_inner_size(surface_size)
        .build(&event_loop)
        .unwrap();

    let window_size = window.inner_size();

    let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
    let mut pixels = Pixels::new(LCD_WIDTH as u32, LCD_HEIGHT as u32, surface_texture).unwrap();

    let mut last_frame = 0;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            while last_frame == cpu.bus.ppu.frame {
                for _ in 0..2048 {
                    cpu.machine_cycle();
                }
            }
            last_frame = cpu.bus.ppu.frame;
            let frame = pixels.frame_mut();
            for (c, pix) in cpu
                .bus
                .ppu
                .framebuffer
                .iter()
                .zip(frame.chunks_exact_mut(4))
            {
                let output: [u8; 4] = [c.r, c.g, c.b, 255];
                pix.copy_from_slice(&output);
            }
            if let Err(e) = pixels.render() {
                error!("unable to render: {}", e);
                *control_flow = ControlFlow::Exit;
            }

            window.request_redraw();
        }
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => pixels.resize_surface(size.width, size.height).unwrap(),
            _ => {}
        },
        _ => {}
    });
}
