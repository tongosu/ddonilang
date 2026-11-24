// Ordinary space2d view range only. Authored renderers do not consume this camera.
// Works whose keys are action inputs (games) turn keyboard zoom/pan off (WO17 R1.2 A.8);
// wheel, middle drag and the reset button stay.
const actionKeyWork=()=>{const root=globalThis.document?.querySelector?.('[data-testid="approved-workbench-root"]');return Number(root?.dataset.projectActionCount||0)>0||root?.dataset.executionOwner==="logical_input";};
export function installStageNavigation({canvas,resetButton,view,draw}) {
  let drag=null,space=false,suppressClick=false;
  const supported=()=>Boolean(canvas.dataset.navigationSurface==='ordinary-space2d'&&canvas.dataset.viewRange&&view.range);
  const refresh=()=>{resetButton.hidden=!supported()||view.userMoved!==true;};
  function apply(range){
    if(!Object.values(range).every(Number.isFinite)||range.xMax<=range.xMin||range.yMax<=range.yMin)return;
    view.range=range;view.mode='fixed';view.userMoved=true;draw();refresh();
  }
  function zoom(factor){
    if(!supported())return;
    const r=view.range,cx=(r.xMin+r.xMax)/2,cy=(r.yMin+r.yMax)/2;
    const x=(r.xMax-r.xMin)*factor/2,y=(r.yMax-r.yMin)*factor/2;
    apply({xMin:cx-x,xMax:cx+x,yMin:cy-y,yMax:cy+y});
  }
  const down=e=>{
    if(!supported()||!(e.button===1||(e.button===0&&space)))return;
    e.preventDefault();canvas.focus();canvas.setPointerCapture(e.pointerId);
    drag={id:e.pointerId,x:e.clientX,y:e.clientY,range:{...view.range},moved:false};
  };
  const move=e=>{
    if(!drag||e.pointerId!==drag.id)return;
    const box=canvas.getBoundingClientRect();if(box.width<=0||box.height<=0)return;
    const dx=e.clientX-drag.x,dy=e.clientY-drag.y,r=drag.range;
    if(Math.abs(dx)+Math.abs(dy)<3&&!drag.moved)return;
    drag.moved=true;
    const x=-dx*(r.xMax-r.xMin)/box.width,y=dy*(r.yMax-r.yMin)/box.height;
    apply({xMin:r.xMin+x,xMax:r.xMax+x,yMin:r.yMin+y,yMax:r.yMax+y});
  };
  const up=e=>{if(!drag||e.pointerId!==drag.id)return;suppressClick=drag.moved;drag=null;if(canvas.hasPointerCapture(e.pointerId))canvas.releasePointerCapture(e.pointerId);};
  const wheel=e=>{if(!supported())return;e.preventDefault();zoom(e.deltaY<0?1/1.1:1.1);};
  const key=e=>{
    if(e.isComposing||e.ctrlKey||e.metaKey||actionKeyWork())return;
    if(e.code==='Space'){space=true;if(supported())e.preventDefault();return;}
    if(!supported())return;
    const pan=e.altKey&&['ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(e.key);
    if(['+','=','-','Home'].includes(e.key)||pan)e.preventDefault();
    if(e.key==='+'||e.key==='=')zoom(1/1.1);
    else if(e.key==='-')zoom(1.1);
    else if(e.key==='Home')reset();
    else if(pan){
      const r=view.range,x=(r.xMax-r.xMin)/10,y=(r.yMax-r.yMin)/10;
      const dx=e.key==='ArrowLeft'?-x:e.key==='ArrowRight'?x:0,dy=e.key==='ArrowDown'?-y:e.key==='ArrowUp'?y:0;
      apply({xMin:r.xMin+dx,xMax:r.xMax+dx,yMin:r.yMin+dy,yMax:r.yMax+dy});
    }
  };
  // After reset the stage takes focus back, so Space/Enter go to the work and not to the reset button.
  function reset(){if(!view.initialRange)return;view.range={...view.initialRange};view.mode='fixed';view.userMoved=false;draw();refresh();canvas.focus();}
  const keyup=e=>{if(e.code==='Space')space=false;};
  const blur=()=>{space=false;if(drag&&canvas.hasPointerCapture(drag.id))canvas.releasePointerCapture(drag.id);drag=null;};
  const click=e=>{if(suppressClick){e.preventDefault();e.stopImmediatePropagation();suppressClick=false;}};
  canvas.addEventListener('pointerdown',down);canvas.addEventListener('pointermove',move);canvas.addEventListener('pointerup',up);canvas.addEventListener('pointercancel',up);
  canvas.addEventListener('wheel',wheel,{passive:false});canvas.addEventListener('keydown',key);canvas.addEventListener('keyup',keyup);canvas.addEventListener('blur',blur);canvas.addEventListener('click',click,true);
  resetButton.addEventListener('click',reset);
  const observer=new MutationObserver(refresh);observer.observe(canvas,{attributes:true,attributeFilter:['data-view-range','data-navigation-surface']});refresh();
  return {refresh,dispose(){observer.disconnect();for(const [name,fn] of [['pointerdown',down],['pointermove',move],['pointerup',up],['pointercancel',up],['wheel',wheel],['keydown',key],['keyup',keyup],['blur',blur],['click',click]])canvas.removeEventListener(name,fn,name==='click');resetButton.removeEventListener('click',reset);}};
}
