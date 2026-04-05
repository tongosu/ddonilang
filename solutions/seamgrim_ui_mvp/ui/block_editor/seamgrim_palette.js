export const SEAMGRIM_PALETTE = {
  id: "seamgrim",
  label: "셈그림",
  categories: [
    {
      id: "charim",
      label: "채비",
      color: "#e67e22",
      blocks: [
        {
          kind: "charim_block",
          label: "채비 블록",
          template: "채비 {\n{items}\n}.",
          inputs: [{ id: "items", type: "statements" }],
        },
        {
          kind: "charim_item_const_step",
          label: "채비 매김(간격)",
          template: "{name}:{type_name} = ({value}) 매김 {\n  범위: {range}.\n  간격: {step}.\n}.",
          fields: [
            { id: "name", type: "text", default: "g" },
            { id: "type_name", type: "text", default: "수" },
            { id: "value", type: "number", default: "9.8" },
            { id: "range", type: "expr", default: "1..20" },
            { id: "step", type: "number", default: "0.1" },
          ],
        },
        {
          kind: "charim_item_const_split",
          label: "채비 매김(분할수)",
          template: "{name}:{type_name} = ({value}) 매김 {\n  범위: {range}.\n  분할수: {split_count}.\n}.",
          fields: [
            { id: "name", type: "text", default: "theta0" },
            { id: "type_name", type: "text", default: "수" },
            { id: "value", type: "number", default: "0.5" },
            { id: "range", type: "expr", default: "-1.2..1.2" },
            { id: "split_count", type: "number", default: "24" },
          ],
        },
        {
          kind: "charim_item_var",
          label: "채비 변수",
          template: "{name}:{type_name} <- {value}.",
          fields: [
            { id: "name", type: "text", default: "t" },
            { id: "type_name", type: "text", default: "변수" },
            { id: "value", type: "expr", default: "0" },
          ],
        },
        {
          kind: "charim_item_plain",
          label: "채비 값",
          template: "{name}:{type_name} = {value}.",
          fields: [
            { id: "name", type: "text", default: "안내" },
            { id: "type_name", type: "text", default: "글" },
            { id: "value", type: "expr", default: "\"초기\"" },
          ],
        },
      ],
    },
    {
      id: "flow",
      label: "실행흐름",
      color: "#f1c40f",
      blocks: [
        {
          kind: "seed_def",
          label: "씨앗 정의",
          template: "",
          fields: [
            { id: "params", type: "text", default: "" },
            { id: "name", type: "text", default: "기상청" },
            { id: "kind", type: "text", default: "임자" },
          ],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "hook_start",
          label: "(시작)할때",
          template: "(시작)할때 {\n{body}\n}.",
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "hook_tick",
          label: "(매마디)마다",
          template: "(매마디)마다 {\n{body}\n}.",
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "repeat",
          label: "되풀이",
          template: "되풀이 {\n{body}\n}.",
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "for_each",
          label: "대해 반복",
          template: "({item}) {iterable}에 대해 {\n{body}\n}.",
          fields: [
            { id: "item", type: "text", default: "x" },
            { id: "iterable", type: "expr", default: "x목록" },
          ],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "open_block",
          label: "너머",
          template: "너머 {\n{body}\n}.",
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "break_loop",
          label: "멈추기",
          template: "멈추기.",
        },
      ],
    },
    {
      id: "logic",
      label: "제어",
      color: "#2980b9",
      blocks: [
        {
          kind: "assign",
          label: "대입",
          template: "{target} <- {value}.",
          fields: [
            { id: "target", type: "text", default: "바탕.t" },
            { id: "value", type: "expr", default: "바탕.t + 1" },
          ],
        },
        {
          kind: "if_then",
          label: "조건 블록",
          template: "{cond} 일때 {\n{then}\n}.",
          fields: [{ id: "cond", type: "expr", default: "체력 < 30" }],
          inputs: [{ id: "then", type: "statements" }],
        },
        {
          kind: "if_else",
          label: "조건/아니면 블록",
          template: "{cond} 일때 {\n{then}\n} 아니면 {\n{else}\n}.",
          fields: [{ id: "cond", type: "expr", default: "체력 < 30" }],
          inputs: [
            { id: "then", type: "statements" },
            { id: "else", type: "statements" },
          ],
        },
        {
          kind: "while_block",
          label: "동안",
          template: "{cond} 동안 {\n{body}\n}.",
          fields: [{ id: "cond", type: "expr", default: "{ 값 < 3 }인것" }],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "contract_guard",
          label: "계약",
          template: "",
          fields: [
            { id: "cond", type: "expr", default: "{ 값 >= 0 }인것" },
            { id: "contract_kind", type: "text", default: "pre" },
            { id: "mode", type: "text", default: "abort" },
          ],
          inputs: [
            { id: "else", type: "statements" },
            { id: "then", type: "statements" },
          ],
        },
        {
          kind: "choose_branch",
          label: "고르기 가지",
          template: "{cond}: {\n{body}\n}",
          fields: [{ id: "cond", type: "expr", default: "{ x < 0 }인것" }],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "choose_else",
          label: "고르기",
          template: "고르기:\n{branches}\n  아니면: {\n{else}\n  }.",
          inputs: [
            { id: "branches", type: "statements" },
            { id: "else", type: "statements" },
          ],
        },
        {
          kind: "receive_block",
          label: "받으면",
          template: "",
          fields: [
            { id: "kind", type: "text", default: "알림" },
            { id: "binding", type: "text", default: "알림" },
            { id: "condition", type: "expr", default: "" },
          ],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "event_react",
          label: "알림 오면",
          template: "\"{kind}\"라는 알림이 오면 {\n{body}\n}.",
          fields: [{ id: "kind", type: "text", default: "tick" }],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "send_signal",
          label: "알림 보내기",
          template: "",
          fields: [
            { id: "sender", type: "expr", default: "" },
            { id: "payload", type: "expr", default: "(값:1) 첫알림" },
            { id: "receiver", type: "expr", default: "제" },
          ],
        },
        {
          kind: "return_value",
          label: "되돌림",
          template: "{value} 되돌림.",
          fields: [{ id: "value", type: "expr", default: "참" }],
        },
        {
          kind: "expr_stmt",
          label: "식 실행",
          template: "{expr}.",
          fields: [{ id: "expr", type: "expr", default: "() 증명" }],
        },
        {
          kind: "exec_policy_block",
          label: "실행정책",
          template: "",
          fields: [{ id: "body", type: "text", default: "\n  실행모드: 일반.\n  효과정책: 허용.\n" }],
        },
        {
          kind: "jjaim_block",
          label: "짜임",
          template: "",
          fields: [
            {
              id: "body",
              type: "text",
              default:
                "\n  형식: 점_틀.\n  입력 {\n    시작점: (수,수) <- (0.0, 0.0).\n  }.\n  출력 {\n    끝점: (수,수) <- (0.0, 0.0).\n  }.\n",
            },
          ],
        },
      ],
    },
    {
      id: "prompt",
      label: "질문",
      color: "#16a085",
      blocks: [
        {
          kind: "prompt_block",
          label: "?? 블록",
          template: "?? {\n{body}\n}",
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "prompt_after",
          label: "값 뒤 질문",
          template: "{value} ??: {\n{body}\n}",
          fields: [{ id: "value", type: "expr", default: "\"질문\"" }],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "prompt_condition",
          label: "조건 질문",
          template: "{cond} ??\n?? {\n{body}\n}",
          fields: [{ id: "cond", type: "expr", default: "{ 값 > 0 }인것" }],
          inputs: [{ id: "body", type: "statements" }],
        },
        {
          kind: "prompt_choose",
          label: "질문 고르기",
          template: "",
          inputs: [
            { id: "branches", type: "statements" },
            { id: "else", type: "statements" },
          ],
        },
      ],
    },
    {
      id: "output",
      label: "출력",
      color: "#8e44ad",
      blocks: [
        {
          kind: "show",
          label: "보여주기",
          template: "{expr} 보여주기.",
          fields: [{ id: "expr", type: "expr", default: "체력" }],
        },
        {
          kind: "inspect_value",
          label: "톺아보기",
          template: "{expr} 톺아보기.",
          fields: [{ id: "expr", type: "expr", default: "체력" }],
        },
        {
          kind: "bogae_draw",
          label: "보개로 그려",
          template: "보개로 그려.",
        },
        {
          kind: "bogae_madang_block",
          label: "보개마당",
          template: "",
          fields: [{ id: "body", type: "text", default: "\n  #자막(\"정본 테스트\").\n" }],
        },
      ],
    },
  ],
};

export function findPaletteBlock(palette, kind) {
  const categories = Array.isArray(palette?.categories) ? palette.categories : [];
  for (const category of categories) {
    const blocks = Array.isArray(category?.blocks) ? category.blocks : [];
    const hit = blocks.find((block) => String(block?.kind ?? "") === String(kind ?? ""));
    if (hit) {
      return {
        category,
        block: hit,
      };
    }
  }
  return null;
}
