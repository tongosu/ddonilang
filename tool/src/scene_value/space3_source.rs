//! Adopted finite read-only space3 source producer. Geometry is checked by the
//! existing exact kernel; this module supplies no evaluator, storage or IDs.
use super::space3_geometry::{CheckedCameraBasis3,CheckedFace3};
use ddonirang_core::{Fixed64,UnitDim,SourceSpan};
use ddonirang_lang::runtime::{Value,PlaneElement,SpaceCamera,SpaceScene};
use serde_json::{json,Value as Json};
fn vector(value:&Value,metres:bool)->Result<[Fixed64;3],String>{
 let Value::List(values)=value else{return Err("E_SPACE3_COORDINATE: 성분3개 차림이 필요합니다".into())};
 let [a,b,c]=values.as_slice() else{return Err("E_SPACE3_COORDINATE: 성분 수는3이어야 합니다".into())};
 let scalar=|v:&Value|match (metres,v){
  (true,Value::Unit(q)) if q.dim==UnitDim::LENGTH=>Ok(q.value),
  (false,Value::Fixed64(n))=>Ok(*n),
  _=>Err("E_SPACE3_UNIT: 자리는 m, 위 벡터는 무단위의 실제 Fixed64 값이어야 합니다".to_string()),
 };
 Ok([scalar(a)?,scalar(b)?,scalar(c)?])
}
pub(super) fn construct(name:&str,args:&[Value],mut meter:Option<&mut crate::work_cost::WorkCostMeter>)->Option<Result<Value,String>> {
 if !matches!(name,"표준장면.면"|"표준장면.카메라"|"표준장면.입체"){return None}
 Some((||match (name,args){
  ("표준장면.면",[Value::List(points)])=>{
   if let Some(m)=meter.as_deref_mut(){m.charge_element_visits(points.len()).map_err(|e|e.to_string())?;}
   let vertices=points.iter().map(|p|vector(p,true)).collect::<Result<Vec<_>,_>>()?;
   let checked=CheckedFace3::check(vertices,meter)?;
   Ok(Value::SceneElement(PlaneElement::Face3{vertices:checked.source_vertices().to_vec(),source:None}))
  },
  ("표준장면.카메라",[position,look_at,up,Value::String(projection),Value::Fixed64(fov)])=>{
   if projection!="원근"{return Err("E_SPACE3_PROJECTION_UNSUPPORTED: 첫 공간 R1은 #원근만 지원합니다".into())}
   let checked=CheckedCameraBasis3::check(vector(position,true)?,vector(look_at,true)?,vector(up,false)?,*fov,meter)?;
   let (position,look_at,up,fov)=checked.source_values();
   Ok(Value::SceneElement(PlaneElement::Camera3(SpaceCamera{position:*position,look_at:*look_at,up:*up,vertical_fov_degrees:fov,source:None})))
  },
  ("표준장면.입체",[Value::List(elements),Value::SceneElement(PlaneElement::Camera3(camera))])=>{
   if elements.is_empty(){return Err("E_SPACE3_ELEMENT: 첫 공간 R1에는 명시 면이 필요합니다".into())}
   if let Some(m)=meter.as_deref_mut(){m.charge_element_visits(elements.len()).map_err(|e|e.to_string())?;}
   let faces=elements.iter().map(|v|match v{Value::SceneElement(e@PlaneElement::Face3{..})=>Ok(e.clone()),_=>Err("E_SPACE3_ELEMENT: 첫 공간 R1은 checked 면 요소만 받습니다".to_string())}).collect::<Result<Vec<_>,_>>()?;
   Ok(Value::SpaceScene(SpaceScene{elements:faces,camera:camera.clone(),source:None}))
  },
  _=>Err("E_SPACE3_ARGUMENT: 면/카메라/입체의 실제 받는자리와 타입을 확인하세요".into()),
 })())
}
fn raw(point:&[Fixed64;3])->Vec<String>{point.iter().map(|x|x.raw_i64().to_string()).collect()}
fn source(span:Option<&SourceSpan>)->Json{span.map(|s|json!({"path":s.file,"start_line":s.start_line,"start_col":s.start_col,"end_line":s.end_line,"end_col":s.end_col})).unwrap_or(Json::Null)}
pub(super) fn project_face(vertices:&[[Fixed64;3]],span:Option<&SourceSpan>)->Json{
 json!({"kind":"face","vertices":vertices.iter().map(raw).collect::<Vec<_>>(),"source":source(span)})
}
pub(super) fn project_camera(c:&SpaceCamera)->Json{
 json!({"position":raw(&c.position),"look_at":raw(&c.look_at),"up":raw(&c.up),"projection":"perspective","vertical_fov_degrees_raw":c.vertical_fov_degrees.raw_i64().to_string(),"source":source(c.source.as_ref())})
}
pub(super) fn project_scene(s:&SpaceScene)->Json{
 json!({"schema":"ddn.scene_space3_projection.v1","kind":"scene","spatial_profile":"space3","profile":"ddn.scene.space3.metres.perspective.r1","coordinate_encoding":"fixed64_raw_decimal.v1","coordinate_unit":"m","camera":project_camera(&s.camera),"elements":s.elements.iter().map(super::project_element).collect::<Vec<_>>(),"source":source(s.source.as_ref())})
}
