# seamgrim.space2d.v0

2D 보개(공간) v0 스키마 요약.

## 기본 구조
- `schema`: "seamgrim.space2d.v0"
- `points?`: `{ x, y }[]` (선택)
- `shapes?`: `{ kind, ... }[]` (선택)
- `drawlist?`: `draw item[]` 또는 `{ items: draw item[], camera? }`
- `camera?`: `{ x_min, x_max, y_min, y_max, pan_x?, pan_y?, zoom? }`
- `meta?`: `{ title?, desc? }`

`points` 또는 `shapes` 또는 `drawlist` 중 하나는 반드시 존재해야 한다.

## shapes
- `kind`: `"line" | "circle" | "point"`
- `id?`: 도형 식별자(선택)
- `token?`: 장면 토큰 매칭용 키(선택)
- `label?`: 표시용 라벨(선택)

### line
- `x1`, `y1`, `x2`, `y2`: 숫자 좌표
- `stroke?`: 색상 문자열
- `width?`: 선 두께(월드 좌표 기준)

### circle
- `x`, `y`: 중심 좌표
- `r`: 반지름(월드 좌표 기준)
- `fill?`: 채움 색상
- `stroke?`: 테두리 색상
- `width?`: 테두리 두께

### point
- `x`, `y`: 좌표
- `size?`: 점 크기(월드 좌표 기준)
- `color?`: 색상

## drawlist (C+ primitive)
- `kind`: `"line" | "circle" | "point" | "polyline" | "rect" | "polygon" | "text" | "arrow"`
- `token?`: 장면 토큰 매칭용 키(선택)
- 공통 스타일: `stroke?`, `fill?`, `width?`, `size?`, `opacity?`

### polyline/polygon
- `points`: `{x,y}[]` 또는 `[[x,y], ...]`

### rect
- `x1`, `y1`, `x2`, `y2` 또는 `x`, `y`, `width`, `height`

### text
- `text`, `x`, `y`, `size?`, `color?`, `align?`

### arrow
- `x1`, `y1`, `x2`, `y2`: 시작/끝 좌표
- `head_size?`: 화살촉 크기(기본 8)
- `label?`: 화살표 옆 텍스트 (예: `"v"`, `"F=mg"`)
- `label_offset?`: `{ dx, dy }` (기본 `{0, -8}`)
- `color?`: 화살표/라벨 색상
- `stroke?`: 선 색상
- `style?`: `"solid" | "dashed" | "dotted"` (기본 `"solid"`)
- `width?`: 선 두께
- `opacity?`: 투명도 0.0~1.0
