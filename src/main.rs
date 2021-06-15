use another_gl as gl;
use nalgebra as na;
use egui_sdl2_gl as egui_backend;
use egui_backend::egui;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::GLProfile;
use std::path::Path;
use std::rc::Rc;
use egui_backend::egui::{vec2, Pos2, Rect};
use anyhow::anyhow;

#[macro_use] extern crate render_gl_derive;

use crate::fonts::install_fonts;
use crate::resources::Resources;

pub mod triangle;
pub mod render_gl;
pub mod resources;
pub mod fonts;
mod model;


const SCREEN_WIDTH: u32 = 1600;
const SCREEN_HEIGHT: u32 = 1200;

fn main()-> Result<(),anyhow::Error> {
    let res =
        Resources::from_relative_exe_path(Path::new("assets"))?;
    let sdl_context = sdl2::init()
      .map_err(|msg| anyhow!("Sdl2 初始化失败 {}",msg))?;
    let video_subsystem = sdl_context.video()
      .map_err(|msg| anyhow!("Video subsystem获取失败 {}", msg))?;

    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(GLProfile::Core);
    // egui支持下限为320
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window(
            "演示: Egui  (SDL2 + GL后端)",
            SCREEN_WIDTH,
            SCREEN_HEIGHT,
        )
        .resizable()
        .position_centered()
        .opengl()
        .build()?;

    let _ctx = window.gl_create_context()
        .map_err(|msg| anyhow!("创建GL上下文失败: {}",msg))?;

    let mut painter = egui_backend::Painter::new(&video_subsystem, SCREEN_WIDTH, SCREEN_HEIGHT);
    let mut egui_ctx = egui::CtxRef::default();
    // 安装中文字体
    install_fonts(&egui_ctx);

   let gl: Rc<gl::Gl> = Rc::new(
      gl::Gl::load_with(|s| {
         video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
      })
   );
    // UI缩放，将影响<设备像素密度>
    let ui_zoom = 5f32;
   // 获取<事件轮询器>，这是在SDL2中处理事件的传统方式
    let mut event_pump = sdl_context.event_pump()
        .map_err(|msg| anyhow!("事件轮询器获取失败: {}",msg))?;

    //  fixme:查明换算公式
    let native_pixels_per_point =  (ui_zoom * 96f32) / video_subsystem.display_dpi(0).unwrap().0;

    let (width, height) = window.size();

    // egui输入状态，主要影响ui的大小以及保证输入位置的准确性
    let mut egui_input_state = egui_backend::EguiInputState::new(egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::new(0f32, 0f32),
            vec2(width as f32, height as f32) / native_pixels_per_point,
        )),
        pixels_per_point: Some(native_pixels_per_point),
        ..Default::default()
    });
    let mut viewport =
        render_gl::Viewport::for_window(SCREEN_WIDTH as i32,SCREEN_HEIGHT as i32);
    viewport.refresh(&gl);

    let mut color_buffer =
        render_gl::ColorBuffer::from_color(na::Vector3::new(0.3, 0.6, 0.3));

    let mut color1_r: f32 = 1f32;
    let mut color1_g: f32 = 1f32;
    let mut color1_b: f32 = 1f32;

    let mut test_str: String = "用于输入的文本框。剪切、复制、粘贴命令可用".to_owned();

    let triangle = triangle::Triangle::new(&res, &gl)?;
    let mut quit = false;

    'running: loop {
        egui_ctx.begin_frame(egui_input_state.input.take());
        // 每次渲染都会丢失视窗变换的数据，推测是egui的行为
        viewport.refresh(&gl);
        // 每次渲染都会丢失设备像素的数据，推测是egui的行为
        egui_input_state.input.pixels_per_point = Some(native_pixels_per_point);

        color_buffer.clear(&gl);
        unsafe {
            gl.Clear(gl::COLOR_BUFFER_BIT);
            // fixme 深度测试无法使用
            // gl.Enable(gl::DEPTH_TEST);
            gl.Enable(gl::BLEND);
        }

        // 自定义的OpenGL渲染部分
        // TODO: 使用FrameBuffer
        triangle.render(&gl);

        // egui的UI定义部分
        egui::Window::new("Egui with SDL2 and GL").show(&egui_ctx, |ui| {
            ui.separator();
            ui.label("这是egui的演示用文本");
            ui.label(" ");
            ui.text_edit_multiline(&mut test_str);
            ui.label(" ");
            ui.add(egui::Slider::new(&mut color1_r, 0.0..=1.0).text("Color1-R"));
            ui.add(egui::Slider::new(&mut color1_g, 0.0..=1.0).text("Color1-G"));
            ui.add(egui::Slider::new(&mut color1_b, 0.0..=1.0).text("Color1-B"));
            ui.label(" ");
            if ui.button("Quit").clicked() {
                quit = true;
            }
        });
        let color1 = na::Vector3::new(color1_r, color1_g, color1_b);
        color_buffer.update_color(color1);
        // egui前端完成渲染，生成后端无关的<绘制指令>
        let (_, paint_cmds) = egui_ctx.end_frame();
        // 将<绘制指令>转化为<网格>(Mesh),即几何体集合
        let paint_jobs = egui_ctx.tessellate(paint_cmds);
        // 由后端完成实际的绘制
        painter.paint_jobs(
            None,
            paint_jobs,
            &egui_ctx.texture(),
            native_pixels_per_point,
        );

        // 用OpenGL渲染结果更新窗口
        // macOS 上，frame buffer必须重新绑定到0上，否则。。。
        window.gl_swap_window();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::Window {
                    win_event: WindowEvent::Resized(w,h),
                    ..
                 } => {
                    viewport.update_size(w, h);
                    egui_input_state.input.screen_rect = Some(Rect::from_min_size(
                        Pos2::new(viewport.x as f32, viewport.y as f32),
                        vec2(viewport.w as f32, viewport.h as f32) / native_pixels_per_point,
                    ))
                 },
                _ => {
                    // 将捕捉的输入交给egui
                    egui_backend::input_to_egui(event, &mut egui_input_state);
                }
            }
        }
        if quit { break; }
    }
    Ok(())
}
