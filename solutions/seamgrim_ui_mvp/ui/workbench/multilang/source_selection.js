// UTF-8 byte 범위를 textarea의 UTF-16 선택 위치로 표시한다. 원문/분석 입력은 바꾸지 않는다.
// textarea의 CRLF/CR 표시 정규화만 offset에 반영하며 문자 중간 범위는 추측 보정하지 않는다.
export function textareaSelectionForUtf8Span(source, span) {
  if (typeof source !== 'string' || span?.encoding !== 'utf-8-byte'
      || !Number.isSafeInteger(span.start) || !Number.isSafeInteger(span.end)
      || span.start < 0 || span.end < span.start) return null;
  const boundaries = new Map([[0,0]]);
  const encoder = new TextEncoder();
  let bytes = 0, utf16 = 0;
  for (const character of source) {
    bytes += encoder.encode(character).length;
    utf16 += character.length;
    boundaries.set(bytes, utf16);
  }
  const start = boundaries.get(span.start), end = boundaries.get(span.end);
  if (start === undefined || end === undefined) return null;
  const splitsNewline = index => source[index - 1] === '\r' && source[index] === '\n';
  if (splitsNewline(start) || splitsNewline(end)) return null;
  const displayedOffset = index => source.slice(0,index).replace(/\r\n?/g,'\n').length;
  return Object.freeze({start:displayedOffset(start),end:displayedOffset(end)});
}
