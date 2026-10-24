//! RD-538 subset: exact typed circle/text/plane/view construction.
//! Coordinate/range payload uses existing two-number collections; no unit
//! coercion, fitting, camera inference or fallback geometry.
use ddonirang_core::Fixed64;
use ddonirang_lang::runtime::{PlaneElement, PlaneScene, Value, ViewValue};
use serde_json::{json, Value as Json};
use num_bigint::BigInt;
use num_traits::Zero;

struct GeometryMath<'a> {meter: Option<&'a mut crate::work_cost::WorkCostMeter>}
impl GeometryMath<'_> {
    fn operation(&mut self,a:&BigInt,b:&BigInt,op:char)->Result<BigInt,String> {
        let result=match op {'+'=>a+b,'-'=>a-b,'*'=>a*b,_=>unreachable!()};
        if let Some(meter)=self.meter.as_deref_mut() {
            meter.charge_bigint_rational_primitive(&[a,b],&[&result]).map_err(|error|error.to_string())?;
        }
        Ok(result)
    }
}

fn orientation(a: &[Fixed64;2], b: &[Fixed64;2], c: &[Fixed64;2],math:&mut GeometryMath<'_>) -> Result<BigInt,String> {
    let raw = |value: Fixed64| BigInt::from(value.raw_i64());
    let dx=math.operation(&raw(b[0]),&raw(a[0]),'-')?;
    let dy=math.operation(&raw(b[1]),&raw(a[1]),'-')?;
    let ex=math.operation(&raw(c[0]),&raw(a[0]),'-')?;
    let ey=math.operation(&raw(c[1]),&raw(a[1]),'-')?;
    let first=math.operation(&dx,&ey,'*')?;
    let second=math.operation(&dy,&ex,'*')?;
    math.operation(&first,&second,'-')
}
fn on_segment(a: &[Fixed64;2], b: &[Fixed64;2], c: &[Fixed64;2],math:&mut GeometryMath<'_>) -> Result<bool,String> {
    // F5-GA[SCENE_AXES]: All three points are length-two Rust arrays and axis ranges only over 0..2.
    Ok(orientation(a,b,c,math)?.is_zero() && (0..2).all(|axis|
        a[axis].min(b[axis]) <= c[axis] && c[axis] <= a[axis].max(b[axis])))
}
fn segments_intersect(a: &[Fixed64;2], b: &[Fixed64;2], c: &[Fixed64;2], d: &[Fixed64;2],math:&mut GeometryMath<'_>) -> Result<bool,String> {
    let (ab_c,ab_d,cd_a,cd_b)=(orientation(a,b,c,math)?,orientation(a,b,d,math)?,orientation(c,d,a,math)?,orientation(c,d,b,math)?);
    Ok((ab_c.sign()!=ab_d.sign() && !ab_c.is_zero() && !ab_d.is_zero()
        && cd_a.sign()!=cd_b.sign() && !cd_a.is_zero() && !cd_b.is_zero())
        || on_segment(a,b,c,math)? || on_segment(a,b,d,math)? || on_segment(c,d,a,math)? || on_segment(c,d,b,math)?)
}
fn polygon(vertices: &[Value],math:&mut GeometryMath<'_>) -> Result<PlaneElement,String> {
    // F5-GA[SCENE_VERTICES]: Rejecting len<3 precedes modular indexing; immutable points and bounded i/j loops keep successors and modulo results in range.
    let points=vertices.iter().map(pair).collect::<Result<Vec<_>,_>>()?;
    let invalid=||"E_SCENE_GEOMETRY: 다각형은 서로 다른 꼭짓점 셋 이상과 단순한 넓이 있는 도형이어야 합니다".to_string();
    if points.len()<3 {return Err(invalid());}
    let mut area=BigInt::zero();
    for i in 0..points.len() {
        let a=&points[i]; let b=&points[(i+1)%points.len()];
        let first=math.operation(&BigInt::from(a[0].raw_i64()),&BigInt::from(b[1].raw_i64()),'*')?;
        let second=math.operation(&BigInt::from(a[1].raw_i64()),&BigInt::from(b[0].raw_i64()),'*')?;
        let term=math.operation(&first,&second,'-')?;
        area=math.operation(&area,&term,'+')?;
        for j in i+1..points.len() {
            if points[i]==points[j] {return Err(invalid());}
            // Adjacent edges share only their common vertex. A fold-back is
            // invalid even when a separate part gives the polygon nonzero area.
            if j==i+1 {
                let c=&points[(j+1)%points.len()];
                if on_segment(a,b,c,math)? || on_segment(b,c,a,math)? {return Err(invalid());}
            } else if i==0 && j==points.len()-1 {
                let c=&points[j];
                if on_segment(c,a,b,math)? || on_segment(a,b,c,math)? {return Err(invalid());}
            } else if segments_intersect(a,b,&points[j],&points[(j+1)%points.len()],math)? {return Err(invalid());}
        }
    }
    if area.is_zero() {return Err(invalid());}
    Ok(PlaneElement::Polygon{vertices:points})
}

fn pair(value: &Value) -> Result<[Fixed64; 2], String> {
    let Value::List(values) = value else { return Err("E_SCENE_COORDINATE: 두 수의 차림이 필요합니다".into()); };
    match values.as_slice() {
        [Value::Fixed64(a), Value::Fixed64(b)] => Ok([*a,*b]),
        _ => Err("E_SCENE_COORDINATE: 차원 2의 유한 무단위 수 차림이 필요합니다".into()),
    }
}

pub fn clone_visits(value: &Value) -> usize {
    match value {
        Value::SceneElement(PlaneElement::Polygon{vertices}) => vertices.len(),
        Value::Scene(scene) => scene_clone_visits(scene),
        Value::View(view) => view_clone_visits(view),
        Value::List(items) => items.iter().map(clone_visits).sum(),
        Value::Pack(fields) | Value::Set(fields) => fields.values().map(clone_visits).sum(),
        Value::Map(entries) => entries.values().map(|entry| clone_visits(&entry.key) + clone_visits(&entry.value)).sum(),
        _ => 0,
    }
}
fn scene_clone_visits(scene:&PlaneScene)->usize {
    scene.elements.iter().map(|element|match element {
        PlaneElement::Polygon{vertices}=>1+vertices.len(),_=>1}).sum()
}
pub fn view_clone_visits(view:&ViewValue)->usize {
    match view {ViewValue::Scene(scene)=>scene_clone_visits(scene),ViewValue::ResultTable=>0,
        ViewValue::Timed {view,..}=>view_clone_visits(view)}
}

pub fn construct(name: &str, args: &[Value]) -> Result<Value, String> {
    construct_with_meter(name,args,None)
}
pub fn construct_with_meter(name: &str, args: &[Value],meter:Option<&mut crate::work_cost::WorkCostMeter>) -> Result<Value, String> {
    match (name,args) {
        ("표준장면.원", [center,Value::Fixed64(radius)]) => {
            if *radius <= Fixed64::ZERO { return Err("E_SCENE_RADIUS: 원 반지름은 양수여야 합니다".into()); }
            Ok(Value::SceneElement(PlaneElement::Circle {center:pair(center)?,radius:*radius}))
        }
        ("표준장면.글", [position,Value::String(content)]) =>
            Ok(Value::SceneElement(PlaneElement::Text {position:pair(position)?,content:content.clone()})),
        ("표준장면.다각형", [Value::List(vertices)]) => Ok(Value::SceneElement(polygon(vertices,&mut GeometryMath{meter})?)),
        ("표준장면.평면", [Value::List(elements),horizontal,vertical]) => {
            let horizontal = pair(horizontal)?; let vertical = pair(vertical)?;
            if horizontal[0] >= horizontal[1] || vertical[0] >= vertical[1] {
                return Err("E_SCENE_BOUNDS: 평면 범위는 유한 증가 구간이어야 합니다".into());
            }
            let elements = elements.iter().map(|element| match element {
                Value::SceneElement(element) => Ok(element.clone()),
                _ => Err("E_SCENE_ELEMENT_TYPE: 요소들은 장면요소값 차림이어야 합니다".to_string()),
            }).collect::<Result<Vec<_>,_>>()?;
            Ok(Value::Scene(PlaneScene {elements,horizontal,vertical}))
        }
        ("표준보기.장면", [Value::Scene(scene)]) => Ok(Value::View(ViewValue::Scene(scene.clone()))),
        _ => Err("E_SCENE_ARGUMENT: 장면 생성자의 받는자리 수와 씨를 확인하세요".into()),
    }
}

fn raw_pair(pair: &[Fixed64;2]) -> Json { json!([pair[0].raw_i64().to_string(),pair[1].raw_i64().to_string()]) }
pub fn project_element(element: &PlaneElement) -> Json {
    match element {
        PlaneElement::Circle {center,radius} => json!({"kind":"circle","center":raw_pair(center),"radius":radius.raw_i64().to_string()}),
        PlaneElement::Polygon {vertices} => json!({"kind":"polygon","vertices":vertices.iter().map(raw_pair).collect::<Vec<_>>()}),
        PlaneElement::Text {position,content} => json!({"kind":"text","position":raw_pair(position),"content":content}),
    }
}
pub fn project_view(view: &ViewValue) -> Json {
    match view {
        ViewValue::Timed {view,..} => project_view(view),
        ViewValue::ResultTable => json!({"kind":"result_table"}),
        ViewValue::Scene(scene) => json!({"kind":"scene","spatial_profile":"plane2",
            "coordinate_encoding":"fixed64_raw_decimal.v1", "horizontal":raw_pair(&scene.horizontal),
            "vertical":raw_pair(&scene.vertical), "elements":scene.elements.iter().map(project_element).collect::<Vec<_>>()}),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn pair_value(a:i64,b:i64)->Value {Value::List(vec![Value::Fixed64(Fixed64::from_i64(a)),Value::Fixed64(Fixed64::from_i64(b))])}
    #[test]
    fn circle_radius_and_authored_bounds_fail_closed() {
        assert!(construct("표준장면.원",&[pair_value(0,0),Value::Fixed64(Fixed64::ZERO)]).unwrap_err().contains("E_SCENE_RADIUS"));
        assert!(construct("표준장면.평면",&[Value::List(vec![]),pair_value(1,1),pair_value(0,2)]).unwrap_err().contains("E_SCENE_BOUNDS"));
        assert!(construct("표준보기.장면",&[Value::Pack(Default::default())]).is_err());
    }
    #[test]
    fn polygon_preserves_order_and_rejects_crossing_degeneracy_and_foldback() {
        let vertices=|points:&[(i64,i64)]| Value::List(points.iter().map(|(x,y)|pair_value(*x,*y)).collect());
        for points in [vec![(0,0),(2,0),(2,2),(0,2)],vec![(0,2),(2,2),(2,0),(0,0)],vec![(0,0),(1,0),(2,0),(2,2),(0,2)]] {
            let value=construct("표준장면.다각형",&[vertices(&points)]).unwrap();
            let Value::SceneElement(PlaneElement::Polygon{vertices})=&value else {panic!("typed polygon expected")};
            assert_eq!(vertices.len(),points.len());
            assert_eq!(vertices[0],[Fixed64::from_i64(points[0].0),Fixed64::from_i64(points[0].1)]);
        }
        for points in [vec![(0,0),(2,2),(0,2),(2,0)],vec![(0,0),(1,0),(2,0)],vec![(0,0),(2,0),(1,0),(2,2)],vec![(0,0),(2,0),(2,2),(0,0)]] {
            assert!(construct("표준장면.다각형",&[vertices(&points)]).unwrap_err().contains("E_SCENE_GEOMETRY"));
        }
        let raw=|x,y|Value::List(vec![Value::Fixed64(Fixed64::from_raw_i64(x)),Value::Fixed64(Fixed64::from_raw_i64(y))]);
        assert!(construct("표준장면.다각형",&[Value::List(vec![raw(i64::MIN,i64::MIN),raw(i64::MAX,i64::MIN),raw(i64::MAX,i64::MAX),raw(i64::MIN,i64::MAX)])]).is_ok());
    }
}
