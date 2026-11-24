import {popupPosition} from './policy.mjs';
// Textarea coordinates only; never parses or rewrites DDN.
export function placeCompletionAtCaret(editor,popup,offset) {
  const style=getComputedStyle(editor),rect=editor.getBoundingClientRect(),mirror=document.createElement('div');
  for(const name of ['font','fontSize','fontFamily','fontWeight','fontStyle','lineHeight','letterSpacing','tabSize','padding','border','boxSizing','wordSpacing','textIndent']) mirror.style[name]=style[name];
  Object.assign(mirror.style,{position:'fixed',left:`${rect.left}px`,top:`${rect.top}px`,width:`${rect.width}px`,height:'auto',visibility:'hidden',whiteSpace:'pre-wrap',overflowWrap:'break-word',pointerEvents:'none'});
  mirror.textContent=editor.value.slice(0,offset);
  const caret=document.createElement('span');caret.textContent='​';mirror.append(caret);document.body.append(mirror);
  const r=caret.getBoundingClientRect(),lineHeight=parseFloat(style.lineHeight)||parseFloat(style.fontSize)*1.6;
  const anchor={left:r.left-editor.scrollLeft,top:r.top-editor.scrollTop,bottom:r.top-editor.scrollTop+lineHeight};mirror.remove();
  const vv=window.visualViewport,viewport={left:vv?.offsetLeft??0,top:vv?.offsetTop??0,width:vv?.width??innerWidth,height:vv?.height??innerHeight};
  popup.style.maxHeight=`${Math.max(44,Math.min(220,viewport.height-32))}px`;
  popup.style.maxWidth=`${Math.max(120,Math.min(420,viewport.width-16))}px`;
  const point=popupPosition(anchor,popup.getBoundingClientRect(),viewport);
  popup.style.left=`${point.left}px`;popup.style.top=`${point.top}px`;
}
