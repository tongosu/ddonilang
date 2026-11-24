// Pixel placement only; callers supply the committed renderer's target position.
export function clampFollowingPanel({x,y,width,height,availableWidth,availableHeight}) {
  const values=[x,y,width,height,availableWidth,availableHeight];
  if(!values.every(Number.isFinite)||width<0||height<0||availableWidth<0||availableHeight<0)return {left:0,top:0};
  const right=Math.max(0,availableWidth-width-4),bottom=Math.max(0,availableHeight-height-4);
  const leftCandidate=x+12+width<=availableWidth-4?x+12:x-width-12;
  return {left:Math.max(Math.min(4,right),Math.min(right,leftCandidate)),top:Math.max(Math.min(4,bottom),Math.min(bottom,y-height/2))};
}

// Place the panel outside the target's drawn bounding box, `gap` px away.
// Order: right, below, left, above. Each side keeps the panel inside the
// available area (4px inset). Without a usable box or room on any side the
// existing centre rule is used and `side` is "fallback".
export function placeOutsideTarget({box,width,height,availableWidth,availableHeight,gap=8}) {
  const values=[width,height,availableWidth,availableHeight,box?.left,box?.top,box?.right,box?.bottom];
  if(!values.every(Number.isFinite)||width<0||height<0||box.right<box.left||box.bottom<box.top)
    return {...clampFollowingPanel({x:(box?.left+box?.right)/2,y:(box?.top+box?.bottom)/2,width,height,availableWidth,availableHeight}),side:'fallback'};
  const minX=4,minY=4,maxX=availableWidth-width-4,maxY=availableHeight-height-4;
  const clamp=(v,lo,hi)=>Math.max(lo,Math.min(hi,v));
  const cx=(box.left+box.right)/2,cy=(box.top+box.bottom)/2;
  const candidates=[
    ['right',box.right+gap,clamp(cy-height/2,minY,maxY)],
    ['below',clamp(cx-width/2,minX,maxX),box.bottom+gap],
    ['left',box.left-gap-width,clamp(cy-height/2,minY,maxY)],
    ['above',clamp(cx-width/2,minX,maxX),box.top-gap-height],
  ];
  for(const [side,left,top] of candidates){
    if(maxX<minX||maxY<minY)break;
    if(left<minX-0.5||left>maxX+0.5||top<minY-0.5||top>maxY+0.5)continue;
    return {left,top,side};
  }
  return {...clampFollowingPanel({x:cx,y:cy,width,height,availableWidth,availableHeight}),side:'fallback'};
}
