// Owner: call_5df482e22d354c89bcc05a608ad07a79, 2026-10-31.
export const WORKBENCH_UI = Object.freeze({mainActionBudget:10, overlayColor:'#a94d00', sideColor:'#145ea8',
  rightWidth:250, bottomFraction:.30, switchAdvantage:.10, startup:'last-saved-or-browse'});
export function chooseSideDock(width,height,ratio,current='right') {
  if (!(width>0 && height>0 && ratio>0)) return current;
  if (width<=720) return 'bottom';
  const area=(w,h)=>Math.min(w,h*ratio)**2/ratio;
  const right=area(Math.max(0,width-WORKBENCH_UI.rightWidth),height);
  const bottom=area(width,height*(1-WORKBENCH_UI.bottomFraction));
  const next=bottom>right?'bottom':'right';
  const now=current==='bottom'?bottom:right, best=next==='bottom'?bottom:right;
  return next===current || best>=now*(1+WORKBENCH_UI.switchAdvantage)?next:current;
}
export function popupPosition(anchor,box,viewport) {
  const pad=8, left=Math.max(viewport.left+pad,Math.min(anchor.left,viewport.left+viewport.width-box.width-pad));
  const below=anchor.bottom+4, above=anchor.top-box.height-4;
  const top=below+box.height<=viewport.top+viewport.height-pad?below:Math.max(viewport.top+pad,above);
  return {left,top};
}
export function completionAcceptsKey(event,visible,hasSelection,composing) {
  return !composing && !event.isComposing && event.keyCode!==229 && visible && hasSelection
    && !event.shiftKey && !event.altKey && ['Enter','Tab'].includes(event.key);
}

export function diagnosticLine(source,span) {
  if(!span || !Number.isSafeInteger(span.start) || !Number.isSafeInteger(span.end) || span.start<0 || span.end<span.start)return null;
  const bytes=new TextEncoder().encode(source);if(span.end>bytes.length)return null;
  try{return new TextDecoder('utf-8',{fatal:true}).decode(bytes.slice(0,span.start)).split('\n').length;}catch{return null;}
}

// A presentation veto on a supplied literal edit; never expands the producer's range.
export function completionWouldDuplicateSuffix(source,start,end,insert) {
 const prefix=source.slice(start,end),tail=insert.startsWith(prefix)?insert.slice(prefix.length):'';
 return Boolean(tail&&source.slice(end).startsWith(tail));
}

// Prefer a new committed result for the same current source; retain an edited
// temporary result when the old committed frame belongs to a different source.
export function temporaryOutputVisible(status,running,committedSourceCurrent){
  return Boolean(status&&status!=='not_produced'&&(running||!committedSourceCurrent));
}
