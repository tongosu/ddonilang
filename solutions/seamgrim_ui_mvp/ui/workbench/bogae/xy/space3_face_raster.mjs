// Opaque, two-sided face display only. Face indices are positions in the
// supplied internal list, never producer/object identities. No scene admission,
// surface physics, source mutation, lighting or observation inference occurs.
const PIXEL_LIMIT = 8 * 1024 * 1024, OPERATION_LIMIT = 32 * 1024 * 1024;
const finiteVector = (v,n) => Array.isArray(v) && v.length === n && v.every(Number.isFinite);
const edge = (a,b,p) => (b[0]-a[0])*(p[1]-a[1])-(b[1]-a[1])*(p[0]-a[0]);
function triangulate(vertices, tick) {
  const normal=[0,0,0];
  for(let i=0;i<vertices.length;i++) {
    tick();const a=vertices[i],b=vertices[(i+1)%vertices.length];
    normal[0]+=(a[1]-b[1])*(a[2]+b[2]);
    normal[1]+=(a[2]-b[2])*(a[0]+b[0]);
    normal[2]+=(a[0]-b[0])*(a[1]+b[1]);
  }
  if(!normal.every(Number.isFinite)) return null;
  const drop=normal.reduce((best,n,i)=>Math.abs(n)>Math.abs(normal[best])?i:best,0);
  if(normal[drop]===0)return null;
  const points=vertices.map(p=>p.filter((_,i)=>i!==drop));
  let area=0;
  for(let i=0;i<points.length;i++)area+=edge([0,0],points[i],points[(i+1)%points.length]);
  if(!Number.isFinite(area)||area===0)return null;
  const sign=Math.sign(area),indices=points.map((_,i)=>i),triangles=[];
  while(indices.length>3) {
    let found=false;
    for(let k=0;k<indices.length;k++) {
      tick();const a=indices[(k+indices.length-1)%indices.length],b=indices[k],c=indices[(k+1)%indices.length];
      const turn=sign*edge(points[a],points[b],points[c]);
      if(turn===0){indices.splice(k,1);found=true;break;}
      if(!(Number.isFinite(turn)&&turn>0))continue;
      let blocked=false;
      for(const index of indices) {
        tick();if(index===a||index===b||index===c)continue;
        if(sign*edge(points[a],points[b],points[index])>=0
          &&sign*edge(points[b],points[c],points[index])>=0
          &&sign*edge(points[c],points[a],points[index])>=0){blocked=true;break;}
      }
      if(blocked)continue;
      triangles.push([a,b,c]);indices.splice(k,1);found=true;break;
    }
    if(!found)return null;
  }
  if(indices.length!==3||sign*edge(...indices.map(i=>points[i]))<=0)return null;
  triangles.push([...indices]);return triangles;
}
function clipTriangle(vertices,planes,tick) {
  let polygon=vertices;
  const dot=(a,b)=>a.reduce((s,n,i)=>s+n*b[i],0);
  for(const plane of planes) {
    if(!polygon.length)break;
    const result=[];
    for(let i=0;i<polygon.length;i++) {
      tick();const a=polygon[i],b=polygon[(i+1)%polygon.length],fa=dot(a,plane),fb=dot(b,plane);
      if(!Number.isFinite(fa)||!Number.isFinite(fb))return null;
      if(fa>=0)result.push(a);
      if((fa<0)!==(fb<0)) {
        const scale=Math.max(Math.abs(fa),Math.abs(fb)),left=Math.abs(fa)/scale,right=Math.abs(fb)/scale;
        const t=left/(left+right),point=a.map((n,j)=>(1-t)*n+t*b[j]);
        if(!point.every(Number.isFinite))return null;
        result.push(point);
      }
    }
    polygon=result;
  }
  return polygon;
}

export function rasterizeSpace3Faces(frame,width,height,{colors}={}) {
  if(!Number.isSafeInteger(width)||!Number.isSafeInteger(height)||width<=0||height<=0
    ||width*height>PIXEL_LIMIT)throw new RangeError('면 표시 픽셀 한도를 벗어났습니다.');
  if(!finiteVector(frame?.matrix,16)||!Array.isArray(frame.samples)||!Array.isArray(frame.faceRanges)
    ||!Array.isArray(frame.clipPlanes)||!frame.clipPlanes.length
    ||!frame.clipPlanes.every(p=>finiteVector(p,4)&&p.some(n=>n!==0))||![1,-1].includes(frame.yDirection))
    throw new TypeError('명시한 면 표시 자료가 필요합니다.');
  if(colors!==undefined&&(!Array.isArray(colors)||colors.length!==frame.faceRanges.length
    ||!colors.every(c=>finiteVector(c,4)&&c.every(n=>Number.isInteger(n)&&n>=0&&n<=255)&&c[3]===255)))
    throw new TypeError('면 표시 색은 불투명 RGBA여야 합니다.');
  let operations=0;
  const tick=()=>{if(++operations>OPERATION_LIMIT)throw new RangeError('면 표시 계산 한도를 벗어났습니다.');};
  const pixels=new Uint8ClampedArray(width*height*4),inverseDepth=new Float64Array(width*height);
  const faceAtPixel=new Int32Array(width*height);faceAtPixel.fill(-1);
  const limitations=[];let triangleCount=0,paintedPixels=0,zeroProjectedAreaCount=0;
  const matrix=frame.matrix;
  const transform=point=>Array.from({length:4},(_,i)=>matrix[i*4]*point[0]+matrix[i*4+1]*point[1]+matrix[i*4+2]*point[2]+matrix[i*4+3]);
  for(const [faceIndex,range] of frame.faceRanges.entries()) {
    if(!range||!Number.isSafeInteger(range.start)||!Number.isSafeInteger(range.count)||range.start<0||range.count<3
      ||range.start+range.count>frame.samples.length)throw new TypeError('면의 내부 꼭짓점 범위를 확인하세요.');
    const vertices=frame.samples.slice(range.start,range.start+range.count);
    if(!vertices.every(v=>finiteVector(v,3))){limitations.push({faceIndex,reason:'invalid_display_vertices'});continue;}
    const triangles=triangulate(vertices,tick);
    if(!triangles){limitations.push({faceIndex,reason:'triangulation_precision'});continue;}
    const transformed=vertices.map(transform),color=colors?.[faceIndex]??[33,91,139,255];
    for(const indices of triangles) {
      const triangle=indices.map(i=>transformed[i]);
      if(!triangle.every(v=>v.every(Number.isFinite))){limitations.push({faceIndex,reason:'non_finite_transform'});continue;}
      const clipped=clipTriangle(triangle,frame.clipPlanes,tick);
      if(!clipped){limitations.push({faceIndex,reason:'clip_precision'});continue;}
      for(let k=1;k+1<clipped.length;k++) {
        const raw=[clipped[0],clipped[k],clipped[k+1]];
        if(raw.some(v=>!(v[3]>0))){limitations.push({faceIndex,reason:'camera_plane_precision'});continue;}
        const screen=raw.map(v=>[(v[0]/v[3]+1)*width/2,(1+frame.yDirection*v[1]/v[3])*height/2,1/v[3]]);
        if(!screen.every(v=>v.every(Number.isFinite))){limitations.push({faceIndex,reason:'screen_precision'});continue;}
        const [a,b,c]=screen,area=edge(a,b,c);
        if(!Number.isFinite(area)){limitations.push({faceIndex,reason:'screen_area_precision'});continue;}
        if(area===0){zeroProjectedAreaCount++;continue;}
        triangleCount++;
        const xmin=Math.max(0,Math.floor(Math.min(a[0],b[0],c[0]))),xmax=Math.min(width-1,Math.ceil(Math.max(a[0],b[0],c[0])));
        const ymin=Math.max(0,Math.floor(Math.min(a[1],b[1],c[1]))),ymax=Math.min(height-1,Math.ceil(Math.max(a[1],b[1],c[1])));
        for(let y=ymin;y<=ymax;y++)for(let x=xmin;x<=xmax;x++) {
          tick();const point=[x+.5,y+.5],wa=edge(b,c,point)/area,wb=edge(c,a,point)/area,wc=edge(a,b,point)/area;
          if(wa<0||wb<0||wc<0)continue;
          const depth=wa*a[2]+wb*b[2]+wc*c[2],pixel=y*width+x;
          if(!(Number.isFinite(depth)&&depth>inverseDepth[pixel]))continue;
          if(faceAtPixel[pixel]===-1)paintedPixels++;
          inverseDepth[pixel]=depth;faceAtPixel[pixel]=faceIndex;pixels.set(color,pixel*4);
        }
      }
    }
  }
  return {pixels,faceAtPixel,paintedPixels,triangleCount,zeroProjectedAreaCount,limitations};
}
