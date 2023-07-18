mod boot;
mod cpu;
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
    event::{ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

const SCALE: f32 = 4.0;

fn main() {
    env_logger::init();

    let mut cpu =
        Cpu::new(Cartridge::from_path(Path::new("./zelda.gb")).expect("unable to load rom"));
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

    let mut right = false;
    let mut left = false;
    let mut up = false;
    let mut down = false;
    let mut a = false;
    let mut b = false;
    let mut select = false;
    let mut start = false;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(_) => {
            cpu.bus.joypad.set_directions(right, left, up, down);
            cpu.bus.joypad.set_actions(a, b, select, start);

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
            WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => {
                if let Some(key) = input.virtual_keycode {
                    let pressed = matches!(input.state, ElementState::Pressed);
                    match key {
                        VirtualKeyCode::Right => right = pressed,
                        VirtualKeyCode::Left => left = pressed,
                        VirtualKeyCode::Up => up = pressed,
                        VirtualKeyCode::Down => down = pressed,
                        VirtualKeyCode::X => a = pressed,
                        VirtualKeyCode::Z => b = pressed,
                        VirtualKeyCode::Back => select = pressed,
                        VirtualKeyCode::Return => start = pressed,
                        _ => {}
                    }
                }
            }
            _ => {}
        },
        _ => {}
    });
}
