use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use blake3;
use serde_json::Value as JsonValue;

use crate::core::fixed64::Fixed64;
use crate::core::state::{Key, State};
use crate::core::value::{ListValue, PackValue, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BogaeCodec {
    Bdl1,
    Bdl2,
}

impl BogaeCodec {
    pub fn tag(self) -> &'static str {
        match self {
            BogaeCodec::Bdl1 => "BDL1",
            BogaeCodec::Bdl2 => "BDL2",
        }
    }

    pub fn file_ext(self) -> &'static str {
        match self {
            BogaeCodec::Bdl1 => "bdl1",
            BogaeCodec::Bdl2 => "bdl2",
        }
    }
}

#[derive(Clone, Debug)]
pub struct ColorNamePack {
    entries: BTreeMap<String, Rgba>,
}

impl ColorNamePack {
    pub fn load(path: &Path) -> Result<Self, BogaeError> {
        let raw = fs::read_to_string(path).map_err(|e| BogaeError::ColorPackLoad {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        let json: JsonValue = serde_json::from_str(&raw).map_err(|e| BogaeError::ColorPackLoad {
            path: path.to_path_buf(),
            message: e.to_string(),
        })?;
        let entries = json
            .get("entries")
            .and_then(|value| value.as_array())
            .ok_or_else(|| BogaeError::ColorPackLoad {
                path: path.to_path_buf(),
                message: "entries 배열이 없습니다.".to_string(),
            })?;
        let mut map = BTreeMap::new();
        for entry in entries {
            let hex = entry
                .get("hex")
                .and_then(|value| value.as_str())
                .ok_or_else(|| BogaeError::ColorPackLoad {
                    path: path.to_path_buf(),
                    message: "hex 필드가 없습니다.".to_string(),
                })?;
            let rgba = parse_hex_color(hex).ok_or_else(|| BogaeError::ColorPackLoad {
                path: path.to_path_buf(),
                message: format!("hex 형식 오류: {}", hex),
            })?;
            collect_names(entry.get("en"), &rgba, &mut map);
            collect_names(entry.get("ko"), &rgba, &mut map);
        }
        Ok(Self { entries: map })
    }

    pub fn lookup(&self, name: &str) -> Option<Rgba> {
        let key = normalize_name(name);
        self.entries.get(&key).copied()
    }
}

#[derive(Clone, Debug)]
pub struct BogaeDrawListV1 {
    pub width_px: u32,
    pub height_px: u32,
    pub cmds: Vec<BogaeCmd>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CmdPolicyMode {
    None,
    Cap,
    Summary,
}

#[derive(Clone, Copy, Debug)]
pub struct CmdPolicyConfig {
    pub mode: CmdPolicyMode,
    pub cap: u32,
}

impl CmdPolicyConfig {
    pub fn none() -> Self {
        Self {
            mode: CmdPolicyMode::None,
            cap: 0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CmdPolicyEvent {
    pub mode: CmdPolicyMode,
    pub cap: u32,
    pub cmd_count: u32,
}

#[derive(Clone, Debug)]
pub enum BogaeCmd {
    Clear { color: Rgba, aa: bool },
    RectFill { x: f32, y: f32, w: f32, h: f32, color: Rgba, aa: bool },
    RectStroke {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        thickness: f32,
        color: Rgba,
        aa: bool,
    },
    Line {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        thickness: f32,
        color: Rgba,
        aa: bool,
    },
    Text {
        x: f32,
        y: f32,
        size_px: f32,
        color: Rgba,
        text: String,
        aa: bool,
    },
    Sprite {
        x: f32,
        y: f32,
        w: f32,
        h: f32,
        tint: Rgba,
        asset: AssetRefV1,
        aa: bool,
    },
    CircleFill {
        cx: f32,
        cy: f32,
        r: f32,
        color: Rgba,
        aa: bool,
    },
    CircleStroke {
        cx: f32,
        cy: f32,
        r: f32,
        thickness: f32,
        color: Rgba,
        aa: bool,
    },
    ArcStroke {
        cx: f32,
        cy: f32,
        r: f32,
        start_turn: f32,
        sweep_turn: f32,
        thickness: f32,
        color: Rgba,
        aa: bool,
    },
    CurveCubicStroke {
        p0x: f32,
        p0y: f32,
        p1x: f32,
        p1y: f32,
        p2x: f32,
        p2y: f32,
        p3x: f32,
        p3y: f32,
        thickness: f32,
        color: Rgba,
        aa: bool,
    },
}

#[derive(Clone, Debug)]
pub struct AssetRefV1 {
    pub uri: String,
    pub hash_kind: u8,
    pub hash32: Option<[u8; 32]>,
}

#[derive(Debug)]
pub enum BogaeError {
    NonIntPixel { field: String, value: Fixed64 },
    BadFieldType { field: String, expected: &'static str },
    MissingField { field: String },
    ColorPackLoad { path: PathBuf, message: String },
    ColorPackNotFound { tried: Vec<PathBuf> },
    InvalidColorHex { value: String },
    UnknownColorName { value: String },
    DetbinParse { message: String },
    Bdl2Parse { code: &'static str, message: String },
    CmdCap { cap: u32, cmd_count: u32 },
}

impl BogaeError {
    pub fn code(&self) -> &'static str {
        match self {
            BogaeError::NonIntPixel { .. } => "E_BOGAE_NON_INT_PIXEL",
            BogaeError::BadFieldType { .. } => "E_BOGAE_BAD_FIELD_TYPE",
            BogaeError::MissingField { .. } => "E_BOGAE_MISSING_FIELD",
            BogaeError::ColorPackLoad { .. } => "E_BOGAE_COLOR_PACK_LOAD",
            BogaeError::ColorPackNotFound { .. } => "E_BOGAE_COLOR_PACK_MISSING",
            BogaeError::InvalidColorHex { .. } => "E_BOGAE_COLOR_INVALID",
            BogaeError::UnknownColorName { .. } => "E_BOGAE_COLOR_UNKNOWN",
            BogaeError::DetbinParse { .. } => "E_BOGAE_DETBIN_PARSE",
            BogaeError::Bdl2Parse { code, .. } => code,
            BogaeError::CmdCap { .. } => "E_BOGAE_CMD_CAP",
        }
    }

    pub fn message(&self) -> String {
        match self {
            BogaeError::NonIntPixel { field, value } => {
                format!("픽셀 필드는 정수여야 합니다: {}={}", field, value.format())
            }
            BogaeError::BadFieldType { field, expected } => {
                format!("보개 필드 타입이 맞지 않습니다: {} ({} 필요)", field, expected)
            }
            BogaeError::MissingField { field } => {
                format!("보개 필드가 없습니다: {}", field)
            }
            BogaeError::ColorPackLoad { path, message } => {
                format!("색 이름팩 로드 실패: {} ({})", path.display(), message)
            }
            BogaeError::ColorPackNotFound { tried } => {
                let list = tried
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("색 이름팩을 찾을 수 없습니다: {}", list)
            }
            BogaeError::InvalidColorHex { value } => {
                format!("색 hex 형식이 올바르지 않습니다: {}", value)
            }
            BogaeError::UnknownColorName { value } => {
                format!("알 수 없는 색 이름입니다: {}", value)
            }
            BogaeError::DetbinParse { message } => {
                format!("보개 detbin 파싱 오류: {}", message)
            }
            BogaeError::Bdl2Parse { message, .. } => message.clone(),
            BogaeError::CmdCap { cap, cmd_count } => {
                format!("보개 cmd_count 상한 초과 cmd_count={} cap={}", cmd_count, cap)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct BogaeOutput {
    pub drawlist: BogaeDrawListV1,
    pub detbin: Vec<u8>,
    pub hash: String,
    pub codec: BogaeCodec,
}

enum DrawEntryExtra {
    None,
    Text { size_px: i32, text: String },
    Sprite { uri: String },
}

struct DrawEntry {
    entity_id: String,
    trait_id: String,
    z_order: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    extra: DrawEntryExtra,
    cmd: BogaeCmd,
}

const SHAPE_CANON_SEGMENT: &str = "생김새";
const SHAPE_ALIAS_SEGMENT: &str = "모양";
const SHAPE_TRAIT_CANON: &str = "결";
const SHAPE_TRAIT_ALIAS: &str = "트레잇";
const CANVAS_W_CANON: &str = "보개_그림판_가로";
const CANVAS_H_CANON: &str = "보개_그림판_세로";
const CANVAS_W_ALIAS: &str = "bogae_canvas_w";
const CANVAS_H_ALIAS: &str = "bogae_canvas_h";
const DRAWLIST_LIST_CANON: &str = "보개_그림판_목록";
const TAG_LIST_CANON: &str = "보개_태그_목록";
const MAPPING_LIST_CANON: &str = "보개_매핑_목록";

pub fn default_css4_pack_paths() -> Vec<PathBuf> {
    let mut paths = BTreeSet::new();
    let names = [
        "COLOR_NAME_PACK_CSS4_V1.detjson",
        "ColorNamePack_CSS4_V1.detjson",
    ];
    let mut roots = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        roots.push(cwd.join("docs/ssot/gaji"));
        roots.push(cwd.join("gaji"));
    }
    if let Some(repo_root) = repo_root_dir() {
        roots.push(repo_root.join("docs/ssot/gaji"));
        roots.push(repo_root.join("gaji"));
    }

    for root in &roots {
        for name in &names {
            paths.insert(
                root.join("ddn.std.colors")
                    .join("쓸감")
                    .join("색")
                    .join(name),
            );
        }
        if root.exists() {
            for name in &names {
                collect_named_files(root, name, &mut paths);
            }
        }
    }

    paths.into_iter().collect()
}

pub fn load_css4_pack() -> Result<ColorNamePack, BogaeError> {
    let tried = default_css4_pack_paths();
    for path in &tried {
        if path.exists() {
            return ColorNamePack::load(path);
        }
    }
    Err(BogaeError::ColorPackNotFound { tried })
}

fn collect_named_files(root: &Path, target: &str, out: &mut BTreeSet<PathBuf>) {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_named_files(&path, target, out);
        } else if path.file_name().map_or(false, |name| name == target) {
            out.insert(path);
        }
    }
}

fn repo_root_dir() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.parent()?.parent().map(|path| path.to_path_buf())
}

pub fn build_drawlist_from_state(
    state: &State,
    pack: Option<&ColorNamePack>,
) -> Result<BogaeDrawListV1, BogaeError> {
    let width = read_pixel_i32_with_alias(state, CANVAS_W_CANON, CANVAS_W_ALIAS)?.unwrap_or(0);
    let height = read_pixel_i32_with_alias(state, CANVAS_H_CANON, CANVAS_H_ALIAS)?.unwrap_or(0);
    let bg = match read_str(state, "보개_바탕색")? {
        Some(value) => value,
        None => read_str(state, "bogae_bg")?.unwrap_or_else(|| "#000000ff".to_string()),
    };
    let color = canonicalize_color(&bg, pack)?;
    let mut cmds = vec![BogaeCmd::Clear { color, aa: false }];
    let mut entries = collect_draw_entries(state, pack)?;
    entries.extend(collect_draw_entries_from_list(state, pack)?);
    entries.extend(collect_draw_entries_from_mapping(state, pack)?);
    entries.sort_by(|a, b| {
        let key_a = (
            a.entity_id.as_str(),
            a.trait_id.as_str(),
            a.z_order,
            a.x,
            a.y,
            a.w,
            a.h,
            extra_sort_key(&a.extra),
        );
        let key_b = (
            b.entity_id.as_str(),
            b.trait_id.as_str(),
            b.z_order,
            b.x,
            b.y,
            b.w,
            b.h,
            extra_sort_key(&b.extra),
        );
        key_a.cmp(&key_b)
    });
    for entry in entries {
        cmds.push(entry.cmd);
    }

    Ok(BogaeDrawListV1 {
        width_px: width as u32,
        height_px: height as u32,
        cmds,
    })
}

pub fn build_bogae_output(
    state: &State,
    pack: Option<&ColorNamePack>,
    policy: CmdPolicyConfig,
    codec: BogaeCodec,
) -> Result<(BogaeOutput, Option<CmdPolicyEvent>), BogaeError> {
    let drawlist = build_drawlist_from_state(state, pack)?;
    let (drawlist, event) = apply_cmd_policy(drawlist, policy)?;
    let detbin = match codec {
        BogaeCodec::Bdl1 => encode_drawlist_detbin(&drawlist),
        BogaeCodec::Bdl2 => encode_drawlist_detbin_bdl2(&drawlist),
    };
    let hash = hash_drawlist_detbin(&detbin);
    Ok((
        BogaeOutput {
            drawlist,
            detbin,
            hash,
            codec,
        },
        event,
    ))
}

fn collect_draw_entries(
    state: &State,
    pack: Option<&ColorNamePack>,
) -> Result<Vec<DrawEntry>, BogaeError> {
    let mut entries = Vec::new();
    let canon_suffix = format!(".{}.{}", SHAPE_CANON_SEGMENT, SHAPE_TRAIT_CANON);
    let alias_suffix = format!(".{}.{}", SHAPE_ALIAS_SEGMENT, SHAPE_TRAIT_ALIAS);
    for (key, value) in &state.resources {
        let key_str = key.as_str();
        let entity_id = if let Some(entity_id) = key_str.strip_suffix(&canon_suffix) {
            entity_id
        } else if let Some(entity_id) = key_str.strip_suffix(&alias_suffix) {
            entity_id
        } else {
            continue;
        };
        let trait_id = match value {
            Value::Str(text) => text.trim().to_string(),
            _ => {
                return Err(BogaeError::BadFieldType {
                    field: key_str.to_string(),
                    expected: "문자열",
                })
            }
        };

        let z_key = shape_key(entity_id, "겹");
        let z_key_alias = shape_key_alias(entity_id, "겹");
        let z_order = read_pixel_i32_with_alias(state, &z_key, &z_key_alias)?.unwrap_or(0);

        if is_rect_trait(&trait_id) {
            let rect_key = shape_key(entity_id, "네모");
            let rect_key_alias = shape_key_alias(entity_id, "네모");
            let rect_pack = read_pack_with_alias(state, &rect_key, &rect_key_alias)?
                .ok_or_else(|| BogaeError::MissingField {
                    field: rect_key.clone(),
                })?;
            let x = read_pack_pixel_i32(rect_pack, "x", &format!("{rect_key}.x"))?;
            let y = read_pack_pixel_i32(rect_pack, "y", &format!("{rect_key}.y"))?;
            let w = read_pack_pixel_i32(rect_pack, "w", &format!("{rect_key}.w"))?;
            let h = read_pack_pixel_i32(rect_pack, "h", &format!("{rect_key}.h"))?;
            let color = read_shape_color(state, entity_id, pack)?;
            entries.push(DrawEntry {
                entity_id: entity_id.to_string(),
                trait_id,
                z_order,
                x,
                y,
                w,
                h,
                extra: DrawEntryExtra::None,
                cmd: BogaeCmd::RectFill {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    color,
                    aa: false,
                },
            });
            continue;
        }

        if is_text_trait(&trait_id) {
            let text_key = shape_key(entity_id, "글씨");
            let text_key_alias = shape_key_alias(entity_id, "글씨");
            let text_pack = read_pack_with_alias(state, &text_key, &text_key_alias)?
                .ok_or_else(|| BogaeError::MissingField {
                    field: text_key.clone(),
                })?;
            let x = read_pack_pixel_i32(text_pack, "x", &format!("{text_key}.x"))?;
            let y = read_pack_pixel_i32(text_pack, "y", &format!("{text_key}.y"))?;
            let size = if text_pack.fields.contains_key("size") {
                read_pack_pixel_i32(text_pack, "size", &format!("{text_key}.size"))?
            } else {
                read_pack_pixel_i32(text_pack, "size_px", &format!("{text_key}.size_px"))?
            };
            let text = read_pack_string(text_pack, "text", &format!("{text_key}.text"))?;
            let color = read_shape_color(state, entity_id, pack)?;
            entries.push(DrawEntry {
                entity_id: entity_id.to_string(),
                trait_id,
                z_order,
                x,
                y,
                w: 0,
                h: 0,
                extra: DrawEntryExtra::Text {
                    size_px: size,
                    text: text.clone(),
                },
                cmd: BogaeCmd::Text {
                    x: x as f32,
                    y: y as f32,
                    size_px: size as f32,
                    color,
                    text,
                    aa: false,
                },
            });
            continue;
        }

        if is_sprite_trait(&trait_id) {
            let sprite_key = shape_key(entity_id, "그림");
            let sprite_key_alias = shape_key_alias(entity_id, "그림");
            let sprite_pack = read_pack_with_alias(state, &sprite_key, &sprite_key_alias)?
                .ok_or_else(|| BogaeError::MissingField {
                    field: sprite_key.clone(),
                })?;
            let x = read_pack_pixel_i32(sprite_pack, "x", &format!("{sprite_key}.x"))?;
            let y = read_pack_pixel_i32(sprite_pack, "y", &format!("{sprite_key}.y"))?;
            let w = read_pack_pixel_i32(sprite_pack, "w", &format!("{sprite_key}.w"))?;
            let h = read_pack_pixel_i32(sprite_pack, "h", &format!("{sprite_key}.h"))?;
            let uri = read_pack_string(sprite_pack, "uri", &format!("{sprite_key}.uri"))?;
            let tint = read_shape_color(state, entity_id, pack)?;
            entries.push(DrawEntry {
                entity_id: entity_id.to_string(),
                trait_id,
                z_order,
                x,
                y,
                w,
                h,
                extra: DrawEntryExtra::Sprite { uri: uri.clone() },
                cmd: BogaeCmd::Sprite {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    tint,
                    asset: AssetRefV1 {
                        uri,
                        hash_kind: 0,
                        hash32: None,
                    },
                    aa: false,
                },
            });
        }
    }
    Ok(entries)
}

fn collect_draw_entries_from_list(
    state: &State,
    pack: Option<&ColorNamePack>,
) -> Result<Vec<DrawEntry>, BogaeError> {
    let Some(list) = read_list(state, DRAWLIST_LIST_CANON)? else {
        return Ok(Vec::new());
    };
    let mut entries = Vec::new();
    for (idx, item) in list.items.iter().enumerate() {
        let pack_value = match item {
            Value::Pack(pack) => pack,
            _ => {
                return Err(BogaeError::BadFieldType {
                    field: format!("{}[{}]", DRAWLIST_LIST_CANON, idx),
                    expected: "묶음",
                })
            }
        };
        let entry_ref = format!("{}[{}]", DRAWLIST_LIST_CANON, idx);
        let trait_id = if let Some(value) =
            read_pack_string_optional(pack_value, "결", &format!("{entry_ref}.결"))?
        {
            value
        } else if let Some(value) =
            read_pack_string_optional(pack_value, "트레잇", &format!("{entry_ref}.트레잇"))?
        {
            value
        } else {
            return Err(BogaeError::MissingField {
                field: format!("{entry_ref}.결"),
            });
        };
        let trait_id = trait_id.trim().to_string();
        let entity_id =
            read_pack_string_optional(pack_value, "id", &format!("{entry_ref}.id"))?
                .unwrap_or_else(|| format!("목록.{:06}", idx));
        let z_order =
            read_pack_pixel_i32_optional(pack_value, "겹", &format!("{entry_ref}.겹"))?
                .unwrap_or(0);

        if is_rect_trait(&trait_id) {
            let x = read_pack_pixel_i32(pack_value, "x", &format!("{entry_ref}.x"))?;
            let y = read_pack_pixel_i32(pack_value, "y", &format!("{entry_ref}.y"))?;
            let w = read_pack_pixel_i32(pack_value, "w", &format!("{entry_ref}.w"))?;
            let h = read_pack_pixel_i32(pack_value, "h", &format!("{entry_ref}.h"))?;
            let color = read_pack_color(pack_value, &entry_ref, pack)?;
            entries.push(DrawEntry {
                entity_id,
                trait_id,
                z_order,
                x,
                y,
                w,
                h,
                extra: DrawEntryExtra::None,
                cmd: BogaeCmd::RectFill {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    color,
                    aa: false,
                },
            });
            continue;
        }

        if is_text_trait(&trait_id) {
            let x = read_pack_pixel_i32(pack_value, "x", &format!("{entry_ref}.x"))?;
            let y = read_pack_pixel_i32(pack_value, "y", &format!("{entry_ref}.y"))?;
            let size = if pack_value.fields.contains_key("size") {
                read_pack_pixel_i32(pack_value, "size", &format!("{entry_ref}.size"))?
            } else {
                read_pack_pixel_i32(pack_value, "size_px", &format!("{entry_ref}.size_px"))?
            };
            let text = read_pack_string(pack_value, "text", &format!("{entry_ref}.text"))?;
            let color = read_pack_color(pack_value, &entry_ref, pack)?;
            entries.push(DrawEntry {
                entity_id,
                trait_id,
                z_order,
                x,
                y,
                w: 0,
                h: 0,
                extra: DrawEntryExtra::Text {
                    size_px: size,
                    text: text.clone(),
                },
                cmd: BogaeCmd::Text {
                    x: x as f32,
                    y: y as f32,
                    size_px: size as f32,
                    color,
                    text,
                    aa: false,
                },
            });
            continue;
        }

        if is_sprite_trait(&trait_id) {
            let x = read_pack_pixel_i32(pack_value, "x", &format!("{entry_ref}.x"))?;
            let y = read_pack_pixel_i32(pack_value, "y", &format!("{entry_ref}.y"))?;
            let w = read_pack_pixel_i32(pack_value, "w", &format!("{entry_ref}.w"))?;
            let h = read_pack_pixel_i32(pack_value, "h", &format!("{entry_ref}.h"))?;
            let uri = read_pack_string(pack_value, "uri", &format!("{entry_ref}.uri"))?;
            let tint = read_pack_color(pack_value, &entry_ref, pack)?;
            entries.push(DrawEntry {
                entity_id,
                trait_id,
                z_order,
                x,
                y,
                w,
                h,
                extra: DrawEntryExtra::Sprite { uri: uri.clone() },
                cmd: BogaeCmd::Sprite {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    tint,
                    asset: AssetRefV1 {
                        uri,
                        hash_kind: 0,
                        hash32: None,
                    },
                    aa: false,
                },
            });
        }
    }
    Ok(entries)
}

fn collect_draw_entries_from_mapping(
    state: &State,
    pack: Option<&ColorNamePack>,
) -> Result<Vec<DrawEntry>, BogaeError> {
    let Some(rule_list) = read_list(state, MAPPING_LIST_CANON)? else {
        return Ok(Vec::new());
    };
    let Some(tag_list) = read_list(state, TAG_LIST_CANON)? else {
        return Ok(Vec::new());
    };

    let mut tags = Vec::new();
    for (idx, item) in tag_list.items.iter().enumerate() {
        let pack_value = match item {
            Value::Pack(pack) => pack,
            _ => {
                return Err(BogaeError::BadFieldType {
                    field: format!("{}[{}]", TAG_LIST_CANON, idx),
                    expected: "묶음",
                })
            }
        };
        let entry_ref = format!("{}[{}]", TAG_LIST_CANON, idx);
        let tag = if let Some(value) =
            read_pack_string_optional(pack_value, "tag", &format!("{entry_ref}.tag"))?
        {
            value
        } else if let Some(value) =
            read_pack_string_optional(pack_value, "id", &format!("{entry_ref}.id"))?
        {
            value
        } else {
            return Err(BogaeError::MissingField {
                field: format!("{entry_ref}.tag"),
            });
        };
        tags.push((idx, tag, pack_value));
    }

    struct MappingRule<'a> {
        rule_id: i32,
        tag: String,
        kind: String,
        idx: usize,
        pack: &'a PackValue,
    }

    let mut rules = Vec::new();
    for (idx, item) in rule_list.items.iter().enumerate() {
        let pack_value = match item {
            Value::Pack(pack) => pack,
            _ => {
                return Err(BogaeError::BadFieldType {
                    field: format!("{}[{}]", MAPPING_LIST_CANON, idx),
                    expected: "묶음",
                })
            }
        };
        let entry_ref = format!("{}[{}]", MAPPING_LIST_CANON, idx);
        let rule_id = read_pack_pixel_i32(pack_value, "rule_id", &format!("{entry_ref}.rule_id"))?;
        if rule_id < 0 {
            return Err(BogaeError::BadFieldType {
                field: format!("{entry_ref}.rule_id"),
                expected: "0 이상의 정수",
            });
        }
        let tag = read_pack_string(pack_value, "tag", &format!("{entry_ref}.tag"))?;
        let kind = read_pack_string(pack_value, "kind", &format!("{entry_ref}.kind"))?;
        rules.push(MappingRule {
            rule_id,
            tag,
            kind,
            idx,
            pack: pack_value,
        });
    }

    rules.sort_by(|a, b| (a.rule_id, a.idx).cmp(&(b.rule_id, b.idx)));

    let mut entries = Vec::new();
    for rule in rules {
        for (tag_idx, tag, tag_pack) in &tags {
            if *tag != rule.tag {
                continue;
            }
            let entry_ref = format!("{}[{}]", MAPPING_LIST_CANON, rule.idx);
            let z_order =
                read_mapping_pixel_optional(rule.pack, "겹", &entry_ref, tag_pack, *tag_idx)?
                    .unwrap_or(0);
            let entity_id = format!(
                "매핑.{:06}.{:06}.{:06}",
                rule.rule_id as usize,
                rule.idx,
                tag_idx
            );
            let kind = rule.kind.trim().to_lowercase();
            if kind == "rect" {
                let x = read_mapping_pixel(rule.pack, "x", &entry_ref, tag_pack, *tag_idx)?;
                let y = read_mapping_pixel(rule.pack, "y", &entry_ref, tag_pack, *tag_idx)?;
                let w = read_mapping_pixel(rule.pack, "w", &entry_ref, tag_pack, *tag_idx)?;
                let h = read_mapping_pixel(rule.pack, "h", &entry_ref, tag_pack, *tag_idx)?;
                let color = read_mapping_color(rule.pack, &entry_ref, tag_pack, *tag_idx, pack)?;
                entries.push(DrawEntry {
                    entity_id,
                    trait_id: "#보개/2D.Rect".to_string(),
                    z_order,
                    x,
                    y,
                    w,
                    h,
                    extra: DrawEntryExtra::None,
                    cmd: BogaeCmd::RectFill {
                        x: x as f32,
                        y: y as f32,
                        w: w as f32,
                        h: h as f32,
                        color,
                        aa: false,
                    },
                });
            } else if kind == "text" {
                let x = read_mapping_pixel(rule.pack, "x", &entry_ref, tag_pack, *tag_idx)?;
                let y = read_mapping_pixel(rule.pack, "y", &entry_ref, tag_pack, *tag_idx)?;
                let size = read_mapping_pixel(rule.pack, "size", &entry_ref, tag_pack, *tag_idx)?;
                let text = read_mapping_string(rule.pack, "text", &entry_ref, tag_pack, *tag_idx)?;
                let color = read_mapping_color(rule.pack, &entry_ref, tag_pack, *tag_idx, pack)?;
                entries.push(DrawEntry {
                    entity_id,
                    trait_id: "#보개/2D.Text".to_string(),
                    z_order,
                    x,
                    y,
                    w: 0,
                    h: 0,
                    extra: DrawEntryExtra::Text {
                        size_px: size,
                        text: text.clone(),
                    },
                    cmd: BogaeCmd::Text {
                        x: x as f32,
                        y: y as f32,
                        size_px: size as f32,
                        color,
                        text,
                        aa: false,
                    },
                });
            } else if kind == "sprite" {
                let x = read_mapping_pixel(rule.pack, "x", &entry_ref, tag_pack, *tag_idx)?;
                let y = read_mapping_pixel(rule.pack, "y", &entry_ref, tag_pack, *tag_idx)?;
                let w = read_mapping_pixel(rule.pack, "w", &entry_ref, tag_pack, *tag_idx)?;
                let h = read_mapping_pixel(rule.pack, "h", &entry_ref, tag_pack, *tag_idx)?;
                let uri = read_mapping_string(rule.pack, "uri", &entry_ref, tag_pack, *tag_idx)?;
                let tint = read_mapping_color(rule.pack, &entry_ref, tag_pack, *tag_idx, pack)?;
                entries.push(DrawEntry {
                    entity_id,
                    trait_id: "#보개/2D.Sprite".to_string(),
                    z_order,
                    x,
                    y,
                    w,
                    h,
                    extra: DrawEntryExtra::Sprite { uri: uri.clone() },
                    cmd: BogaeCmd::Sprite {
                        x: x as f32,
                        y: y as f32,
                        w: w as f32,
                        h: h as f32,
                        tint,
                        asset: AssetRefV1 {
                            uri,
                            hash_kind: 0,
                            hash32: None,
                        },
                        aa: false,
                    },
                });
            } else {
                return Err(BogaeError::BadFieldType {
                    field: format!("{entry_ref}.kind"),
                    expected: "rect/text/sprite",
                });
            }
        }
    }

    Ok(entries)
}

fn is_rect_trait(trait_id: &str) -> bool {
    matches!(trait_id.trim(), "#보개/2D.Rect" | "보개/2D.Rect")
}

fn is_text_trait(trait_id: &str) -> bool {
    matches!(trait_id.trim(), "#보개/2D.Text" | "보개/2D.Text")
}

fn is_sprite_trait(trait_id: &str) -> bool {
    matches!(trait_id.trim(), "#보개/2D.Sprite" | "보개/2D.Sprite")
}

fn shape_key(entity_id: &str, field: &str) -> String {
    format!("{}.{}.{}", entity_id, SHAPE_CANON_SEGMENT, field)
}

fn shape_key_alias(entity_id: &str, field: &str) -> String {
    format!("{}.{}.{}", entity_id, SHAPE_ALIAS_SEGMENT, field)
}

fn read_str_with_alias(
    state: &State,
    primary: &str,
    alias: &str,
) -> Result<Option<String>, BogaeError> {
    if let Some(value) = read_str(state, primary)? {
        return Ok(Some(value));
    }
    read_str(state, alias)
}

fn read_pixel_i32_with_alias(
    state: &State,
    primary: &str,
    alias: &str,
) -> Result<Option<i32>, BogaeError> {
    if let Some(value) = read_pixel_i32(state, primary)? {
        return Ok(Some(value));
    }
    read_pixel_i32(state, alias)
}

fn read_pack_with_alias<'a>(
    state: &'a State,
    primary: &str,
    alias: &str,
) -> Result<Option<&'a PackValue>, BogaeError> {
    if let Some(pack) = read_pack(state, primary)? {
        return Ok(Some(pack));
    }
    read_pack(state, alias)
}

fn read_shape_color(
    state: &State,
    entity_id: &str,
    pack: Option<&ColorNamePack>,
) -> Result<Rgba, BogaeError> {
    let candidates = ["채움색", "색"];
    for name in candidates {
        let key = shape_key(entity_id, name);
        let key_alias = shape_key_alias(entity_id, name);
        if let Some(value) = read_str_with_alias(state, &key, &key_alias)? {
            return canonicalize_color(&value, pack);
        }
    }
    Err(BogaeError::MissingField {
        field: shape_key(entity_id, "색"),
    })
}

fn extra_sort_key(extra: &DrawEntryExtra) -> (i32, &str, i32) {
    match extra {
        DrawEntryExtra::None => (0, "", 0),
        DrawEntryExtra::Text { size_px, text } => (1, text.as_str(), *size_px),
        DrawEntryExtra::Sprite { uri } => (2, uri.as_str(), 0),
    }
}

fn apply_cmd_policy(
    mut drawlist: BogaeDrawListV1,
    policy: CmdPolicyConfig,
) -> Result<(BogaeDrawListV1, Option<CmdPolicyEvent>), BogaeError> {
    if matches!(policy.mode, CmdPolicyMode::None) {
        return Ok((drawlist, None));
    }
    let cmd_count = drawlist.cmds.len() as u32;
    if cmd_count <= policy.cap {
        return Ok((drawlist, None));
    }
    match policy.mode {
        CmdPolicyMode::None => Ok((drawlist, None)),
        CmdPolicyMode::Cap => Err(BogaeError::CmdCap {
            cap: policy.cap,
            cmd_count,
        }),
        CmdPolicyMode::Summary => {
            let keep = if policy.cap <= 1 {
                policy.cap as usize
            } else {
                (policy.cap - 1) as usize
            };
            let mut cmds = Vec::with_capacity(policy.cap as usize);
            cmds.extend(drawlist.cmds.iter().take(keep).cloned());
            if policy.cap > 1 {
                cmds.push(BogaeCmd::Text {
                    x: 0.0,
                    y: 0.0,
                    size_px: 12.0,
                    color: Rgba {
                        r: 255,
                        g: 64,
                        b: 64,
                        a: 255,
                    },
                    text: format!("cmds {} > cap {}", cmd_count, policy.cap),
                    aa: false,
                });
            }
            drawlist.cmds = cmds;
            Ok((
                drawlist,
                Some(CmdPolicyEvent {
                    mode: policy.mode,
                    cap: policy.cap,
                    cmd_count,
                }),
            ))
        }
    }
}

pub fn canonicalize_color(value: &str, pack: Option<&ColorNamePack>) -> Result<Rgba, BogaeError> {
    let trimmed = value.trim();
    if trimmed.starts_with('#') {
        return parse_hex_color(trimmed)
            .ok_or_else(|| BogaeError::InvalidColorHex { value: trimmed.to_string() });
    }
    let Some(pack) = pack else {
        return Err(BogaeError::ColorPackNotFound {
            tried: default_css4_pack_paths(),
        });
    };
    pack.lookup(trimmed)
        .ok_or_else(|| BogaeError::UnknownColorName {
            value: trimmed.to_string(),
        })
}

pub fn encode_drawlist_detbin(drawlist: &BogaeDrawListV1) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"BDL1");
    out.extend_from_slice(&1u32.to_le_bytes());
    out.extend_from_slice(&drawlist.width_px.to_le_bytes());
    out.extend_from_slice(&drawlist.height_px.to_le_bytes());
    out.extend_from_slice(&(drawlist.cmds.len() as u32).to_le_bytes());

    for cmd in &drawlist.cmds {
        match cmd {
            BogaeCmd::Clear { color, .. } => {
                out.push(0x01);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::RectFill { x, y, w, h, color, .. } => {
                out.push(0x02);
                push_i32(&mut out, round_f32_to_i32(*x));
                push_i32(&mut out, round_f32_to_i32(*y));
                push_i32(&mut out, round_f32_to_i32(*w));
                push_i32(&mut out, round_f32_to_i32(*h));
                push_rgba(&mut out, *color);
            }
            BogaeCmd::RectStroke {
                x,
                y,
                w,
                h,
                thickness,
                color,
                ..
            } => {
                out.push(0x03);
                push_i32(&mut out, round_f32_to_i32(*x));
                push_i32(&mut out, round_f32_to_i32(*y));
                push_i32(&mut out, round_f32_to_i32(*w));
                push_i32(&mut out, round_f32_to_i32(*h));
                push_i32(&mut out, round_f32_to_i32(*thickness));
                push_rgba(&mut out, *color);
            }
            BogaeCmd::Line {
                x1,
                y1,
                x2,
                y2,
                thickness,
                color,
                ..
            } => {
                out.push(0x04);
                push_i32(&mut out, round_f32_to_i32(*x1));
                push_i32(&mut out, round_f32_to_i32(*y1));
                push_i32(&mut out, round_f32_to_i32(*x2));
                push_i32(&mut out, round_f32_to_i32(*y2));
                push_i32(&mut out, round_f32_to_i32(*thickness));
                push_rgba(&mut out, *color);
            }
            BogaeCmd::Text {
                x,
                y,
                size_px,
                color,
                text,
                ..
            } => {
                out.push(0x05);
                push_i32(&mut out, round_f32_to_i32(*x));
                push_i32(&mut out, round_f32_to_i32(*y));
                push_i32(&mut out, round_f32_to_i32(*size_px));
                push_rgba(&mut out, *color);
                push_str(&mut out, text);
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
                out.push(0x06);
                push_i32(&mut out, round_f32_to_i32(*x));
                push_i32(&mut out, round_f32_to_i32(*y));
                push_i32(&mut out, round_f32_to_i32(*w));
                push_i32(&mut out, round_f32_to_i32(*h));
                push_rgba(&mut out, *tint);
                push_str(&mut out, &asset.uri);
                out.push(asset.hash_kind);
                if asset.hash_kind == 1 {
                    if let Some(hash) = &asset.hash32 {
                        out.extend_from_slice(hash);
                    } else {
                        out.extend_from_slice(&[0u8; 32]);
                    }
                }
            }
            BogaeCmd::CircleFill { cx, cy, r, color, .. } => {
                out.push(0x02);
                let cx = round_f32_to_i32(*cx);
                let cy = round_f32_to_i32(*cy);
                let r = round_f32_to_i32(*r);
                push_i32(&mut out, cx - r);
                push_i32(&mut out, cy - r);
                push_i32(&mut out, r * 2);
                push_i32(&mut out, r * 2);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::CircleStroke {
                cx,
                cy,
                r,
                thickness,
                color,
                ..
            } => {
                out.push(0x03);
                let cx = round_f32_to_i32(*cx);
                let cy = round_f32_to_i32(*cy);
                let r = round_f32_to_i32(*r);
                push_i32(&mut out, cx - r);
                push_i32(&mut out, cy - r);
                push_i32(&mut out, r * 2);
                push_i32(&mut out, r * 2);
                push_i32(&mut out, round_f32_to_i32(*thickness));
                push_rgba(&mut out, *color);
            }
            BogaeCmd::ArcStroke {
                cx,
                cy,
                r,
                start_turn,
                sweep_turn,
                thickness,
                color,
                ..
            } => {
                let start = (*start_turn as f64) * std::f64::consts::PI * 2.0;
                let end = start + (*sweep_turn as f64) * std::f64::consts::PI * 2.0;
                let cx = *cx as f64;
                let cy = *cy as f64;
                let r = *r as f64;
                let x1 = (cx + r * start.cos()).round() as i32;
                let y1 = (cy + r * start.sin()).round() as i32;
                let x2 = (cx + r * end.cos()).round() as i32;
                let y2 = (cy + r * end.sin()).round() as i32;
                out.push(0x04);
                push_i32(&mut out, x1);
                push_i32(&mut out, y1);
                push_i32(&mut out, x2);
                push_i32(&mut out, y2);
                push_i32(&mut out, round_f32_to_i32(*thickness));
                push_rgba(&mut out, *color);
            }
            BogaeCmd::CurveCubicStroke {
                p0x,
                p0y,
                p3x,
                p3y,
                thickness,
                color,
                ..
            } => {
                out.push(0x04);
                push_i32(&mut out, round_f32_to_i32(*p0x));
                push_i32(&mut out, round_f32_to_i32(*p0y));
                push_i32(&mut out, round_f32_to_i32(*p3x));
                push_i32(&mut out, round_f32_to_i32(*p3y));
                push_i32(&mut out, round_f32_to_i32(*thickness));
                push_rgba(&mut out, *color);
            }
        }
    }

    out
}

pub fn encode_drawlist_detbin_bdl2(drawlist: &BogaeDrawListV1) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(b"BDL2");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&drawlist.width_px.to_le_bytes());
    out.extend_from_slice(&drawlist.height_px.to_le_bytes());
    out.push(8u8);
    out.push(0u8);
    out.extend_from_slice(&0u16.to_le_bytes());
    out.extend_from_slice(&(drawlist.cmds.len() as u32).to_le_bytes());

    for cmd in &drawlist.cmds {
        match cmd {
            BogaeCmd::Clear { color, aa } => {
                out.push(0x01);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_rgba(&mut out, *color);
            }
            BogaeCmd::RectFill { x, y, w, h, color, aa } => {
                out.push(0x02);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *x);
                push_q24_8(&mut out, *y);
                push_q24_8(&mut out, *w);
                push_q24_8(&mut out, *h);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::RectStroke {
                x,
                y,
                w,
                h,
                thickness,
                color,
                aa,
            } => {
                out.push(0x03);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *x);
                push_q24_8(&mut out, *y);
                push_q24_8(&mut out, *w);
                push_q24_8(&mut out, *h);
                push_q24_8(&mut out, *thickness);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::Line {
                x1,
                y1,
                x2,
                y2,
                thickness,
                color,
                aa,
            } => {
                out.push(0x04);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *x1);
                push_q24_8(&mut out, *y1);
                push_q24_8(&mut out, *x2);
                push_q24_8(&mut out, *y2);
                push_q24_8(&mut out, *thickness);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::Text {
                x,
                y,
                size_px,
                color,
                text,
                aa,
            } => {
                out.push(0x05);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *x);
                push_q24_8(&mut out, *y);
                push_q24_8(&mut out, *size_px);
                push_rgba(&mut out, *color);
                push_str(&mut out, text);
            }
            BogaeCmd::Sprite {
                x,
                y,
                w,
                h,
                tint,
                asset,
                aa,
            } => {
                out.push(0x06);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *x);
                push_q24_8(&mut out, *y);
                push_q24_8(&mut out, *w);
                push_q24_8(&mut out, *h);
                push_rgba(&mut out, *tint);
                push_str(&mut out, &asset.uri);
                out.push(asset.hash_kind);
                if asset.hash_kind == 1 {
                    if let Some(hash) = &asset.hash32 {
                        out.extend_from_slice(hash);
                    } else {
                        out.extend_from_slice(&[0u8; 32]);
                    }
                }
            }
            BogaeCmd::CircleFill { cx, cy, r, color, aa } => {
                out.push(0x07);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *cx);
                push_q24_8(&mut out, *cy);
                push_q24_8(&mut out, *r);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::CircleStroke {
                cx,
                cy,
                r,
                thickness,
                color,
                aa,
            } => {
                out.push(0x08);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *cx);
                push_q24_8(&mut out, *cy);
                push_q24_8(&mut out, *r);
                push_q24_8(&mut out, *thickness);
                push_rgba(&mut out, *color);
            }
            BogaeCmd::ArcStroke {
                cx,
                cy,
                r,
                start_turn,
                sweep_turn,
                thickness,
                color,
                aa,
            } => {
                out.push(0x09);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *cx);
                push_q24_8(&mut out, *cy);
                push_q24_8(&mut out, *r);
                push_q24_8(&mut out, *start_turn);
                push_q24_8(&mut out, *sweep_turn);
                push_q24_8(&mut out, *thickness);
                push_rgba(&mut out, *color);
            }
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
                color,
                aa,
            } => {
                out.push(0x0A);
                out.push(if *aa { 0x01 } else { 0x00 });
                push_q24_8(&mut out, *p0x);
                push_q24_8(&mut out, *p0y);
                push_q24_8(&mut out, *p1x);
                push_q24_8(&mut out, *p1y);
                push_q24_8(&mut out, *p2x);
                push_q24_8(&mut out, *p2y);
                push_q24_8(&mut out, *p3x);
                push_q24_8(&mut out, *p3y);
                push_q24_8(&mut out, *thickness);
                push_rgba(&mut out, *color);
            }
        }
    }

    out
}

pub fn hash_drawlist_detbin(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    format!("blake3:{}", digest.to_hex())
}

pub fn decode_drawlist_detbin(bytes: &[u8]) -> Result<BogaeDrawListV1, BogaeError> {
    let mut idx = 0usize;
    let magic = take_bytes(bytes, &mut idx, 4)?;
    if magic != b"BDL1" {
        return Err(BogaeError::DetbinParse {
            message: "magic이 BDL1이 아님".to_string(),
        });
    }
    let version = read_u32(bytes, &mut idx)?;
    if version != 1 {
        return Err(BogaeError::DetbinParse {
            message: format!("지원하지 않는 버전: {}", version),
        });
    }
    let width = read_u32(bytes, &mut idx)?;
    let height = read_u32(bytes, &mut idx)?;
    let cmd_count = read_u32(bytes, &mut idx)? as usize;
    let mut cmds = Vec::with_capacity(cmd_count);
    for _ in 0..cmd_count {
        let kind = read_u8(bytes, &mut idx)?;
        let cmd = match kind {
            0x01 => {
                let color = read_rgba(bytes, &mut idx)?;
                BogaeCmd::Clear { color, aa: false }
            }
            0x02 => {
                let x = read_i32(bytes, &mut idx)?;
                let y = read_i32(bytes, &mut idx)?;
                let w = read_i32(bytes, &mut idx)?;
                let h = read_i32(bytes, &mut idx)?;
                let color = read_rgba(bytes, &mut idx)?;
                BogaeCmd::RectFill {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    color,
                    aa: false,
                }
            }
            0x03 => {
                let x = read_i32(bytes, &mut idx)?;
                let y = read_i32(bytes, &mut idx)?;
                let w = read_i32(bytes, &mut idx)?;
                let h = read_i32(bytes, &mut idx)?;
                let thickness = read_i32(bytes, &mut idx)?;
                let color = read_rgba(bytes, &mut idx)?;
                BogaeCmd::RectStroke {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    thickness: thickness as f32,
                    color,
                    aa: false,
                }
            }
            0x04 => {
                let x1 = read_i32(bytes, &mut idx)?;
                let y1 = read_i32(bytes, &mut idx)?;
                let x2 = read_i32(bytes, &mut idx)?;
                let y2 = read_i32(bytes, &mut idx)?;
                let thickness = read_i32(bytes, &mut idx)?;
                let color = read_rgba(bytes, &mut idx)?;
                BogaeCmd::Line {
                    x1: x1 as f32,
                    y1: y1 as f32,
                    x2: x2 as f32,
                    y2: y2 as f32,
                    thickness: thickness as f32,
                    color,
                    aa: false,
                }
            }
            0x05 => {
                let x = read_i32(bytes, &mut idx)?;
                let y = read_i32(bytes, &mut idx)?;
                let size_px = read_i32(bytes, &mut idx)?;
                let color = read_rgba(bytes, &mut idx)?;
                let text = read_string(bytes, &mut idx)?;
                BogaeCmd::Text {
                    x: x as f32,
                    y: y as f32,
                    size_px: size_px as f32,
                    color,
                    text,
                    aa: false,
                }
            }
            0x06 => {
                let x = read_i32(bytes, &mut idx)?;
                let y = read_i32(bytes, &mut idx)?;
                let w = read_i32(bytes, &mut idx)?;
                let h = read_i32(bytes, &mut idx)?;
                let tint = read_rgba(bytes, &mut idx)?;
                let uri = read_string(bytes, &mut idx)?;
                let hash_kind = read_u8(bytes, &mut idx)?;
                let hash32 = if hash_kind == 1 {
                    Some(read_hash32(bytes, &mut idx)?)
                } else {
                    None
                };
                BogaeCmd::Sprite {
                    x: x as f32,
                    y: y as f32,
                    w: w as f32,
                    h: h as f32,
                    tint,
                    asset: AssetRefV1 {
                        uri,
                        hash_kind,
                        hash32,
                    },
                    aa: false,
                }
            }
            other => {
                return Err(BogaeError::DetbinParse {
                    message: format!("알 수 없는 cmd kind: 0x{:02x}", other),
                })
            }
        };
        cmds.push(cmd);
    }
    if idx != bytes.len() {
        return Err(BogaeError::DetbinParse {
            message: "detbin 뒤에 여분 바이트가 있음".to_string(),
        });
    }
    Ok(BogaeDrawListV1 {
        width_px: width,
        height_px: height,
        cmds,
    })
}

pub fn decode_drawlist_detbin_bdl2(bytes: &[u8]) -> Result<BogaeDrawListV1, BogaeError> {
    let mut idx = 0usize;
    let magic = take_bytes_bdl2(bytes, &mut idx, 4)?;
    if magic != b"BDL2" {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_MAGIC",
            message: "magic이 BDL2가 아님".to_string(),
        });
    }
    let version = read_u32_bdl2(bytes, &mut idx)?;
    if version != 2 {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_VERSION",
            message: format!("지원하지 않는 버전: {}", version),
        });
    }
    let width = read_u32_bdl2(bytes, &mut idx)?;
    let height = read_u32_bdl2(bytes, &mut idx)?;
    let fixed_q = read_u8_bdl2(bytes, &mut idx)?;
    if fixed_q != 8 {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_FIXED_Q",
            message: format!("fixed_q가 8이 아님: {}", fixed_q),
        });
    }
    let flags = read_u8_bdl2(bytes, &mut idx)?;
    if flags != 0 {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_FLAGS",
            message: format!("reserved flags가 0이 아님: {}", flags),
        });
    }
    let _reserved = read_u16_bdl2(bytes, &mut idx)?;
    let cmd_count = read_u32_bdl2(bytes, &mut idx)? as usize;
    let mut cmds = Vec::with_capacity(cmd_count);
    for _ in 0..cmd_count {
        let kind = read_u8_bdl2(bytes, &mut idx)?;
        let cmd_flags = read_u8_bdl2(bytes, &mut idx)?;
        if cmd_flags & 0xFE != 0 {
            return Err(BogaeError::Bdl2Parse {
                code: "E_BDL2_FLAGS",
                message: format!("cmd flags가 0이 아님: {}", cmd_flags),
            });
        }
        let aa = (cmd_flags & 0x01) != 0;
        let cmd = match kind {
            0x01 => {
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::Clear { color, aa }
            }
            0x02 => {
                let x = read_q24_8_f32(bytes, &mut idx, "x")?;
                let y = read_q24_8_f32(bytes, &mut idx, "y")?;
                let w = read_q24_8_f32(bytes, &mut idx, "w")?;
                let h = read_q24_8_f32(bytes, &mut idx, "h")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::RectFill { x, y, w, h, color, aa }
            }
            0x03 => {
                let x = read_q24_8_f32(bytes, &mut idx, "x")?;
                let y = read_q24_8_f32(bytes, &mut idx, "y")?;
                let w = read_q24_8_f32(bytes, &mut idx, "w")?;
                let h = read_q24_8_f32(bytes, &mut idx, "h")?;
                let thickness = read_q24_8_f32(bytes, &mut idx, "thickness")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::RectStroke {
                    x,
                    y,
                    w,
                    h,
                    thickness,
                    color,
                    aa,
                }
            }
            0x04 => {
                let x1 = read_q24_8_f32(bytes, &mut idx, "x1")?;
                let y1 = read_q24_8_f32(bytes, &mut idx, "y1")?;
                let x2 = read_q24_8_f32(bytes, &mut idx, "x2")?;
                let y2 = read_q24_8_f32(bytes, &mut idx, "y2")?;
                let thickness = read_q24_8_f32(bytes, &mut idx, "thickness")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::Line {
                    x1,
                    y1,
                    x2,
                    y2,
                    thickness,
                    color,
                    aa,
                }
            }
            0x05 => {
                let x = read_q24_8_f32(bytes, &mut idx, "x")?;
                let y = read_q24_8_f32(bytes, &mut idx, "y")?;
                let size_px = read_q24_8_f32(bytes, &mut idx, "size")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                let text = read_string_bdl2(bytes, &mut idx)?;
                BogaeCmd::Text {
                    x,
                    y,
                    size_px,
                    color,
                    text,
                    aa,
                }
            }
            0x06 => {
                let x = read_q24_8_f32(bytes, &mut idx, "x")?;
                let y = read_q24_8_f32(bytes, &mut idx, "y")?;
                let w = read_q24_8_f32(bytes, &mut idx, "w")?;
                let h = read_q24_8_f32(bytes, &mut idx, "h")?;
                let tint = read_rgba_bdl2(bytes, &mut idx)?;
                let uri = read_string_bdl2(bytes, &mut idx)?;
                let hash_kind = read_u8_bdl2(bytes, &mut idx)?;
                let hash32 = if hash_kind == 1 {
                    Some(read_hash32_bdl2(bytes, &mut idx)?)
                } else {
                    None
                };
                BogaeCmd::Sprite {
                    x,
                    y,
                    w,
                    h,
                    tint,
                    asset: AssetRefV1 {
                        uri,
                        hash_kind,
                        hash32,
                    },
                    aa,
                }
            }
            0x07 => {
                let cx = read_q24_8_f32(bytes, &mut idx, "cx")?;
                let cy = read_q24_8_f32(bytes, &mut idx, "cy")?;
                let r = read_q24_8_f32(bytes, &mut idx, "r")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::CircleFill { cx, cy, r, color, aa }
            }
            0x08 => {
                let cx = read_q24_8_f32(bytes, &mut idx, "cx")?;
                let cy = read_q24_8_f32(bytes, &mut idx, "cy")?;
                let r = read_q24_8_f32(bytes, &mut idx, "r")?;
                let thickness = read_q24_8_f32(bytes, &mut idx, "thickness")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::CircleStroke {
                    cx,
                    cy,
                    r,
                    thickness,
                    color,
                    aa,
                }
            }
            0x09 => {
                let cx = read_q24_8_f32(bytes, &mut idx, "cx")?;
                let cy = read_q24_8_f32(bytes, &mut idx, "cy")?;
                let r = read_q24_8_f32(bytes, &mut idx, "r")?;
                let start_turn = read_q24_8_f32(bytes, &mut idx, "start_turn")?;
                let sweep_turn = read_q24_8_f32(bytes, &mut idx, "sweep_turn")?;
                let thickness = read_q24_8_f32(bytes, &mut idx, "thickness")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
                BogaeCmd::ArcStroke {
                    cx,
                    cy,
                    r,
                    start_turn,
                    sweep_turn,
                    thickness,
                    color,
                    aa,
                }
            }
            0x0A => {
                let p0x = read_q24_8_f32(bytes, &mut idx, "p0x")?;
                let p0y = read_q24_8_f32(bytes, &mut idx, "p0y")?;
                let p1x = read_q24_8_f32(bytes, &mut idx, "p1x")?;
                let p1y = read_q24_8_f32(bytes, &mut idx, "p1y")?;
                let p2x = read_q24_8_f32(bytes, &mut idx, "p2x")?;
                let p2y = read_q24_8_f32(bytes, &mut idx, "p2y")?;
                let p3x = read_q24_8_f32(bytes, &mut idx, "p3x")?;
                let p3y = read_q24_8_f32(bytes, &mut idx, "p3y")?;
                let thickness = read_q24_8_f32(bytes, &mut idx, "thickness")?;
                let color = read_rgba_bdl2(bytes, &mut idx)?;
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
                    color,
                    aa,
                }
            }
            other => {
                return Err(BogaeError::Bdl2Parse {
                    code: "E_BDL2_CMD_KIND",
                    message: format!("알 수 없는 cmd kind: 0x{:02x}", other),
                })
            }
        };
        cmds.push(cmd);
    }
    if idx != bytes.len() {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_TRAILING",
            message: "detbin 뒤에 여분 바이트가 있음".to_string(),
        });
    }
    Ok(BogaeDrawListV1 {
        width_px: width,
        height_px: height,
        cmds,
    })
}

pub fn decode_drawlist_detbin_any(
    bytes: &[u8],
) -> Result<(BogaeDrawListV1, BogaeCodec), BogaeError> {
    if bytes.len() < 4 {
        return Err(BogaeError::DetbinParse {
            message: "detbin 길이가 부족함".to_string(),
        });
    }
    match &bytes[0..4] {
        b"BDL1" => decode_drawlist_detbin(bytes).map(|drawlist| (drawlist, BogaeCodec::Bdl1)),
        b"BDL2" => decode_drawlist_detbin_bdl2(bytes).map(|drawlist| (drawlist, BogaeCodec::Bdl2)),
        _ => Err(BogaeError::DetbinParse {
            message: "알 수 없는 detbin magic".to_string(),
        }),
    }
}

fn read_str(state: &State, key: &str) -> Result<Option<String>, BogaeError> {
    let value = state.resources.get(&Key::new(key));
    match value {
        None => Ok(None),
        Some(Value::Str(text)) => Ok(Some(text.clone())),
        Some(_) => Err(BogaeError::BadFieldType {
            field: key.to_string(),
            expected: "문자열",
        }),
    }
}

fn read_pixel_i32(state: &State, key: &str) -> Result<Option<i32>, BogaeError> {
    let value = state.resources.get(&Key::new(key));
    let Some(value) = value else {
        return Ok(None);
    };
    value_to_pixel_i32(value, key).map(Some)
}

fn read_pack<'a>(state: &'a State, key: &str) -> Result<Option<&'a PackValue>, BogaeError> {
    let value = state.resources.get(&Key::new(key));
    match value {
        None => Ok(None),
        Some(Value::Pack(pack)) => Ok(Some(pack)),
        Some(_) => Err(BogaeError::BadFieldType {
            field: key.to_string(),
            expected: "묶음",
        }),
    }
}

fn read_list<'a>(state: &'a State, key: &str) -> Result<Option<&'a ListValue>, BogaeError> {
    let value = state.resources.get(&Key::new(key));
    match value {
        None => Ok(None),
        Some(Value::List(list)) => Ok(Some(list)),
        Some(_) => Err(BogaeError::BadFieldType {
            field: key.to_string(),
            expected: "목록",
        }),
    }
}

fn read_pack_pixel_i32(
    pack: &PackValue,
    field: &str,
    full_field: &str,
) -> Result<i32, BogaeError> {
    let value = pack
        .fields
        .get(field)
        .ok_or_else(|| BogaeError::MissingField {
            field: full_field.to_string(),
        })?;
    value_to_pixel_i32(value, full_field)
}

fn read_pack_pixel_i32_optional(
    pack: &PackValue,
    field: &str,
    full_field: &str,
) -> Result<Option<i32>, BogaeError> {
    let Some(value) = pack.fields.get(field) else {
        return Ok(None);
    };
    value_to_pixel_i32(value, full_field).map(Some)
}

fn read_pack_string(
    pack: &PackValue,
    field: &str,
    full_field: &str,
) -> Result<String, BogaeError> {
    let value = pack
        .fields
        .get(field)
        .ok_or_else(|| BogaeError::MissingField {
            field: full_field.to_string(),
        })?;
    match value {
        Value::Str(text) => Ok(text.clone()),
        _ => Err(BogaeError::BadFieldType {
            field: full_field.to_string(),
            expected: "문자열",
        }),
    }
}

fn read_pack_string_optional(
    pack: &PackValue,
    field: &str,
    full_field: &str,
) -> Result<Option<String>, BogaeError> {
    match pack.fields.get(field) {
        None => Ok(None),
        Some(Value::Str(text)) => Ok(Some(text.clone())),
        Some(_) => Err(BogaeError::BadFieldType {
            field: full_field.to_string(),
            expected: "문자열",
        }),
    }
}

fn read_mapping_pixel(
    rule_pack: &PackValue,
    field: &str,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
) -> Result<i32, BogaeError> {
    let Some(value) = rule_pack.fields.get(field) else {
        return Err(BogaeError::MissingField {
            field: format!("{entry_ref}.{field}"),
        });
    };
    resolve_mapping_pixel(value, field, entry_ref, tag_pack, tag_idx)
}

fn read_mapping_pixel_optional(
    rule_pack: &PackValue,
    field: &str,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
) -> Result<Option<i32>, BogaeError> {
    let Some(value) = rule_pack.fields.get(field) else {
        return Ok(None);
    };
    resolve_mapping_pixel(value, field, entry_ref, tag_pack, tag_idx).map(Some)
}

fn resolve_mapping_pixel(
    value: &Value,
    field: &str,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
) -> Result<i32, BogaeError> {
    match value {
        Value::Num(_) => value_to_pixel_i32(value, &format!("{entry_ref}.{field}")),
        Value::Str(text) => {
            let key = text.trim();
            let Some(tag_key) = key.strip_prefix('$') else {
                return Err(BogaeError::BadFieldType {
                    field: format!("{entry_ref}.{field}"),
                    expected: "숫자 또는 $필드",
                });
            };
            let Some(tag_value) = tag_pack.fields.get(tag_key) else {
                return Err(BogaeError::MissingField {
                    field: format!("{}[{}].{}", TAG_LIST_CANON, tag_idx, tag_key),
                });
            };
            value_to_pixel_i32(tag_value, &format!("{}[{}].{}", TAG_LIST_CANON, tag_idx, tag_key))
        }
        _ => Err(BogaeError::BadFieldType {
            field: format!("{entry_ref}.{field}"),
            expected: "숫자 또는 $필드",
        }),
    }
}

fn read_mapping_string(
    rule_pack: &PackValue,
    field: &str,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
) -> Result<String, BogaeError> {
    let Some(value) = rule_pack.fields.get(field) else {
        return Err(BogaeError::MissingField {
            field: format!("{entry_ref}.{field}"),
        });
    };
    resolve_mapping_string(value, field, entry_ref, tag_pack, tag_idx)
}

fn resolve_mapping_string(
    value: &Value,
    field: &str,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
) -> Result<String, BogaeError> {
    match value {
        Value::Str(text) => {
            let key = text.trim();
            if let Some(tag_key) = key.strip_prefix('$') {
                let Some(tag_value) = tag_pack.fields.get(tag_key) else {
                    return Err(BogaeError::MissingField {
                        field: format!("{}[{}].{}", TAG_LIST_CANON, tag_idx, tag_key),
                    });
                };
                match tag_value {
                    Value::Str(tag_text) => Ok(tag_text.clone()),
                    _ => Err(BogaeError::BadFieldType {
                        field: format!("{}[{}].{}", TAG_LIST_CANON, tag_idx, tag_key),
                        expected: "문자열",
                    }),
                }
            } else {
                Ok(text.clone())
            }
        }
        _ => Err(BogaeError::BadFieldType {
            field: format!("{entry_ref}.{field}"),
            expected: "문자열",
        }),
    }
}

fn read_mapping_color(
    rule_pack: &PackValue,
    entry_ref: &str,
    tag_pack: &PackValue,
    tag_idx: usize,
    pack: Option<&ColorNamePack>,
) -> Result<Rgba, BogaeError> {
    let candidates = ["색", "채움색", "color"];
    for field in candidates {
        if let Some(value) = rule_pack.fields.get(field) {
            let text = resolve_mapping_string(value, field, entry_ref, tag_pack, tag_idx)?;
            return canonicalize_color(&text, pack);
        }
    }
    Err(BogaeError::MissingField {
        field: format!("{entry_ref}.색"),
    })
}

fn read_pack_color(
    pack_value: &PackValue,
    entry_ref: &str,
    pack: Option<&ColorNamePack>,
) -> Result<Rgba, BogaeError> {
    let candidates = ["채움색", "색"];
    for name in candidates {
        if let Some(value) = read_pack_string_optional(pack_value, name, &format!("{entry_ref}.{name}"))? {
            return canonicalize_color(&value, pack);
        }
    }
    Err(BogaeError::MissingField {
        field: format!("{entry_ref}.색"),
    })
}

fn value_to_pixel_i32(value: &Value, field: &str) -> Result<i32, BogaeError> {
    let raw = match value {
        Value::Num(qty) => qty.raw,
        _ => {
            return Err(BogaeError::BadFieldType {
                field: field.to_string(),
                expected: "숫자",
            })
        }
    };
    let raw_bits = raw.raw();
    if raw_bits & 0xFFFF_FFFF != 0 {
        return Err(BogaeError::NonIntPixel {
            field: field.to_string(),
            value: raw,
        });
    }
    let int = (raw_bits >> Fixed64::SCALE_BITS) as i64;
    if int < 0 || int < i32::MIN as i64 || int > i32::MAX as i64 {
        return Err(BogaeError::NonIntPixel {
            field: field.to_string(),
            value: raw,
        });
    }
    Ok(int as i32)
}

fn parse_hex_color(value: &str) -> Option<Rgba> {
    let trimmed = value.trim();
    let hex = trimmed.strip_prefix('#')?;
    let bytes = match hex.len() {
        6 => {
            let mut out = [0u8; 8];
            out[..6].copy_from_slice(hex.as_bytes());
            out[6] = b'f';
            out[7] = b'f';
            out
        }
        8 => {
            let mut out = [0u8; 8];
            out.copy_from_slice(hex.as_bytes());
            out
        }
        _ => return None,
    };

    let r = hex_pair_to_u8(&bytes[0..2])?;
    let g = hex_pair_to_u8(&bytes[2..4])?;
    let b = hex_pair_to_u8(&bytes[4..6])?;
    let a = hex_pair_to_u8(&bytes[6..8])?;
    Some(Rgba { r, g, b, a })
}

fn collect_names(source: Option<&JsonValue>, rgba: &Rgba, map: &mut BTreeMap<String, Rgba>) {
    let Some(list) = source.and_then(|value| value.as_array()) else {
        return;
    };
    for name in list {
        let Some(text) = name.as_str() else {
            continue;
        };
        let key = normalize_name(text);
        map.insert(key, *rgba);
    }
}

fn normalize_name(name: &str) -> String {
    name.trim().to_lowercase()
}

fn hex_pair_to_u8(pair: &[u8]) -> Option<u8> {
    let text = std::str::from_utf8(pair).ok()?;
    u8::from_str_radix(text, 16).ok()
}

fn push_i32(out: &mut Vec<u8>, value: i32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn round_half_away_from_zero(value: f64) -> i32 {
    if value.is_sign_negative() {
        -((value.abs() + 0.5).floor() as i32)
    } else {
        (value + 0.5).floor() as i32
    }
}

fn round_f32_to_i32(value: f32) -> i32 {
    round_half_away_from_zero(value as f64)
}

fn q24_8_from_f32(value: f32) -> i32 {
    round_half_away_from_zero((value as f64) * 256.0)
}

fn push_q24_8(out: &mut Vec<u8>, value: f32) {
    let raw = q24_8_from_f32(value);
    out.extend_from_slice(&raw.to_le_bytes());
}

fn push_rgba(out: &mut Vec<u8>, color: Rgba) {
    out.push(color.r);
    out.push(color.g);
    out.push(color.b);
    out.push(color.a);
}

fn push_str(out: &mut Vec<u8>, text: &str) {
    let bytes = text.as_bytes();
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

fn take_bytes<'a>(
    bytes: &'a [u8],
    idx: &mut usize,
    len: usize,
) -> Result<&'a [u8], BogaeError> {
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err(BogaeError::DetbinParse {
            message: "detbin 길이가 부족함".to_string(),
        });
    }
    let out = &bytes[*idx..end];
    *idx = end;
    Ok(out)
}

fn take_bytes_bdl2<'a>(
    bytes: &'a [u8],
    idx: &mut usize,
    len: usize,
) -> Result<&'a [u8], BogaeError> {
    let end = idx.saturating_add(len);
    if end > bytes.len() {
        return Err(BogaeError::Bdl2Parse {
            code: "E_BDL2_TRUNCATED",
            message: "detbin 길이가 부족함".to_string(),
        });
    }
    let out = &bytes[*idx..end];
    *idx = end;
    Ok(out)
}

fn read_u8(bytes: &[u8], idx: &mut usize) -> Result<u8, BogaeError> {
    let raw = take_bytes(bytes, idx, 1)?;
    Ok(raw[0])
}

fn read_u8_bdl2(bytes: &[u8], idx: &mut usize) -> Result<u8, BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 1)?;
    Ok(raw[0])
}

fn read_u32(bytes: &[u8], idx: &mut usize) -> Result<u32, BogaeError> {
    let raw = take_bytes(bytes, idx, 4)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u16_bdl2(bytes: &[u8], idx: &mut usize) -> Result<u16, BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 2)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32_bdl2(bytes: &[u8], idx: &mut usize) -> Result<u32, BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 4)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_i32(bytes: &[u8], idx: &mut usize) -> Result<i32, BogaeError> {
    let raw = take_bytes(bytes, idx, 4)?;
    Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_q24_8_f32(bytes: &[u8], idx: &mut usize, _field: &str) -> Result<f32, BogaeError> {
    let raw = read_i32_bdl2(bytes, idx)?;
    Ok((raw as f32) / 256.0)
}

fn read_i32_bdl2(bytes: &[u8], idx: &mut usize) -> Result<i32, BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 4)?;
    Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_rgba(bytes: &[u8], idx: &mut usize) -> Result<Rgba, BogaeError> {
    let raw = take_bytes(bytes, idx, 4)?;
    Ok(Rgba {
        r: raw[0],
        g: raw[1],
        b: raw[2],
        a: raw[3],
    })
}

fn read_rgba_bdl2(bytes: &[u8], idx: &mut usize) -> Result<Rgba, BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 4)?;
    Ok(Rgba {
        r: raw[0],
        g: raw[1],
        b: raw[2],
        a: raw[3],
    })
}

fn read_string(bytes: &[u8], idx: &mut usize) -> Result<String, BogaeError> {
    let len = read_u32(bytes, idx)? as usize;
    let raw = take_bytes(bytes, idx, len)?;
    String::from_utf8(raw.to_vec()).map_err(|_| BogaeError::DetbinParse {
        message: "문자열 UTF-8 디코드 실패".to_string(),
    })
}

fn read_string_bdl2(bytes: &[u8], idx: &mut usize) -> Result<String, BogaeError> {
    let len = read_u32_bdl2(bytes, idx)? as usize;
    let raw = take_bytes_bdl2(bytes, idx, len)?;
    String::from_utf8(raw.to_vec()).map_err(|_| BogaeError::Bdl2Parse {
        code: "E_BDL2_UTF8",
        message: "문자열 UTF-8 디코드 실패".to_string(),
    })
}

fn read_hash32(bytes: &[u8], idx: &mut usize) -> Result<[u8; 32], BogaeError> {
    let raw = take_bytes(bytes, idx, 32)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(raw);
    Ok(out)
}

fn read_hash32_bdl2(bytes: &[u8], idx: &mut usize) -> Result<[u8; 32], BogaeError> {
    let raw = take_bytes_bdl2(bytes, idx, 32)?;
    let mut out = [0u8; 32];
    out.copy_from_slice(raw);
    Ok(out)
}
