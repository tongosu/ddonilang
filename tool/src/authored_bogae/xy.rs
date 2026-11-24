//! Source-local typed XY candidate. No public-call admission, registration
//! constructor, serialization, sample-ID issuance or final projection sealing.
pub(crate) mod binary_assets;
pub(crate) mod runtime_static_png;
pub(crate) use ddonirang_lang::runtime::static_png_assets;
use crate::numeric_constraint_plan::{table_selection_candidate, TableColumnKind, TableSelectionField};
use ddonirang_lang::{CanonProgram, Expr, ExprKind, TypeRef};
use serde_json::Value;

fn fail(message:&str)->String {format!("E_AUTHORED_BOGAE: {message}")}

pub(crate) struct ResolvedXyAxes {
    source:String,
    axes:[TableSelectionField;2],
}
fn numeric(kind:&TableColumnKind)->bool {
    match kind {
        TableColumnKind::Unit(_)=>true,
        TableColumnKind::Existing(TypeRef::Named(name))=>matches!(
            ddonirang_lang::stdlib::canonicalize_type_alias(name),
            "바른수"|"셈수"|"나눔수"|"큰바른수"|"곱수"),
        _=>false,
    }
}
/// Reuses the actual root observation resolver. Resolve each role separately:
/// table column deduplication must not forbid an explicit same-observation XY.
pub(crate) fn resolve_axes(program:&CanonProgram, x:&Expr, y:&Expr)->Result<ResolvedXyAxes,String> {
    let one=|expr:&Expr| {
        let mut singleton=expr.clone();
        singleton.kind=ExprKind::CollectionConstructor{elements:vec![expr.clone()]};
        table_selection_candidate(program,&singleton)
    };
    let x=one(x)?;let y=one(y)?;
    if x.source!=y.source {return Err(fail("두 축의 원문 소유가 다릅니다"));}
    let a=x.fields.into_iter().next().ok_or_else(||fail("가로 축 해소가 없습니다"))?;
    let b=y.fields.into_iter().next().ok_or_else(||fail("세로 축 해소가 없습니다"))?;
    if a.observation!=b.observation {return Err(fail("두 축은 같은 공개 관측 emission에 속해야 합니다; 자동 결합하지 않습니다"));}
    if !numeric(&a.kind)||!numeric(&b.kind) {return Err(fail("축에는 실제 수량 타입의 공개 관측이 필요합니다"));}
    Ok(ResolvedXyAxes{source:x.source,axes:[a,b]})
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub(crate) struct BranchContext {
    pub(crate) parent_run_id:String,
    pub(crate) madi:u64,
    pub(crate) state_hash:String,
}
/// Existing provenance coordinates scoped by the candidate's source/run/trace
/// and branch. display_event_index is not a new global or persistent identity.
pub(crate) struct JointSample {
    pub(crate) display_event_index:u64,
    pub(crate) committed_madi:u64,
    pub(crate) state_hash:String,
    pub(crate) cells:[Value;2],
}
pub(crate) struct JointXyCandidate {
    pub(crate) full_projection:Value,
    pub(crate) axis_columns:[Value;2],
    pub(crate) axis_types:[TableColumnKind;2],
    pub(crate) branch:Option<BranchContext>,
    pub(crate) samples:Vec<JointSample>,
}
/// The final call/view owner must additionally be bound by the projection
/// producer. This function does not relabel the full projection as selected,
/// reuse its hash for changed content, or convert coordinates to floats.
pub(crate) fn produce_joint_candidate(axes:&ResolvedXyAxes, trace:&Value,
    expected_run:&str, expected_branch:Option<&BranchContext>)->Result<JointXyCandidate,String> {
    if trace["source_identity"].as_str()!=Some(axes.source.as_str()) {
        return Err(fail("축과 관측 실행의 원문 판본이 다릅니다"));
    }
    let branch=match expected_branch {
        Some(branch) if trace["parent_run_id"].as_str()==Some(branch.parent_run_id.as_str())
            && trace["branch_madi"].as_u64()==Some(branch.madi)
            && trace["branch_state_hash"].as_str()==Some(branch.state_hash.as_str())=>Some(branch.clone()),
        Some(_)=>return Err(fail("축 표본의 branch 출처가 다릅니다")),
        None=>{
            if trace["parent_run_id"].as_str().is_some_and(|s|!s.is_empty())
                ||!trace["branch_madi"].is_null()
                ||trace["branch_state_hash"].as_str().is_some_and(|s|!s.is_empty()) {
                return Err(fail("branch 표본에는 명시된 부모 문맥이 필요합니다"));
            }
            None
        }
    };
    let full=crate::deterministic_debug::build_public_observation_table_projection(trace,expected_run)?;
    if full["presentation_state"]=="run_failed" {return Err(fail("실패 실행은 정상 XY 표본이 아닙니다"));}
    let matches=|column:&Value,axis:&TableSelectionField|column["identity"]["name"]==axis.name
        && column["identity"]["source_ref"]==axis.source_ref;
    let source_columns=trace["columns"].as_array().ok_or_else(||fail("관측 열 공급이 없습니다"))?;
    let column=|axis:&TableSelectionField|source_columns.iter().find(|c|matches(c,axis))
        .cloned().ok_or_else(||fail("해소된 축의 관측 열 공급이 없습니다"));
    let axis_columns=[column(&axes.axes[0])?,column(&axes.axes[1])?];
    for (column,axis) in axis_columns.iter().zip(&axes.axes) {
        if let TableColumnKind::Unit(expected)=axis.kind {
            let unit=column["unit"].as_str().ok_or_else(||fail("축 관측의 단위 공급이 없습니다"))?;
            // Reuse the core unit registry/dimension formatter. A consistent
            // trace label cannot override the source's checked UnitDim, and
            // admitting an equivalent symbol does not convert its values.
            if column["dtype"]!="unit" || !(unit==expected.format()
                || ddonirang_core::unit_spec_from_symbol(unit).is_some_and(|spec|spec.dim==expected)) {
                return Err(fail("축 관측의 단위 차원이 원문 타입과 다릅니다"));
            }
        }
    }
    let mut samples=Vec::new();
    // The shared validator makes one row from one actual emission sample.
    // Select both cells from that same row, never zip rows from separate tables.
    for table in full["tables"].as_array().ok_or_else(||fail("공동 표본 표가 없습니다"))? {
        let columns=table["columns"].as_array().ok_or_else(||fail("표 열이 없습니다"))?;
        let indices=[columns.iter().position(|c|matches(c,&axes.axes[0])),
            columns.iter().position(|c|matches(c,&axes.axes[1]))];
        if indices==[None,None] {continue;}
        let [Some(x),Some(y)]=indices else {return Err(fail("관측 emission에 두 축이 함께 공급되지 않았습니다"));};
        for row in table["rows"].as_array().ok_or_else(||fail("공동 표본 행이 없습니다"))? {
            let cells=row["cells"].as_array().ok_or_else(||fail("공동 표본 값이 없습니다"))?;
            let sample=JointSample{
                display_event_index:row["display_event_index"].as_u64().ok_or_else(||fail("실제 emission 순서가 없습니다"))?,
                committed_madi:row["committed_madi"].as_u64().ok_or_else(||fail("표본 commit이 없습니다"))?,
                state_hash:row["state_hash"].as_str().ok_or_else(||fail("표본 상태 출처가 없습니다"))?.into(),
                cells:[cells.get(x).cloned().ok_or_else(||fail("가로 표본이 없습니다"))?,
                    cells.get(y).cloned().ok_or_else(||fail("세로 표본이 없습니다"))?],
            };
            for (i,axis) in axes.axes.iter().enumerate() {
                if matches!(axis.kind,TableColumnKind::Unit(_)) && sample.cells[i]["status"]=="present" {
                    let suffix=sample.cells[i]["value_canon"].as_str().and_then(|value|value.rsplit_once('@'))
                        .map(|(_,unit)|unit);
                    if suffix!=axis_columns[i]["unit"].as_str() {
                        return Err(fail("축 정확값과 공급 열의 단위가 다릅니다"));
                    }
                }
            }
            samples.push(sample);
        }
    }
    // Table grouping is not sample order. Restore only the supplied global
    // emission order; no sorting by X/time, merging equal X, interpolation/join.
    samples.sort_by_key(|sample|sample.display_event_index);
    if samples.windows(2).any(|p|p[0].display_event_index==p[1].display_event_index) {
        return Err(fail("공동 emission 순서가 중복됐습니다"));
    }
    Ok(JointXyCandidate{full_projection:full,axis_columns,
        axis_types:[axes.axes[0].kind.clone(),axes.axes[1].kind.clone()],branch,samples})
}


impl ResolvedXyAxes {
    pub(crate) fn into_typed(self,registration:&ddonirang_lang::stdlib::xy_registration_r1::CheckedXyDescriptor)
        ->Result<ddonirang_lang::runtime::XySelection,String> {
        use ddonirang_lang::runtime::{ResultTableColumn,ResultTableColumnType,XySelection};
        let [x,y]=self.axes;
        let column=|a:TableSelectionField|ResultTableColumn {declaration:a.declaration,observation:a.observation,
            name:a.name,source_ref:a.source_ref,kind:match a.kind {TableColumnKind::Existing(t)=>ResultTableColumnType::Existing(t),TableColumnKind::Unit(u)=>ResultTableColumnType::Unit(u)}};
        XySelection::from_resolved_source(registration,self.source,[column(x),column(y)])
    }
}
fn transported_axes(selection:&ddonirang_lang::runtime::XySelection)->ResolvedXyAxes {
    use ddonirang_lang::runtime::ResultTableColumnType;
    let field=|c:&ddonirang_lang::runtime::ResultTableColumn|TableSelectionField {declaration:c.declaration,
        observation:c.observation,name:c.name.clone(),source_ref:c.source_ref.clone(),
        kind:match &c.kind {ResultTableColumnType::Existing(t)=>TableColumnKind::Existing(t.clone()),ResultTableColumnType::Unit(u)=>TableColumnKind::Unit(*u)}};
    ResolvedXyAxes {source:selection.source().into(),axes:[field(&selection.axes()[0]),field(&selection.axes()[1])]}
}
/// Source spelling only selects a checked entry of the reviewed immutable pack.
/// The same actual root owner/typed observation resolver supplies both axes.
pub(crate) fn resolve_call(program:&CanonProgram,call:&Expr,shadowed:bool)
    ->Result<Option<ddonirang_lang::runtime::XySelection>,String> {
    use ddonirang_lang::stdlib::xy_registration_r1::{XyMode,AxisRole,checked_descriptor};
    let ExprKind::Call{func,args}=&call.kind else{return Ok(None)};
    let mode=match func.as_str(){"표준보기.흩어진점"=>XyMode::Scatter,"표준보기.좌표선"=>XyMode::Polyline,_=>return Ok(None)};
    let masks=|name:&str|name==func||name=="표준보기";
    if shadowed||program.items.iter().any(|ddonirang_lang::TopLevelItem::SeedDef(seed)|seed.callable_surface_names().iter().any(|n|masks(n)))
        ||program.root_stmts.iter().any(|s|match s {
            ddonirang_lang::Stmt::Definition{name,..}=>masks(name),
            ddonirang_lang::Stmt::DeclBlock{items,..}=>items.iter().any(|i|masks(&i.name)),
            _=>false,
        }) {return Ok(None)}
    let descriptor=checked_descriptor(mode)?;
    let roles=args.iter().map(|arg|match arg.resolved_pin.as_deref(){Some("가로")=>Ok(AxisRole::X),Some("세로")=>Ok(AxisRole::Y),_=>Err(fail("그래프에는 명시 가로·세로 축만 각각 한 번 필요합니다"))}).collect::<Result<Vec<_>,_>>()?;
    let [x,y]=descriptor.argument_slots(&roles)?;
    resolve_axes(program,&args[x].expr,&args[y].expr)?.into_typed(&descriptor).map(Some)
}

/// Private selected tables from one validated joint sample stream. No new wire
/// fields: preserve the existing v2 columns/rows, canonical cells and provenance.
pub(crate) fn selected_tables(candidate:&JointXyCandidate)->Result<Vec<Value>,String> {
    let same=|a:&Value,b:&Value|a["identity"]==b["identity"];
    let mut result=Vec::new();
    for table in candidate.full_projection["tables"].as_array().ok_or_else(||fail("표 공급 없음"))? {
        let columns=table["columns"].as_array().ok_or_else(||fail("열 공급 없음"))?;
        let x=columns.iter().position(|c|same(c,&candidate.axis_columns[0]));
        let y=columns.iter().position(|c|same(c,&candidate.axis_columns[1]));
        if x.is_none()&&y.is_none(){continue;}
        let (Some(x),Some(y))=(x,y)else{return Err(fail("두 축의 공동 표본 표가 없습니다"))};
        let indices=if x==y {vec![x]}else{vec![x,y]};
        let mut selected=table.clone();selected["columns"]=Value::Array(indices.iter().map(|i|columns[*i].clone()).collect());
        for row in selected["rows"].as_array_mut().ok_or_else(||fail("표본 행 없음"))? {
            let cells=row["cells"].as_array().ok_or_else(||fail("표본 값 없음"))?;
            row["cells"]=Value::Array(indices.iter().map(|i|cells[*i].clone()).collect());
        }
        result.push(selected);
    }
    Ok(result)
}
fn plot_number(cell:&Value,kind:&TableColumnKind)->Result<Option<f64>,String> {
    if cell["status"]!="present" {return Ok(None)}
    if !numeric(kind) {return Err(fail("축의 수량 타입이 다릅니다"))}
    let text=cell["value_canon"].as_str().ok_or_else(||fail("정확한 축 값이 없습니다"))?;
    // Typed quantities were admitted above. This conversion is only screen
    // projection; canonical cells/units are retained verbatim in the v2 table.
    let text=match kind {
        TableColumnKind::Unit(_)=>text.split_once('@')
            .ok_or_else(||fail("수량의 단위 canonical이 다릅니다"))?.0,
        TableColumnKind::Existing(TypeRef::Named(name))=>{
            let prefix=match ddonirang_lang::stdlib::canonicalize_type_alias(name) {
                "큰바른수"=>Some("큰바른수{"),
                "나눔수"=>Some("나눔수{"),
                _=>None,
            };
            match prefix {
                Some(prefix)=>text.strip_prefix(prefix).and_then(|t|t.strip_suffix('}'))
                    .ok_or_else(||fail("축 타입과 수량 canonical이 다릅니다"))?,
                None=>text,
            }
        },
        _=>text,
    };
    let parse=|s:&str|s.trim().parse::<f64>().map_err(|_|fail("축 값의 화면 투영 범위를 지원하지 않습니다"));
    let value=if let Some((a,b))=text.split_once('/') {let b=parse(b)?;if b==0.0{return Err(fail("축 분모가 영입니다"))}parse(a)?/b}else{parse(text)?};
    if !value.is_finite(){return Err(fail("축 값의 화면 투영 범위를 지원하지 않습니다"))}
    Ok(Some(value))
}
fn axis_screen_fraction(value:f64,min:f64,max:f64)->f64 {
    if min==max {return 0.5;}
    let span=max-min;
    if span.is_finite() {
        (value-min)/span
    } else {
        // Opposite finite extremes can have an infinite difference. Scale
        // before subtracting so valid screen points do not become NaN/zero.
        let scale=min.abs().max(max.abs());
        (value/scale-min/scale)/(max/scale-min/scale)
    }
}
// Screen annotation only, never a canonical quantity or a unit conversion.
// Bound the text width for finite extremes without hiding their exponent.
fn screen_number(value:f64)->String {
    if value==0.0 {return "0".into();}
    if value.abs()>=1e6 || value.abs()<0.001 {
        let text=format!("{value:.2e}");
        let (mantissa,exponent)=text.split_once('e').expect("scientific formatter");
        format!("{}e{exponent}",mantissa.trim_end_matches('0').trim_end_matches('.'))
    } else {
        format!("{value:.3}").trim_end_matches('0').trim_end_matches('.').into()
    }
}
/// Final view is existing scene_plane2.v1. Canonical data never comes back from
/// screen coordinates: the separately sealed existing table owns exact values.
pub(crate) fn project_joint(selection:&ddonirang_lang::runtime::XySelection,trace:&Value,run:&str)
    ->Result<(ddonirang_lang::runtime::PlaneScene,JointXyCandidate),String> {
    use ddonirang_core::Fixed64;
    use ddonirang_lang::runtime::{PlaneScene,PlaneElement};
    use ddonirang_lang::stdlib::xy_registration_r1::XyMode;
    let candidate=produce_joint_candidate(&transported_axes(selection),trace,run,None)?;
    let mut points=Vec::new();
    for sample in &candidate.samples {
        let x=plot_number(&sample.cells[0],&candidate.axis_types[0])?;
        let y=plot_number(&sample.cells[1],&candidate.axis_types[1])?;
        points.push(match (x,y){(Some(x),Some(y))=>Some([x,y]),_=>None});
    }
    let mut min=[f64::INFINITY;2];let mut max=[f64::NEG_INFINITY;2];
    for p in points.iter().flatten(){for i in 0..2{min[i]=min[i].min(p[i]);max[i]=max[i].max(p[i]);}}
    let raw=|n:f64|Fixed64::from_raw_i64((n*Fixed64::ONE_RAW as f64).round() as i64);
    let normalized=|p:[f64;2]|-> [f64;2] {std::array::from_fn(|i|axis_screen_fraction(p[i],min[i],max[i]))};
    let mut elements=Vec::new();let mut previous:Option<[f64;2]>=None;
    for point in points {
        let Some(point)=point else{previous=None;continue};let p=normalized(point);
        if selection.mode()==XyMode::Polyline {if let Some(a)=previous {
            let dx=p[0]-a[0];let dy=p[1]-a[1];let length=dx.hypot(dy);
            if length>0.0 {let o=[-dy/length*0.003,dx/length*0.003];
                let vertices=[[a[0]+o[0],a[1]+o[1]],[p[0]+o[0],p[1]+o[1]],[p[0]-o[0],p[1]-o[1]],[a[0]-o[0],a[1]-o[1]]].map(|v|[raw(v[0]),raw(v[1])]).to_vec();elements.push(PlaneElement::Polygon{vertices});}
        }}
        elements.push(PlaneElement::Circle{center:[raw(p[0]),raw(p[1])],radius:raw(0.012)});previous=Some(p);
    }
    let label=|i:usize| {
        let axis=if i==0{"가로"}else{"세로"};
        let name=candidate.axis_columns[i]["label"].as_str().unwrap_or(&selection.axes()[i].name);
        let unit=candidate.axis_columns[i]["unit"].as_str().filter(|unit|!unit.is_empty()).unwrap_or("단위 미표기");
        let full=format!("{axis}: {name} [{unit}]");
        // This is a conservative display budget, not a font metric or a
        // semantic name limit. Never truncate the supplied name or unit:
        // the sealed table keeps both in full, in selected-axis order.
        // Repeated X/Y has one table column, so first and last still agree.
        let columns=full.chars().map(|c|match c {
            'A'..='Z'|'m'|'w'|'@'|'%'=>2,
            c if c.is_ascii()=>1,
            _=>2,
        }).sum::<usize>();
        if columns<=24 {full}
        else {format!("{axis}: 표의 {} 열",if i==0{"첫"}else{"끝"})}
    };
    let range=|i:usize| {
        let axis=if i==0{"가로"}else{"세로"};
        if !min[i].is_finite() {return format!("{axis}: 현재 표본 없음");}
        // Mark approximation even for integer-looking screen values. In
        // particular, f64 rounding must not claim equality of exact cells.
        format!("{axis} ≈ {} … {}",screen_number(min[i]),screen_number(max[i]))
    };
    elements.push(PlaneElement::Text{position:[raw(0.02),raw(-0.12)],content:label(0)});
    elements.push(PlaneElement::Text{position:[raw(0.02),raw(-0.25)],content:range(0)});
    elements.push(PlaneElement::Text{position:[raw(0.02),raw(1.04)],content:label(1)});
    elements.push(PlaneElement::Text{position:[raw(0.02),raw(1.17)],content:range(1)});
    elements.push(PlaneElement::Text{position:[raw(0.02),raw(1.31)],content:"화면 근사 / 정확값은 표".into()});
    Ok((PlaneScene{elements,horizontal:[raw(-0.2),raw(1.2)],vertical:[raw(-0.36),raw(1.47)]},candidate))
}

#[cfg(test)]
#[path = "xy/sparse_projection_tests.rs"]
mod sparse_projection_tests;

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) const SOURCE:&str="(경과: 1@s, 위치: 2@m, 완료: 참) 내보임.";
    #[test]
    fn screen_range_keeps_finite_extremes_ordered() {
        for (min,max) in [(-1e308,1e308),(-f64::MAX,f64::MAX)] {
            assert_eq!(axis_screen_fraction(min,min,max),0.0);
            assert_eq!(axis_screen_fraction(0.0,min,max),0.5);
            assert_eq!(axis_screen_fraction(max,min,max),1.0);
        }
        assert_eq!(axis_screen_fraction(-2.0,-2.0,6.0),0.0);
        assert_eq!(axis_screen_fraction(2.0,-2.0,6.0),0.5);
        assert_eq!(axis_screen_fraction(6.0,-2.0,6.0),1.0);
        assert_eq!(axis_screen_fraction(f64::MAX,f64::MAX,f64::MAX),0.5);
    }
    pub(super) fn axes(source:&str,x:&str,y:&str)->Result<ResolvedXyAxes,String> {
        let mut program=ddonirang_lang::parse_v25(source,"xy.ddn").map_err(|e|format!("{e:?}"))?;
        ddonirang_lang::canonicalize(&mut program).map_err(|e|format!("{e:?}"))?;
        // Internal resolver exercise, not public graph dispatch or source lowering.
        let parsed=ddonirang_lang::parse_v25(&format!("가로축 := {x}. 세로축 := {y}."),"axes.ddn").unwrap();
        let ddonirang_lang::Stmt::Definition{value:a,..}=&parsed.root_stmts[0] else {panic!("axis")};
        let ddonirang_lang::Stmt::Definition{value:b,..}=&parsed.root_stmts[1] else {panic!("axis")};
        resolve_axes(&program,a,b)
    }
    pub(super) fn trace()->Value {
        crate::runtime_surface::run_summary_from_supported_source_profile(SOURCE,"xy.ddn",Some(1),Some("v1-core-v25"))
            .unwrap()["public_observation_trace"].clone()
    }
    #[test]
    fn actual_trace_units_swap_same_axis_and_source_are_preserved() {
        let trace=trace();let before=trace.clone();let run=trace["run_id"].as_str().unwrap();
        let axes=axes(SOURCE,"관찰.경과","관찰.위치").unwrap();
        let joint=produce_joint_candidate(&axes,&trace,run,None).unwrap();
        assert_eq!(joint.samples.len(),1);assert_eq!(joint.axis_columns[0]["unit"],"s");assert_eq!(joint.axis_columns[1]["unit"],"m");
        assert!(matches!(joint.axis_types[0],TableColumnKind::Unit(_)));assert!(joint.branch.is_none());
        assert_eq!(joint.samples[0].cells[0]["value_canon"],"1@s");assert_eq!(joint.samples[0].cells[1]["value_canon"],"2@m");
        assert_eq!(joint.samples[0].committed_madi,0);assert_eq!(joint.samples[0].state_hash,trace["samples"][0]["state_hash"].as_str().unwrap());
        assert_eq!(joint.full_projection["source_trace_hash"],crate::deterministic_debug::public_observation_trace_hash(&trace));
        let swapped=super::tests::axes(SOURCE,"관찰.위치","관찰.경과").unwrap();
        let swap=produce_joint_candidate(&swapped,&trace,run,None).unwrap();
        assert_eq!(swap.samples[0].cells,[joint.samples[0].cells[1].clone(),joint.samples[0].cells[0].clone()]);
        let same=super::tests::axes(SOURCE,"관찰.위치","관찰.위치").unwrap();
        let same=produce_joint_candidate(&same,&trace,run,None).unwrap();assert_eq!(same.samples[0].cells[0],same.samples[0].cells[1]);
        assert_eq!(trace,before);
    }
    #[test]
    fn refuse_non_numeric_other_emissions_and_unresolved_axes() {
        assert!(axes(SOURCE,"관찰.완료","관찰.위치").is_err());
        assert!(axes(SOURCE,"관찰.미공급","관찰.위치").is_err());
        assert!(axes("(가: 1) 내보임. (나: 2) 내보임.","관찰.가","관찰.나").is_err());
    }
    #[test]
    fn refuse_wrong_run_source_and_unbound_branch() {
        let trace=trace();let axes=axes(SOURCE,"관찰.경과","관찰.위치").unwrap();
        assert!(produce_joint_candidate(&axes,&trace,"wrong",None).is_err());
        let mut other=trace.clone();other["source_identity"]=serde_json::json!("sha256:other");
        assert!(produce_joint_candidate(&axes,&other,trace["run_id"].as_str().unwrap(),None).is_err());
        let branch=BranchContext{parent_run_id:"parent".into(),madi:0,state_hash:"state".into()};
        assert!(produce_joint_candidate(&axes,&trace,trace["run_id"].as_str().unwrap(),Some(&branch)).is_err());
        let mut branch_trace=trace.clone();branch_trace["parent_run_id"]=serde_json::json!("parent");branch_trace["branch_madi"]=serde_json::json!(0);branch_trace["branch_state_hash"]=serde_json::json!("state");
        assert!(produce_joint_candidate(&axes,&branch_trace,trace["run_id"].as_str().unwrap(),None).is_err());
        assert_eq!(produce_joint_candidate(&axes,&branch_trace,trace["run_id"].as_str().unwrap(),Some(&branch)).unwrap().branch,Some(branch));
    }
    #[test]
    fn sparse_status_is_preserved_and_separate_emissions_are_not_zipped() {
        let mut trace=trace();let run=trace["run_id"].as_str().unwrap().to_owned();let axes=axes(SOURCE,"관찰.경과","관찰.위치").unwrap();
        // Deliberate contract mutation, not a real sparse-source execution.
        let cell=trace["samples"][0]["cells"][1].as_object_mut().unwrap();
        cell.remove("value_canon");cell.remove("value_text");cell.insert("status".into(),serde_json::json!("missing"));cell.insert("reason".into(),serde_json::json!("#원천없음"));
        let joint=produce_joint_candidate(&axes,&trace,&run,None).unwrap();
        assert_eq!(joint.samples[0].cells[1]["status"],"missing");assert_eq!(joint.samples[0].cells[1]["reason"],"#원천없음");
        assert!(joint.samples[0].cells[1].get("value_canon").is_none());
        let mut split=trace.clone();let cells=split["samples"][0]["cells"].as_array().unwrap().clone();
        let mut first=split["samples"][0].clone();first["cells"]=serde_json::json!([cells[0]]);
        let mut second=first.clone();second["cells"]=serde_json::json!([cells[1]]);
        split["samples"]=serde_json::json!([first,second]);
        assert!(produce_joint_candidate(&axes,&split,&run,None).is_err());
    }

    #[test]
    fn supplied_sparse_status_breaks_sealed_polyline_without_fabricating_values() {
        use ddonirang_lang::stdlib::xy_registration_r1::{checked_descriptor,XyMode};
        // Real dense source/trace and registered typed axes. Sparse states below
        // are deliberate producer-contract mutations, not public sparse syntax.
        let source="(시작)할때 { 계수 <- 0. }. (매마디)마다 { 계수 <- 계수 + 1. }. (경과:계수 * 1@s,위치:계수 * 2@m) 내보임.";
        let original=crate::runtime_surface::run_summary_from_supported_source_profile(source,"xy.ddn",Some(5),Some("v1-core-v25"))
            .unwrap()["public_observation_trace"].clone();
        let selection=axes(source,"관찰.경과","관찰.위치").unwrap()
            .into_typed(&checked_descriptor(XyMode::Polyline).unwrap()).unwrap();
        let run=original["run_id"].as_str().unwrap();
        for status in ["gap","missing","pruned"] {
            let mut trace=original.clone();
            let cell=trace["samples"][2]["cells"][1].as_object_mut().unwrap();
            cell.remove("value_canon");cell.remove("value_text");
            cell.insert("status".into(),serde_json::json!(status));
            if status=="missing" {cell.insert("reason".into(),serde_json::json!("#원천없음"));}
            let before=trace.clone();
            let (scene,table)=crate::deterministic_debug::build_xy_public_observation_projection(&trace,run,&selection).unwrap();
            assert_eq!(trace,before);
            let circles=scene.elements.iter().filter(|e|matches!(e,ddonirang_lang::runtime::PlaneElement::Circle{..})).count();
            let segments:Vec<_>=scene.elements.iter().filter_map(|e|match e {
                ddonirang_lang::runtime::PlaneElement::Polygon{vertices}=>Some(vertices),_=>None,
            }).collect();
            assert_eq!(circles,4);assert_eq!(segments.len(),2);
            for segment in segments {
                let xs:Vec<_>=segment.iter().map(|p|p[0].raw_i64()).collect();
                let middle=ddonirang_core::Fixed64::ONE_RAW/2;
                assert!(xs.iter().all(|x|*x<middle)||xs.iter().all(|x|*x>middle),"line bridged supplied sparse sample");
            }
            let rows=table["tables"][0]["rows"].as_array().unwrap();
            assert_eq!(rows.len(),5);
            // Trace cells carry identity; v2 tables put it on the column.
            let mut expected=trace["samples"][2]["cells"][1].as_object().unwrap().clone();
            let identity=expected.remove("identity").unwrap();
            assert_eq!(table["tables"][0]["columns"][1]["identity"],identity);
            assert_eq!(rows[2]["cells"][1],Value::Object(expected));
            assert!(rows[2]["cells"][1].get("value_canon").is_none());
            assert!(table["projection_hash"].as_str().is_some_and(|hash|!hash.is_empty()));
        }
    }

    #[test]
    fn supplied_branch_context_cannot_be_sealed_as_an_unbound_xy_view() {
        use ddonirang_lang::stdlib::xy_registration_r1::{checked_descriptor,XyMode};
        let mut trace=trace();
        let selection=axes(SOURCE,"관찰.경과","관찰.위치").unwrap()
            .into_typed(&checked_descriptor(XyMode::Polyline).unwrap()).unwrap();
        // This fixture checks the final projection gate, not a real branch run.
        trace["parent_run_id"]=serde_json::json!("UI_TEST_PARENT_NOT_A_REAL_BRANCH");
        trace["branch_madi"]=serde_json::json!(0);
        trace["branch_state_hash"]=trace["samples"][0]["state_hash"].clone();
        let before=trace.clone();let run=trace["run_id"].as_str().unwrap();
        let error=crate::deterministic_debug::build_xy_public_observation_projection(&trace,run,&selection).err().unwrap();
        assert!(error.contains("branch 표본에는 명시된 부모 문맥이 필요합니다"),"{error}");
        assert_eq!(trace,before);
    }
}
