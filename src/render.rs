//! Drawing a picture into an image (a "render target"), so it can be cut into puzzle pieces
//! or drawn as a solid-color shadow.

use crate::pictures::{self, Picture};
use macroquad::miniquad::{BlendFactor, BlendState, BlendValue, Equation};
use macroquad::prelude::*;

/// Draw `pic` into a square image `px` pixels wide, on a see-through background.
/// Keep the returned `RenderTarget` for as long as you draw its `.texture`: when the
/// render target is dropped, macroquad deletes the image too.
pub fn picture_texture(pic: Picture, px: u32) -> RenderTarget {
    picture_texture_ex(pic, px, false)
}

/// Like `picture_texture`, but with `backdrop` the picture sits on a soft color blend
/// (pink top-left, yellow top-right, green bottom-left, blue bottom-right). Used for
/// puzzles, so even pieces with no picture on them show where they belong.
pub fn picture_texture_ex(pic: Picture, px: u32, backdrop: bool) -> RenderTarget {
    let target = render_target(px, px);
    target.texture.set_filter(FilterMode::Linear);
    let size = px as f32;
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, size, size));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    clear_background(Color::new(1.0, 1.0, 1.0, 0.0));
    if backdrop {
        let corners = [
            Color::from_rgba(255, 214, 230, 255), // top-left: pink
            Color::from_rgba(255, 240, 190, 255), // top-right: yellow
            Color::from_rgba(205, 240, 205, 255), // bottom-left: green
            Color::from_rgba(200, 225, 255, 255), // bottom-right: blue
        ];
        let steps = 24;
        let cell = size / steps as f32;
        for gy in 0..steps {
            for gx in 0..steps {
                let (x, y) = ((gx as f32 + 0.5) / steps as f32, (gy as f32 + 0.5) / steps as f32);
                let top = crate::art::mix(corners[0], corners[1], x);
                let bottom = crate::art::mix(corners[2], corners[3], x);
                let color = crate::art::mix(top, bottom, y);
                draw_rectangle(gx as f32 * cell, gy as f32 * cell, cell + 1.0, cell + 1.0, color);
            }
        }
    }
    let scale = if backdrop { 0.46 } else { 0.42 };
    pictures::draw(pic, vec2(size / 2.0, size / 2.0), size * scale);
    set_default_camera();
    target
}

/// Draw anything into a square see-through image: `draw` gets the center and a size.
/// (Used for coloring-page outlines of shapes and pictures.)
pub fn shape_texture(px: u32, draw: impl Fn(Vec2, f32)) -> RenderTarget {
    let target = render_target(px, px);
    target.texture.set_filter(FilterMode::Linear);
    let size = px as f32;
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, size, size));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    clear_background(Color::new(1.0, 1.0, 1.0, 0.0));
    draw(vec2(size / 2.0, size / 2.0), size * 0.44);
    set_default_camera();
    target
}

/// A blank white canvas image to paint on, `w` x `h` pixels.
pub fn canvas(w: u32, h: u32) -> RenderTarget {
    let target = render_target(w.max(1), h.max(1));
    target.texture.set_filter(FilterMode::Linear);
    clear_canvas(&target);
    target
}

/// Wipe a canvas back to white.
pub fn clear_canvas(target: &RenderTarget) {
    with_canvas(target, || clear_background(WHITE));
}

/// Run some drawing code that paints onto `target` instead of the screen.
/// Coordinates are the canvas's own pixels (0,0 is its top-left corner).
pub fn with_canvas(target: &RenderTarget, draw: impl FnOnce()) {
    let (w, h) = (target.texture.width(), target.texture.height());
    let mut camera = Camera2D::from_display_rect(Rect::new(0.0, 0.0, w, h));
    camera.render_target = Some(target.clone());
    set_camera(&camera);
    draw();
    set_default_camera();
}

/// Draw (part of) a texture made by `picture_texture` into `dest`.
/// `source` is the part to draw, in 0..1 units (the whole image is 0,0 to 1,1).
pub fn draw_picture(tex: &Texture2D, dest: Rect, source: Rect, tint: Color) {
    let (w, h) = (tex.width(), tex.height());
    // Render-target images are stored upside down, so the top of the picture is at the
    // bottom of the image: pick the part to draw from the flipped position.
    let flipped_y = 1.0 - source.y - source.h;
    draw_texture_ex(
        tex,
        dest.x,
        dest.y,
        tint,
        DrawTextureParams {
            dest_size: Some(vec2(dest.w, dest.h)),
            source: Some(Rect::new(source.x * w, flipped_y * h, source.w * w, source.h * h)),
            // Images drawn with a camera come out upside down; flip them back.
            flip_y: true,
            ..Default::default()
        },
    );
}

/// A tiny shader program that paints every visible pixel of a texture in one solid color
/// (the tint), keeping only the shape. That's exactly a shadow!
pub fn shadow_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;
    const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
void main() {
    // Anything at least half visible counts as solid shadow (so see-through details like
    // rosy cheeks don't show up), with a soft edge for smooth outlines.
    lowp float a = smoothstep(0.0, 0.5, texture2D(Texture, uv).a);
    gl_FragColor = vec4(color.rgb, a * color.a);
}
"#;
    // Blend with what's underneath, so the see-through parts stay see-through.
    let blend = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        MaterialParams {
            pipeline_params: PipelineParams { color_blend: Some(blend), ..Default::default() },
            ..Default::default()
        },
    )
    .ok()
}

/// Draw a picture texture with a special material (a shadow, or an outline).
pub fn draw_with_material(material: &Material, tex: &Texture2D, dest: Rect, color: Color) {
    gl_use_material(material);
    draw_picture(tex, dest, Rect::new(0.0, 0.0, 1.0, 1.0), color);
    gl_use_default_material();
}

/// A shader that draws just the *outline* of a picture's shape: a pixel is part of the line
/// if the shape is solid there but some nearby pixel is empty. Turns any picture into a
/// coloring-page outline. (Made for 512-pixel images: `STEP` is about 3 pixels.)
pub fn outline_material() -> Option<Material> {
    const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0 / 255.0;
    uv = texcoord;
}
"#;
    const FRAGMENT: &str = r#"#version 100
precision mediump float;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform sampler2D Texture;
const float STEP = 3.5 / 512.0;
float solid(vec2 p) { return step(0.5, texture2D(Texture, p).a); }
void main() {
    float here = solid(uv);
    float least = 1.0;
    for (int i = 0; i < 8; i++) {
        float a = float(i) * 0.785398;
        least = min(least, solid(uv + vec2(cos(a), sin(a)) * STEP));
    }
    // On the edge: solid here, but something nearby is empty (or we're at the very border).
    float edge = here * (1.0 - least);
    gl_FragColor = vec4(color.rgb, edge * color.a);
}
"#;
    let blend = BlendState::new(
        Equation::Add,
        BlendFactor::Value(BlendValue::SourceAlpha),
        BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
    );
    load_material(
        ShaderSource::Glsl { vertex: VERTEX, fragment: FRAGMENT },
        MaterialParams {
            pipeline_params: PipelineParams { color_blend: Some(blend), ..Default::default() },
            ..Default::default()
        },
    )
    .ok()
}
