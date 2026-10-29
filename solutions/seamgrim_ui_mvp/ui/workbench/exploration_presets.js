// A3-P5: explicit author edits, not runtime lowering or hidden world inputs.
export function applyExplorationPreset(source, parameters, values) {
  let next=source;
  for(const [name,value] of Object.entries(values)) {
    const rule=parameters[name];
    if(!rule||!Number.isFinite(value)||value<rule.min||value>rule.max||(rule.integer&&!Number.isInteger(value)))throw new Error(`E_EXPLORATION_PRESET: ${name}의 허용 범위를 확인하세요.`);
    const escaped=name.replace(/[.*+?^${}()|[\]\\]/g,'\\$&');
    const pattern=new RegExp(`^${escaped}[ \\t]*:=[ \\t]*-?(?:[0-9]+(?:\\.[0-9]+)?)[ \\t]*\\.[ \\t]*$`,'gm');
    const matches=[...next.matchAll(pattern)];
    if(matches.length!==1)throw new Error(`E_EXPLORATION_PRESET: ${name}의 초기값 줄이 바뀌었습니다. 원문에서 직접 수정하세요.`);
    next=next.replace(pattern,`${name} := ${value}.`);
  }
  return next;
}
