use std::io::{self, Write};

use crate::core::bogae::{BogaeCmd, BogaeDrawListV1};
use clap::ValueEnum;

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ConsoleCellAspect {
    Auto,
    #[value(name = "1:1", alias = "1", alias = "1x1")]
    OneToOne,
    #[value(name = "2:1", alias = "2", alias = "2x1")]
    TwoToOne,
}

#[derive(Clone, Copy, Debug)]
pub struct ConsoleRenderConfig {
    pub max_cols: usize,
    pub max_rows: usize,
    pub cell_aspect: ConsoleCellAspect,
    pub grid_cols: Option<usize>,
    pub grid_rows: Option<usize>,
    pub panel_cols: usize,
    pub panel_fill: char,
    pub panel_border: char,
}

impl ConsoleRenderConfig {
    pub fn default() -> Self {
        Self {
            max_cols: 80,
            max_rows: 40,
            cell_aspect: ConsoleCellAspect::Auto,
            grid_cols: None,
            grid_rows: None,
            panel_cols: 0,
            panel_fill: '.',
            panel_border: '|',
        }
    }

    pub fn with_cell_aspect(cell_aspect: ConsoleCellAspect) -> Self {
        Self {
            cell_aspect,
            ..Self::default()
        }
    }
}

pub struct ConsoleLive {
    stdout: io::Stdout,
    config: ConsoleRenderConfig,
    cursor_hidden: bool,
}

impl ConsoleLive {
    pub fn new(config: ConsoleRenderConfig) -> Self {
        Self {
            stdout: io::stdout(),
            config,
            cursor_hidden: false,
        }
    }

    pub fn render(&mut self, madi: u64, drawlist: &BogaeDrawListV1, hash: &str, cmd_count: u32) {
        let mut out = String::new();
        if !self.cursor_hidden {
            out.push_str("\u{1b}[?25l");
            self.cursor_hidden = true;
        }
        out.push_str("\u{1b}[2J\u{1b}[H");
        out.push_str(&format!(
            "madi={} cmd_count={} bogae_hash={}\n",
            madi, cmd_count, hash
        ));
        out.push_str(&render_drawlist_ascii(drawlist, self.config));
        let _ = self.stdout.write_all(out.as_bytes());
        let _ = self.stdout.flush();
    }
}

impl Drop for ConsoleLive {
    fn drop(&mut self) {
        if self.cursor_hidden {
            let _ = self.stdout.write_all(b"\x1b[?25h");
            let _ = self.stdout.flush();
        }
    }
}

pub fn render_drawlist_ascii(drawlist: &BogaeDrawListV1, config: ConsoleRenderConfig) -> String {
    let (cols, rows, scale_x, scale_y) = compute_grid(drawlist, config);
    let mut grid = vec![vec![' '; cols]; rows];
    prefill_panel(&mut grid, config);
    for cmd in &drawlist.cmds {
        match cmd {
            BogaeCmd::Clear { .. } => {
                for row in &mut grid {
                    for cell in row.iter_mut() {
                        *cell = ' ';
                    }
                }
                prefill_panel(&mut grid, config);
            }
            BogaeCmd::RectFill { x, y, w, h, .. } => {
                let x = snap_i32(*x);
                let y = snap_i32(*y);
                let w = snap_i32(*w);
                let h = snap_i32(*h);
                fill_rect(&mut grid, x, y, w, h, scale_x, scale_y, '#');
            }
            BogaeCmd::RectStroke {
                x,
                y,
                w,
                h,
                thickness,
                ..
            } => {
                let x = snap_i32(*x);
                let y = snap_i32(*y);
                let w = snap_i32(*w);
                let h = snap_i32(*h);
                let thickness = snap_i32(*thickness);
                stroke_rect(&mut grid, x, y, w, h, thickness, scale_x, scale_y, '*');
            }
            BogaeCmd::Line { x1, y1, x2, y2, .. } => {
                let x1 = snap_i32(*x1);
                let y1 = snap_i32(*y1);
                let x2 = snap_i32(*x2);
                let y2 = snap_i32(*y2);
                draw_line(&mut grid, x1, y1, x2, y2, scale_x, scale_y, '*');
            }
            BogaeCmd::Text { x, y, text, .. } => {
                let x = snap_i32(*x);
                let y = snap_i32(*y);
                draw_text(&mut grid, x, y, scale_x, scale_y, text, config);
            }
            BogaeCmd::Sprite {
                x,
                y,
                w,
                h,
                tint,
                asset,
                ..
            } => {
                if tint.a == 0 {
                    continue;
                }
                let x = snap_i32(*x);
                let y = snap_i32(*y);
                let w = snap_i32(*w);
                let h = snap_i32(*h);
                let ch = sprite_glyph(&asset.uri);
                fill_rect(&mut grid, x, y, w, h, scale_x, scale_y, ch);
            }
            BogaeCmd::CircleFill { cx, cy, r, .. } => {
                let cx = snap_i32(*cx);
                let cy = snap_i32(*cy);
                let r = snap_i32(*r).abs();
                fill_rect(
                    &mut grid,
                    cx - r,
                    cy - r,
                    r * 2,
                    r * 2,
                    scale_x,
                    scale_y,
                    'o',
                );
            }
            BogaeCmd::CircleStroke {
                cx,
                cy,
                r,
                thickness,
                ..
            } => {
                let cx = snap_i32(*cx);
                let cy = snap_i32(*cy);
                let r = snap_i32(*r).abs();
                let thickness = snap_i32(*thickness);
                stroke_rect(
                    &mut grid,
                    cx - r,
                    cy - r,
                    r * 2,
                    r * 2,
                    thickness,
                    scale_x,
                    scale_y,
                    'o',
                );
            }
            BogaeCmd::ArcStroke {
                cx,
                cy,
                r,
                start_turn,
                sweep_turn,
                ..
            } => {
                let cx = snap_i32(*cx) as f64;
                let cy = snap_i32(*cy) as f64;
                let r = snap_i32(*r).abs() as f64;
                let start = (*start_turn as f64) * std::f64::consts::PI * 2.0;
                let end = start + (*sweep_turn as f64) * std::f64::consts::PI * 2.0;
                let x1 = (cx + r * start.cos()).round() as i32;
                let y1 = (cy + r * start.sin()).round() as i32;
                let x2 = (cx + r * end.cos()).round() as i32;
                let y2 = (cy + r * end.sin()).round() as i32;
                draw_line(&mut grid, x1, y1, x2, y2, scale_x, scale_y, '*');
            }
            BogaeCmd::CurveCubicStroke {
                p0x, p0y, p3x, p3y, ..
            } => {
                let p0x = snap_i32(*p0x);
                let p0y = snap_i32(*p0y);
                let p3x = snap_i32(*p3x);
                let p3y = snap_i32(*p3y);
                draw_line(&mut grid, p0x, p0y, p3x, p3y, scale_x, scale_y, '*');
            }
        }
    }
    draw_panel_border(&mut grid, config);
    let border_bottom = draw_playfield_border(&mut grid, config, drawlist, scale_x, scale_y);
    grid_to_string(&grid, border_bottom)
}

fn sprite_glyph(uri: &str) -> char {
    if let Some(rest) = uri.strip_prefix("sym:tetris.") {
        if let Some(piece) = rest.chars().next() {
            return tetris_glyph(piece);
        }
        return '\u{2588}';
    }
    if let Some(rest) = uri.strip_prefix("sym:") {
        if let Some(ch) = rest.chars().find(|c| c.is_ascii_alphanumeric()) {
            return ch;
        }
    }
    let leaf = uri.rsplit(['/', '.']).next().unwrap_or(uri);
    leaf.chars()
        .find(|c| c.is_ascii_alphanumeric())
        .unwrap_or('@')
}

fn tetris_glyph(piece: char) -> char {
    match piece.to_ascii_uppercase() {
        'I' => '\u{2593}', // dark shade
        'J' => '\u{2592}', // medium shade
        'L' => '\u{2591}', // light shade
        'O' => '\u{2593}', // dark shade
        'S' => '\u{2592}', // medium shade
        'T' => '\u{2591}', // light shade
        'Z' => '\u{2593}', // dark shade
        _ => '\u{2592}',
    }
}

fn draw_text(
    grid: &mut [Vec<char>],
    x: i32,
    y: i32,
    scale_x: i32,
    scale_y: i32,
    text: &str,
    config: ConsoleRenderConfig,
) {
    let (cols, rows) = grid_dims(grid);
    if cols == 0 || rows == 0 {
        return;
    }
    let cy = y.div_euclid(scale_y);
    if cy < 0 || cy >= rows as i32 {
        return;
    }
    let mut cx = x.div_euclid(scale_x);
    if let (Some(grid_cols), Some(_)) = (config.grid_cols, config.grid_rows) {
        if config.panel_cols > 0 {
            let col_scale = grid_col_scale(config);
            let grid_cols_scaled = grid_cols.saturating_mul(col_scale);
            let border_col = grid_cols_scaled
                .saturating_sub(1)
                .min(cols.saturating_sub(1)) as i32;
            let border_x = border_col.saturating_mul(scale_x);
            if x >= border_x && cx == border_col {
                cx += 1;
            }
        }
    }
    let mut wrote = false;
    for ch in text.chars() {
        if !is_allowed_console_char(ch) {
            continue;
        }
        wrote = true;
        if cx >= 0 && (cx as usize) < cols {
            if ch != ' ' {
                grid[cy as usize][cx as usize] = ch;
            }
        }
        cx += 1;
        if cx >= cols as i32 {
            break;
        }
    }
    if !wrote {
        return;
    }
}

fn is_allowed_console_char(ch: char) -> bool {
    if ch.is_ascii() {
        return !ch.is_ascii_control();
    }
    let code = ch as u32;
    if is_control_or_zero_width(code) {
        return false;
    }
    if is_wide(code) {
        return false;
    }
    true
}

fn is_control_or_zero_width(code: u32) -> bool {
    in_range(code, 0x0080, 0x009F) // C1 controls
        || in_range(code, 0x0300, 0x036F) // Combining Diacritical Marks
        || in_range(code, 0x1AB0, 0x1AFF) // Combining Diacritical Marks Extended
        || in_range(code, 0x1DC0, 0x1DFF) // Combining Diacritical Marks Supplement
        || in_range(code, 0x20D0, 0x20FF) // Combining Diacritical Marks for Symbols
        || in_range(code, 0xFE20, 0xFE2F) // Combining Half Marks
        || in_range(code, 0x200B, 0x200F) // Zero-width and directional marks
        || in_range(code, 0x202A, 0x202E) // Bidirectional embedding/override
        || in_range(code, 0x2060, 0x206F) // Word joiner and format controls
        || code == 0xFEFF // BOM/ZWNBSP
}

fn is_wide(code: u32) -> bool {
    // East Asian wide/fullwidth ranges + emoji/pictographs.
    in_range(code, 0x1100, 0x11FF) // Hangul Jamo
        || in_range(code, 0x3130, 0x318F) // Hangul Compatibility Jamo
        || in_range(code, 0xA960, 0xA97F) // Hangul Jamo Extended-A
        || in_range(code, 0xAC00, 0xD7AF) // Hangul Syllables
        || in_range(code, 0xD7B0, 0xD7FF) // Hangul Jamo Extended-B
        || in_range(code, 0x3000, 0x303F) // CJK Symbols and Punctuation
        || in_range(code, 0x3040, 0x309F) // Hiragana
        || in_range(code, 0x30A0, 0x30FF) // Katakana
        || in_range(code, 0x31F0, 0x31FF) // Katakana Phonetic Extensions
        || in_range(code, 0x3100, 0x312F) // Bopomofo
        || in_range(code, 0x31A0, 0x31BF) // Bopomofo Extended
        || in_range(code, 0x2E80, 0x2EFF) // CJK Radicals Supplement
        || in_range(code, 0x2F00, 0x2FDF) // Kangxi Radicals
        || in_range(code, 0x2FF0, 0x2FFF) // Ideographic Description Characters
        || in_range(code, 0x3200, 0x32FF) // Enclosed CJK Letters and Months
        || in_range(code, 0x3300, 0x33FF) // CJK Compatibility
        || in_range(code, 0x3400, 0x4DBF) // CJK Unified Ideographs Extension A
        || in_range(code, 0x4E00, 0x9FFF) // CJK Unified Ideographs
        || in_range(code, 0xA000, 0xA4CF) // Yi Syllables/Radicals
        || in_range(code, 0xF900, 0xFAFF) // CJK Compatibility Ideographs
        || in_range(code, 0xFE30, 0xFE4F) // CJK Compatibility Forms
        || in_range(code, 0xFF01, 0xFF60) // Fullwidth Forms
        || in_range(code, 0xFFE0, 0xFFE6) // Fullwidth Forms (symbols)
        || in_range(code, 0x2600, 0x26FF) // Misc Symbols (often wide in terminals)
        || in_range(code, 0x2700, 0x27BF) // Dingbats
        || in_range(code, 0x1F300, 0x1F5FF) // Misc Symbols and Pictographs
        || in_range(code, 0x1F600, 0x1F64F) // Emoticons
        || in_range(code, 0x1F680, 0x1F6FF) // Transport and Map Symbols
        || in_range(code, 0x1F700, 0x1F77F) // Alchemical Symbols
        || in_range(code, 0x1F780, 0x1F7FF) // Geometric Shapes Extended
        || in_range(code, 0x1F800, 0x1F8FF) // Supplemental Arrows-C
        || in_range(code, 0x1F900, 0x1F9FF) // Supplemental Symbols and Pictographs
        || in_range(code, 0x1FA00, 0x1FAFF) // Symbols and Pictographs Extended-A
        || in_range(code, 0x20000, 0x2A6DF) // CJK Unified Ideographs Extension B
        || in_range(code, 0x2A700, 0x2B73F) // CJK Unified Ideographs Extension C
        || in_range(code, 0x2B740, 0x2B81F) // CJK Unified Ideographs Extension D
        || in_range(code, 0x2B820, 0x2CEAF) // CJK Unified Ideographs Extension E
        || in_range(code, 0x2CEB0, 0x2EBEF) // CJK Unified Ideographs Extension F
        || in_range(code, 0x2F800, 0x2FA1F) // CJK Compatibility Ideographs Supplement
        || in_range(code, 0x30000, 0x3134F) // CJK Unified Ideographs Extension G
}

fn in_range(code: u32, start: u32, end: u32) -> bool {
    code >= start && code <= end
}

fn compute_grid(
    drawlist: &BogaeDrawListV1,
    config: ConsoleRenderConfig,
) -> (usize, usize, i32, i32) {
    let width = drawlist.width_px.max(1) as usize;
    let height = drawlist.height_px.max(1) as usize;
    if let (Some(grid_cols), Some(grid_rows)) = (config.grid_cols, config.grid_rows) {
        let grid_cols = grid_cols.max(1);
        let grid_rows = grid_rows.max(1);
        let col_scale = grid_col_scale(config);
        let grid_cols_scaled = grid_cols.saturating_mul(col_scale).max(1);
        let panel_cols_scaled = config.panel_cols.saturating_mul(col_scale);
        let cols = grid_cols_scaled.saturating_add(panel_cols_scaled).max(1);
        let scale_x = ceil_div(width, cols).max(1);
        let scale_y = ceil_div(height, grid_rows).max(1);
        return (cols, grid_rows, scale_x as i32, scale_y as i32);
    }
    let max_cols = config.max_cols.max(1);
    let max_rows = config.max_rows.max(1);
    let (scale_x, scale_y) = match config.cell_aspect {
        ConsoleCellAspect::Auto => {
            let scale_x = ceil_div(width, max_cols).max(1);
            let scale_y = ceil_div(height, max_rows).max(1);
            (scale_x, scale_y)
        }
        ConsoleCellAspect::OneToOne => {
            let scale = ceil_div(width, max_cols)
                .max(ceil_div(height, max_rows))
                .max(1);
            (scale, scale)
        }
        ConsoleCellAspect::TwoToOne => {
            let ratio = 2;
            let max_cols_ratio = max_cols.saturating_mul(ratio).max(1);
            let scale_y = ceil_div(height, max_rows)
                .max(ceil_div(width, max_cols_ratio))
                .max(1);
            let scale_x = scale_y.saturating_mul(ratio);
            (scale_x, scale_y)
        }
    };
    let cols = ceil_div(width, scale_x).max(1);
    let rows = ceil_div(height, scale_y).max(1);
    (cols, rows, scale_x as i32, scale_y as i32)
}

fn ceil_div(value: usize, divisor: usize) -> usize {
    (value + divisor - 1) / divisor
}

fn grid_col_scale(config: ConsoleRenderConfig) -> usize {
    match config.cell_aspect {
        ConsoleCellAspect::TwoToOne => 2,
        _ => 1,
    }
}

fn prefill_panel(grid: &mut [Vec<char>], config: ConsoleRenderConfig) {
    let (Some(grid_cols), Some(_)) = (config.grid_cols, config.grid_rows) else {
        return;
    };
    if config.panel_cols == 0 {
        return;
    }
    if grid.is_empty() || grid[0].is_empty() {
        return;
    }
    let col_scale = grid_col_scale(config);
    let grid_cols_scaled = grid_cols.saturating_mul(col_scale);
    let start_col = grid_cols_scaled.min(grid[0].len());
    for row in grid.iter_mut() {
        for cell in row.iter_mut().skip(start_col) {
            if *cell == ' ' {
                *cell = config.panel_fill;
            }
        }
    }
}

fn draw_panel_border(grid: &mut [Vec<char>], config: ConsoleRenderConfig) {
    let (Some(grid_cols), Some(_)) = (config.grid_cols, config.grid_rows) else {
        return;
    };
    if config.panel_cols == 0 {
        return;
    }
    if grid.is_empty() || grid[0].is_empty() {
        return;
    }
    let col_scale = grid_col_scale(config);
    let grid_cols_scaled = grid_cols.saturating_mul(col_scale);
    let left_col = 0;
    let border_col = grid_cols_scaled
        .saturating_sub(1)
        .min(grid[0].len().saturating_sub(1));
    for row in grid.iter_mut() {
        if row[left_col] == ' ' {
            row[left_col] = config.panel_border;
        }
        if row[border_col] == config.panel_fill || row[border_col] == ' ' {
            row[border_col] = config.panel_border;
        }
    }
}

fn fill_rect(
    grid: &mut [Vec<char>],
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale_x: i32,
    scale_y: i32,
    ch: char,
) {
    let Some((x0, x1, y0, y1)) = rect_bounds(x, y, w, h, scale_x, scale_y, grid) else {
        return;
    };
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            grid[yy][xx] = ch;
        }
    }
}

fn stroke_rect(
    grid: &mut [Vec<char>],
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    thickness: i32,
    scale_x: i32,
    scale_y: i32,
    ch: char,
) {
    let Some((x0, x1, y0, y1)) = rect_bounds(x, y, w, h, scale_x, scale_y, grid) else {
        return;
    };
    let scale = scale_x.max(scale_y) as usize;
    let mut t = (thickness.abs() as usize + scale - 1) / scale;
    if t == 0 {
        t = 1;
    }
    let t = t as usize;
    for yy in y0..=y1 {
        for xx in x0..=x1 {
            let left = xx - x0;
            let right = x1 - xx;
            let top = yy - y0;
            let bottom = y1 - yy;
            if left < t || right < t || top < t || bottom < t {
                grid[yy][xx] = ch;
            }
        }
    }
}

fn draw_line(
    grid: &mut [Vec<char>],
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    scale_x: i32,
    scale_y: i32,
    ch: char,
) {
    let (cols, rows) = grid_dims(grid);
    let mut x0 = x1.div_euclid(scale_x);
    let mut y0 = y1.div_euclid(scale_y);
    let mut x1 = x2.div_euclid(scale_x);
    let mut y1 = y2.div_euclid(scale_y);
    if !clip_point(&mut x0, &mut y0, cols, rows) && !clip_point(&mut x1, &mut y1, cols, rows) {
        return;
    }
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 {
            let ux = x0 as usize;
            let uy = y0 as usize;
            if uy < rows && ux < cols {
                grid[uy][ux] = ch;
            }
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn rect_bounds(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale_x: i32,
    scale_y: i32,
    grid: &[Vec<char>],
) -> Option<(usize, usize, usize, usize)> {
    if w == 0 || h == 0 {
        return None;
    }
    let (cols, rows) = grid_dims(grid);
    let mut x0 = x;
    let mut x1 = x + w;
    if x1 < x0 {
        std::mem::swap(&mut x0, &mut x1);
    }
    let mut y0 = y;
    let mut y1 = y + h;
    if y1 < y0 {
        std::mem::swap(&mut y0, &mut y1);
    }
    let cx0 = x0.div_euclid(scale_x);
    let cx1 = (x1 - 1).div_euclid(scale_x);
    let cy0 = y0.div_euclid(scale_y);
    let cy1 = (y1 - 1).div_euclid(scale_y);
    let (x0, x1) = clamp_range(cx0, cx1, cols)?;
    let (y0, y1) = clamp_range(cy0, cy1, rows)?;
    Some((x0, x1, y0, y1))
}

fn grid_dims(grid: &[Vec<char>]) -> (usize, usize) {
    let rows = grid.len();
    let cols = if rows > 0 { grid[0].len() } else { 0 };
    (cols, rows)
}

fn clamp_range(start: i32, end: i32, max: usize) -> Option<(usize, usize)> {
    if max == 0 {
        return None;
    }
    let max_i = max as i32 - 1;
    if end < 0 || start > max_i {
        return None;
    }
    let s = start.max(0) as usize;
    let e = end.min(max_i) as usize;
    if s > e {
        return None;
    }
    Some((s, e))
}

fn clip_point(x: &mut i32, y: &mut i32, cols: usize, rows: usize) -> bool {
    if cols == 0 || rows == 0 {
        return false;
    }
    if *x < 0 {
        *x = 0;
    }
    if *y < 0 {
        *y = 0;
    }
    if *x >= cols as i32 {
        *x = cols as i32 - 1;
    }
    if *y >= rows as i32 {
        *y = rows as i32 - 1;
    }
    true
}

fn draw_playfield_border(
    grid: &mut [Vec<char>],
    config: ConsoleRenderConfig,
    drawlist: &BogaeDrawListV1,
    scale_x: i32,
    scale_y: i32,
) -> Option<usize> {
    let (Some(grid_cols), Some(grid_rows)) = (config.grid_cols, config.grid_rows) else {
        return None;
    };
    if grid.is_empty() || grid[0].is_empty() {
        return None;
    }
    let max_cols = grid[0].len().saturating_sub(1);
    let max_rows = grid.len().saturating_sub(1);
    let left_col = 0usize;
    let col_scale = grid_col_scale(config);
    let grid_cols_scaled = grid_cols.saturating_mul(col_scale);
    let default_right = grid_cols_scaled.saturating_sub(1).min(max_cols);
    let default_bottom = grid_rows.saturating_sub(1).min(max_rows);
    let (left, right, top, bottom) = if let Some((min_c, max_c, min_r, max_r)) =
        tetris_sprite_bounds(drawlist, scale_x, scale_y, grid_dims(grid))
    {
        let left = min_c.saturating_sub(1);
        let right = max_c.saturating_add(1).min(max_cols);
        let top = min_r.saturating_sub(1);
        let bottom = max_r.saturating_add(1).min(max_rows);
        (left, right, top, bottom)
    } else {
        (left_col, default_right, 0usize, default_bottom)
    };
    if top >= bottom || left >= right {
        return None;
    }

    let horiz = '\u{2501}';
    let vert = '\u{2503}';
    let tl = '\u{250F}';
    let tr = '\u{2513}';
    let bl = '\u{2517}';
    let br = '\u{251B}';

    set_border_cell(grid, top, left, tl, config);
    set_border_cell(grid, top, right, tr, config);
    set_border_cell(grid, bottom, left, bl, config);
    set_border_cell(grid, bottom, right, br, config);

    for col in (left + 1)..right {
        set_border_cell(grid, top, col, horiz, config);
        set_border_cell(grid, bottom, col, horiz, config);
    }
    for row in (top + 1)..bottom {
        set_border_cell(grid, row, left, vert, config);
        set_border_cell(grid, row, right, vert, config);
    }
    Some(bottom)
}

fn set_border_cell(
    grid: &mut [Vec<char>],
    row: usize,
    col: usize,
    ch: char,
    config: ConsoleRenderConfig,
) {
    if row >= grid.len() || col >= grid[0].len() {
        return;
    }
    let cell = grid[row][col];
    if cell == ' ' || cell == config.panel_fill || cell == config.panel_border {
        grid[row][col] = ch;
    }
}

fn tetris_sprite_bounds(
    drawlist: &BogaeDrawListV1,
    scale_x: i32,
    scale_y: i32,
    (cols, rows): (usize, usize),
) -> Option<(usize, usize, usize, usize)> {
    if cols == 0 || rows == 0 {
        return None;
    }
    let mut min_c = usize::MAX;
    let mut max_c = 0usize;
    let mut min_r = usize::MAX;
    let mut max_r = 0usize;
    let mut found = false;
    for cmd in &drawlist.cmds {
        let BogaeCmd::Sprite {
            x, y, w, h, asset, ..
        } = cmd
        else {
            continue;
        };
        if !asset.uri.starts_with("sym:tetris.") {
            continue;
        }
        let x = snap_i32(*x);
        let y = snap_i32(*y);
        let w = snap_i32(*w);
        let h = snap_i32(*h);
        let Some((x0, x1, y0, y1)) = sprite_bounds(x, y, w, h, scale_x, scale_y, cols, rows) else {
            continue;
        };
        found = true;
        min_c = min_c.min(x0);
        max_c = max_c.max(x1);
        min_r = min_r.min(y0);
        max_r = max_r.max(y1);
    }
    if !found {
        return None;
    }
    Some((min_c, max_c, min_r, max_r))
}

fn sprite_bounds(
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    scale_x: i32,
    scale_y: i32,
    cols: usize,
    rows: usize,
) -> Option<(usize, usize, usize, usize)> {
    if w == 0 || h == 0 {
        return None;
    }
    let mut x0 = x;
    let mut x1 = x + w;
    if x1 < x0 {
        std::mem::swap(&mut x0, &mut x1);
    }
    let mut y0 = y;
    let mut y1 = y + h;
    if y1 < y0 {
        std::mem::swap(&mut y0, &mut y1);
    }
    let cx0 = x0.div_euclid(scale_x);
    let cx1 = (x1 - 1).div_euclid(scale_x);
    let cy0 = y0.div_euclid(scale_y);
    let cy1 = (y1 - 1).div_euclid(scale_y);
    let (x0, x1) = clamp_range(cx0, cx1, cols)?;
    let (y0, y1) = clamp_range(cy0, cy1, rows)?;
    Some((x0, x1, y0, y1))
}

fn grid_to_string(grid: &[Vec<char>], max_row: Option<usize>) -> String {
    let mut out = String::new();
    let last_row = max_row.unwrap_or_else(|| grid.len().saturating_sub(1));
    for (row_idx, row) in grid.iter().take(last_row + 1).enumerate() {
        for ch in row {
            out.push(*ch);
        }
        if row_idx + 1 <= last_row {
            out.push('\n');
        }
    }
    out
}

fn snap_i32(value: f32) -> i32 {
    if !value.is_finite() {
        return 0;
    }
    if value >= 0.0 {
        (value + 0.5).floor() as i32
    } else {
        (value - 0.5).ceil() as i32
    }
}
