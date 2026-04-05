import { SEAMGRIM_PALETTE } from "./seamgrim_palette.js";

export const RPGBOX_PALETTE = {
  id: "rpgbox",
  label: "RPG 박스",
  categories: [
    ...SEAMGRIM_PALETTE.categories,
    {
      id: "imja",
      label: "임자",
      color: "#e74c3c",
      blocks: [
        {
          kind: "alrimsi_define",
          label: "알림씨 정의",
          template: "({fields}) {name}:알림씨 = {\n}.",
          fields: [
            { id: "name", type: "text", default: "피해_받음" },
            { id: "fields", type: "text", default: "피해량:수" },
          ],
        },
        {
          kind: "alrimsi_send",
          label: "알림씨 보내기",
          template: "({payload}) {alrim} ~~> {target}.",
          fields: [
            { id: "payload", type: "expr", default: "피해량:30" },
            { id: "alrim", type: "text", default: "피해_받음" },
            { id: "target", type: "text", default: "플레이어" },
          ],
        },
        {
          kind: "receive_hook",
          label: "알림씨 받으면",
          template: "({bind})인 {alrim}을 받으면 {\n{body}\n}.",
          fields: [
            { id: "bind", type: "text", default: "피해량" },
            { id: "alrim", type: "text", default: "피해_받음" },
          ],
          inputs: [{ id: "body", type: "statements" }],
        },
      ],
    },
    {
      id: "state",
      label: "상태",
      color: "#e91e63",
      blocks: [
        {
          kind: "clamp_min",
          label: "최솟값 유지",
          template: "({expr}) 최소 {min}",
          fields: [
            { id: "expr", type: "expr", default: "제.체력 - 알림.정보.피해량" },
            { id: "min", type: "number", default: "0" },
          ],
        },
        {
          kind: "clamp_max",
          label: "최댓값 유지",
          template: "({expr}) 최대 {max}",
          fields: [
            { id: "expr", type: "expr", default: "제.체력 + 알림.정보.회복량" },
            { id: "max", type: "text", default: "제.체력_최대" },
          ],
        },
      ],
    },
  ],
};
