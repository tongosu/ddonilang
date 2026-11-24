//! Private validity kernel for RD519/RD538. No source registration, projection
//! enum, unit conversion, wire, identity, serialization or renderer is issued.
//! Future callers must supply checked unit-profile values before these checks.
use super::{GeometryMath, polygon};
use ddonirang_core::Fixed64;
use ddonirang_lang::runtime::Value;
use num_bigint::BigInt;
use num_traits::Zero;

type Point3 = [Fixed64; 3];
fn invalid() -> String { "E_SCENE_GEOMETRY: 명시 공간 기하와 카메라 기저를 확인하세요".into() }
fn difference(a: &Point3, b: &Point3, m: &mut GeometryMath<'_>) -> Result<Vec<BigInt>,String> {
    (0..3).map(|i| m.operation(&BigInt::from(a[i].raw_i64()), &BigInt::from(b[i].raw_i64()), '-')).collect()
}
fn cross(a: &[BigInt], b: &[BigInt], m: &mut GeometryMath<'_>) -> Result<Vec<BigInt>,String> {
    let mut out=Vec::with_capacity(3);
    for (i,j) in [(1,2),(2,0),(0,1)] {
        let left=m.operation(&a[i],&b[j],'*')?;
        let right=m.operation(&a[j],&b[i],'*')?;
        out.push(m.operation(&left,&right,'-')?);
    }
    Ok(out)
}
fn dot(a: &[BigInt], b: &[BigInt], m: &mut GeometryMath<'_>) -> Result<BigInt,String> {
    let mut value=BigInt::zero();
    for i in 0..3 {
        let term=m.operation(&a[i],&b[i],'*')?;
        value=m.operation(&value,&term,'+')?;
    }
    Ok(value)
}
#[derive(Debug)]
pub(crate) struct CheckedCameraBasis3 { position:Point3, look_at:Point3, up:Point3, fov:Fixed64 }
impl CheckedCameraBasis3 {
    pub(crate) fn check(position:Point3,look_at:Point3,up:Point3,fov:Fixed64,
        meter:Option<&mut crate::work_cost::WorkCostMeter>) -> Result<Self,String> {
        if !(fov>Fixed64::ZERO && fov<Fixed64::from_i64(180)) { return Err(invalid()); }
        let mut m=GeometryMath{meter};
        if let Some(meter)=m.meter.as_deref_mut() { meter.charge_element_visits(9).map_err(|e|e.to_string())?; }
        let direction=difference(&look_at,&position,&mut m)?;
        let up_raw=up.iter().map(|v|BigInt::from(v.raw_i64())).collect::<Vec<_>>();
        if direction.iter().all(Zero::is_zero) || up_raw.iter().all(Zero::is_zero)
            || cross(&direction,&up_raw,&mut m)?.iter().all(Zero::is_zero) { return Err(invalid()); }
        Ok(Self{position,look_at,up,fov})
    }
    pub(crate) fn source_values(&self)->(&Point3,&Point3,&Point3,Fixed64) {
        (&self.position,&self.look_at,&self.up,self.fov)
    }
}
#[derive(Debug)]
pub(crate) struct CheckedFace3 { vertices:Vec<Point3> }
impl CheckedFace3 {
    pub(crate) fn check(vertices:Vec<Point3>,meter:Option<&mut crate::work_cost::WorkCostMeter>) -> Result<Self,String> {
        if vertices.len()<3 { return Err(invalid()); }
        let mut m=GeometryMath{meter};
        let visits=vertices.len().checked_mul(3).ok_or_else(invalid)?;
        if let Some(meter)=m.meter.as_deref_mut() { meter.charge_element_visits(visits).map_err(|e|e.to_string())?; }
        for i in 0..vertices.len() { if vertices[i+1..].contains(&vertices[i]) { return Err(invalid()); } }
        let first=difference(&vertices[1],&vertices[0],&mut m)?;
        let mut normal=None;
        for point in &vertices[2..] {
            let next=difference(point,&vertices[0],&mut m)?;
            let n=cross(&first,&next,&mut m)?;
            if n.iter().any(|v|!v.is_zero()) { normal=Some(n);break; }
        }
        let normal=normal.ok_or_else(invalid)?;
        for point in &vertices[1..] {
            let delta=difference(point,&vertices[0],&mut m)?;
            if !dot(&normal,&delta,&mut m)?.is_zero() { return Err(invalid()); }
        }
        // A nonzero normal component proves this coordinate-plane projection
        // injective on the face. This is exact validity math only, not output.
        let dropped=normal.iter().position(|v|!v.is_zero()).ok_or_else(invalid)?;
        let axes=(0..3).filter(|i|*i!=dropped).collect::<Vec<_>>();
        let visits=vertices.len().checked_mul(2).ok_or_else(invalid)?;
        if let Some(meter)=m.meter.as_deref_mut() { meter.charge_element_visits(visits).map_err(|e|e.to_string())?; }
        let plane=vertices.iter().map(|p|Value::List(vec![Value::Fixed64(p[axes[0]]),Value::Fixed64(p[axes[1]])])).collect::<Vec<_>>();
        polygon(&plane,&mut m)?;
        Ok(Self{vertices})
    }
    pub(crate) fn source_vertices(&self)->&[Point3] { &self.vertices }
}
#[cfg(test)] mod tests {
    use super::*;
    fn p(x:i64,y:i64,z:i64)->Point3 { [x,y,z].map(Fixed64::from_i64) }
    #[test] fn camera_preserves_authored_values() {
        let (a,b,u)=(p(4,3,5),p(0,0,0),p(0,0,1));let f=Fixed64::from_i64(60);
        let c=CheckedCameraBasis3::check(a,b,u,f,None).unwrap();assert_eq!(c.source_values(),(&a,&b,&u,f));
    }
    #[test] fn invalid_camera_never_gets_repaired() {
        for (a,b,u,f) in [(p(0,0,0),p(0,0,0),p(0,1,0),60),(p(0,0,3),p(0,0,0),p(0,0,0),60),(p(0,0,3),p(0,0,0),p(0,0,1),60),(p(0,0,3),p(0,0,0),p(0,1,0),0),(p(0,0,3),p(0,0,0),p(0,1,0),180)] {
            assert!(CheckedCameraBasis3::check(a,b,u,Fixed64::from_i64(f),None).is_err());
        }
    }
    #[test] fn sloped_face_order_is_preserved() {
        let v=vec![p(0,0,0),p(1,0,1),p(0,1,2)];let result=CheckedFace3::check(v.clone(),None).unwrap();assert_eq!(result.source_vertices(),v);
    }
    #[test] fn nonplanar_crossing_degenerate_and_duplicate_faces_reject() {
        for v in [vec![p(0,0,0),p(1,0,0),p(1,1,1),p(0,1,0)],vec![p(0,0,0),p(2,2,0),p(0,2,0),p(2,0,0)],vec![p(0,0,0),p(1,1,1),p(2,2,2)],vec![p(0,0,0),p(1,0,1),p(0,0,0)]] { assert!(CheckedFace3::check(v,None).is_err()); }
    }
}
