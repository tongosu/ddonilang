use std::fs;
use std::path::Path;

use crate::core::bogae::{
    canonicalize_color, decode_drawlist_detbin_any, encode_drawlist_detbin,
    encode_drawlist_detbin_bdl2, hash_drawlist_detbin, load_css4_pack, BogaeCmd, BogaeCodec,
    BogaeDrawListV1,
};

pub struct BogaeEditOptions<'a> {
    pub input: &'a Path,
    pub output: &'a Path,
    pub dx: i32,
    pub dy: i32,
    pub color: Option<&'a str>,
}

pub fn run_edit(options: BogaeEditOptions<'_>) -> Result<(), String> {
    let bytes = fs::read(options.input).map_err(|e| e.to_string())?;
    let (drawlist, codec) = decode_drawlist_detbin_any(&bytes).map_err(|err| err.message())?;
    let override_color = if let Some(color_text) = options.color {
        let pack = load_css4_pack().ok();
        Some(canonicalize_color(color_text, pack.as_ref()).map_err(|err| err.message())?)
    } else {
        None
    };
    let edited = apply_edit(&drawlist, options.dx, options.dy, override_color);
    let detbin = match codec {
        BogaeCodec::Bdl1 => encode_drawlist_detbin(&edited),
        BogaeCodec::Bdl2 => encode_drawlist_detbin_bdl2(&edited),
    };
    fs::write(options.output, &detbin).map_err(|e| e.to_string())?;
    let hash = hash_drawlist_detbin(&detbin);
    println!("bogae_hash={}", hash);
    Ok(())
}

fn apply_edit(
    drawlist: &BogaeDrawListV1,
    dx: i32,
    dy: i32,
    color: Option<crate::core::bogae::Rgba>,
) -> BogaeDrawListV1 {
    let dx = dx as f32;
    let dy = dy as f32;
    let cmds = drawlist
        .cmds
        .iter()
        .map(|cmd| edit_cmd(cmd, dx, dy, color))
        .collect();
    BogaeDrawListV1 {
        width_px: drawlist.width_px,
        height_px: drawlist.height_px,
        cmds,
    }
}

fn edit_cmd(cmd: &BogaeCmd, dx: f32, dy: f32, color: Option<crate::core::bogae::Rgba>) -> BogaeCmd {
    match cmd {
        BogaeCmd::Clear { color: clear, aa } => BogaeCmd::Clear {
            color: *clear,
            aa: *aa,
        },
        BogaeCmd::RectFill {
            x,
            y,
            w,
            h,
            color: c,
            aa,
        } => BogaeCmd::RectFill {
            x: x + dx,
            y: y + dy,
            w: *w,
            h: *h,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::RectStroke {
            x,
            y,
            w,
            h,
            thickness,
            color: c,
            aa,
        } => BogaeCmd::RectStroke {
            x: x + dx,
            y: y + dy,
            w: *w,
            h: *h,
            thickness: *thickness,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::Line {
            x1,
            y1,
            x2,
            y2,
            thickness,
            color: c,
            aa,
        } => BogaeCmd::Line {
            x1: x1 + dx,
            y1: y1 + dy,
            x2: x2 + dx,
            y2: y2 + dy,
            thickness: *thickness,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::Text {
            x,
            y,
            size_px,
            color: c,
            text,
            aa,
        } => BogaeCmd::Text {
            x: x + dx,
            y: y + dy,
            size_px: *size_px,
            color: color.unwrap_or(*c),
            text: text.clone(),
            aa: *aa,
        },
        BogaeCmd::Sprite {
            x,
            y,
            w,
            h,
            tint,
            asset,
            aa,
        } => BogaeCmd::Sprite {
            x: x + dx,
            y: y + dy,
            w: *w,
            h: *h,
            tint: color.unwrap_or(*tint),
            asset: asset.clone(),
            aa: *aa,
        },
        BogaeCmd::CircleFill {
            cx,
            cy,
            r,
            color: c,
            aa,
        } => BogaeCmd::CircleFill {
            cx: cx + dx,
            cy: cy + dy,
            r: *r,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::CircleStroke {
            cx,
            cy,
            r,
            thickness,
            color: c,
            aa,
        } => BogaeCmd::CircleStroke {
            cx: cx + dx,
            cy: cy + dy,
            r: *r,
            thickness: *thickness,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::ArcStroke {
            cx,
            cy,
            r,
            start_turn,
            sweep_turn,
            thickness,
            color: c,
            aa,
        } => BogaeCmd::ArcStroke {
            cx: cx + dx,
            cy: cy + dy,
            r: *r,
            start_turn: *start_turn,
            sweep_turn: *sweep_turn,
            thickness: *thickness,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
        BogaeCmd::CurveCubicStroke {
            p0x,
            p0y,
            p1x,
            p1y,
            p2x,
            p2y,
            p3x,
            p3y,
            thickness,
            color: c,
            aa,
        } => BogaeCmd::CurveCubicStroke {
            p0x: p0x + dx,
            p0y: p0y + dy,
            p1x: p1x + dx,
            p1y: p1y + dy,
            p2x: p2x + dx,
            p2y: p2y + dy,
            p3x: p3x + dx,
            p3y: p3y + dy,
            thickness: *thickness,
            color: color.unwrap_or(*c),
            aa: *aa,
        },
    }
}
