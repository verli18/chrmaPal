use egui_macroquad::egui::{Color32, Slider};
use macroquad::prelude::*;

mod color_fun;
mod curves;
use curves::SaturationCurve;

struct Palette {
    num_colors: usize,
    color1: Color32,
    color2: Color32,
    blending_mode: usize,
    hue_shift: f32,
    sat_curve: SaturationCurve,
    chroma_boost: f32,
}

impl Palette {
    fn new() -> Self {
        Self {
            num_colors: 8,
            color1: Color32::from_rgb(30, 60, 120),
            color2: Color32::from_rgb(255, 220, 100),
            blending_mode: 2,
            hue_shift: 0.0,
            sat_curve: SaturationCurve::Midtones,
            chroma_boost: 1.0,
        }
    }
}

struct swatches {
    colors: Vec<Color32>
}

impl swatches {
    fn new() -> Self {
        Self {
            colors: Vec::new()
        }
    }
}

#[macroquad::main("egui with macroquad")]
async fn main() {
    let pal_range = (screen_width()/8.0, screen_width()/8.0 * 7.0);
    let pal_vertical_range = (screen_height()/4.0, screen_height()/4.0 * 3.0);
    
    let mut palette = Palette::new();
    
    loop {
        clear_background(Color::new(79.0/255.0, 92.0/255.0, 98.0/255.0, 1.0));

        egui_macroquad::ui(|egui_ctx| {
            egui_macroquad::egui::Window::new("Palette Generator")
                .show(egui_ctx, |ui| {
                    ui.add(Slider::new(&mut palette.num_colors, 2..=16).text("Number of colors"));
                    
                    ui.horizontal(|ui| {
                        ui.label("Dark color:");
                        ui.color_edit_button_srgba(&mut palette.color1);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Light color:");
                        ui.color_edit_button_srgba(&mut palette.color2);
                    });

                    ui.separator();
                    ui.label("Blending Mode");
                    egui_macroquad::egui::ComboBox::from_label("Mode")
                        .selected_text(match palette.blending_mode {
                            0 => "RGB (basic)",
                            1 => "OkLab (perceptual)",
                            2 => "OkLCh (recommended)",
                            _ => "Unknown"
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut palette.blending_mode, 0, "RGB (basic)");
                            ui.selectable_value(&mut palette.blending_mode, 1, "OkLab (perceptual)");
                            ui.selectable_value(&mut palette.blending_mode, 2, "OkLCh (recommended)");
                        });

                    ui.separator();
                    ui.label("Hue Rotation");
                    ui.add(Slider::new(&mut palette.hue_shift, -180.0..=180.0)
                        .text("degrees")
                        .suffix("°"));
                    ui.label("<- cold | warm ->");
                    
                    ui.separator();
                    ui.label("Chroma / Saturation");
                    let sat_curves = SaturationCurve::all();
                    egui_macroquad::egui::ComboBox::from_label("Curve shape")
                        .selected_text(palette.sat_curve.name())
                        .show_ui(ui, |ui| {
                            for curve in sat_curves.iter() {
                                ui.selectable_value(&mut palette.sat_curve, *curve, curve.name());
                            }
                        });
                    ui.add(Slider::new(&mut palette.chroma_boost, 0.0..=2.0)
                        .text("Chroma boost"));
                });
        });

        // Draw palette
        let rectangle_length = (pal_range.1 - pal_range.0) / palette.num_colors as f32;

        for i in 0..palette.num_colors {
            let t = if palette.num_colors > 1 { i as f32 / (palette.num_colors - 1) as f32 } else { 0.5 };

            let rect_col = match palette.blending_mode {
                0 => color_fun::lerp_color(palette.color1, palette.color2, t),
                1 => color_fun::lerp_color_oklab(palette.color1, palette.color2, t),
                2 => color_fun::generate_ramp_color_oklch(
                    palette.color1,
                    palette.color2,
                    t,
                    palette.hue_shift,
                    palette.sat_curve,
                    palette.chroma_boost
                ),
                _ => color_fun::lerp_color(palette.color1, palette.color2, t),

            
            };
            
            let macroquad_color = Color::new(
                rect_col.r() as f32 / 255.0,
                rect_col.g() as f32 / 255.0,
                rect_col.b() as f32 / 255.0,
                1.0,
            );
            draw_rectangle(
                pal_range.0 + i as f32 * rectangle_length,
                pal_vertical_range.0,
                rectangle_length,
                pal_vertical_range.1 - pal_vertical_range.0,
                macroquad_color,
            );

            if (rect_col.r() as u32 + rect_col.g() as u32 + rect_col.b() as u32) < 382 {
                draw_text(
                    &format!("#{:02X}{:02X}{:02X}", rect_col.r(), rect_col.g(), rect_col.b()),
                    pal_range.0 + i as f32 * rectangle_length + 8.0,
                    pal_vertical_range.0 + 24.0,
                    16.0,
                    WHITE,
                );
            } else {
                draw_text(
                    &format!("#{:02X}{:02X}{:02X}", rect_col.r(), rect_col.g(), rect_col.b()),
                    pal_range.0 + i as f32 * rectangle_length + 8.0,
                    pal_vertical_range.0 + 24.0,
                    16.0,
                    BLACK,
                );
            }
        }
        
        egui_macroquad::draw();
        next_frame().await;
    }
}