//! 조건별 계산의 닫힌 정확 구간·정의가능성 검사와 컴파일 로컬 소비.
//! 공통 numeric owner의 같은 부모/핀/식·선택 연산만 소비한다. formal Proof,
//! 공개 identity/profile/wire 발급이나 일반 수학 함수 참여로 확장하지 않는다.
use ddonirang_lang::{CanonProgram, Expr, ExprKind, ParseError, PiecewiseCondition, Span, TopLevelItem};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{One, Zero};

type Q = BigRational;
type Set = Vec<Interval>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Edge { value: Q, closed: bool }
#[derive(Clone, Debug, PartialEq, Eq)]
struct Interval { low: Option<Edge>, high: Option<Edge> }

fn failure(span: Span, message: &str) -> ParseError {
    ParseError { span, message: format!("조건별 구간 검사: {message}") }
}

struct Scope<'a> { pin: &'a str, span: Span, remaining: usize }
impl Scope<'_> {
    fn meet(&mut self, a:&Set, b:&Set)->Result<Set,ParseError> {
        let work=a.len().saturating_mul(b.len());
        if a.len()>128 || b.len()>128 || work>self.remaining {
            return Err(failure(self.span,"구간 조합 한도 초과로 미확인입니다"));
        }
        self.remaining-=work;
        let result=intersection(a,b);
        if result.len()>128 {return Err(failure(self.span,"구간 수 한도 초과로 미확인입니다"));}
        Ok(result)
    }
    fn step(&mut self, depth: usize) -> Result<(), ParseError> {
        if depth > 64 || self.remaining == 0 {
            return Err(failure(self.span, "검사 한도 초과로 미확인입니다"));
        }
        self.remaining -= 1;
        Ok(())
    }
    fn bounded(&self, q: Q) -> Result<Q, ParseError> {
        if q.numer().bits() > 4096 || q.denom().bits() > 4096 {
            Err(failure(self.span, "정확수 표현 한도 초과로 미확인입니다"))
        } else { Ok(q) }
    }
    // 닫힌 구간 predicate의 affine 조각만 해석한다. 호출·다른 이름·capture는 거부.
    fn linear(&mut self, expr: &Expr, depth: usize) -> Result<(Q,Q), ParseError> {
        self.step(depth)?;
        if let Some(operand)=expr.source_negated_operand() {
            let (a,b)=self.linear(operand,depth+1)?;return Ok((-a,-b));
        }
        match &expr.kind {
            ExprKind::Var(name) if name == self.pin => Ok((Q::one(), Q::zero())),
            ExprKind::Literal(_) => {
                let literal = expr.numeric_literal_source.as_ref()
                    .ok_or_else(|| failure(expr.span, "원문 수 리터럴 공급이 없습니다"))?;
                let raw = literal.raw();
                if raw.len() > 1230 { return Err(failure(expr.span, "정확수 표현 한도 초과로 미확인입니다")); }
                let (digits, scale) = if let Some((a,b)) = raw.split_once('.') {
                    (format!("{a}{b}"), b.len())
                } else { (raw.to_string(), 0) };
                let n = digits.parse::<BigInt>().map_err(|_| failure(expr.span, "미지원 수 리터럴입니다"))?;
                Ok((Q::zero(), self.bounded(Q::new(n, BigInt::from(10).pow(scale as u32)))?))
            }
            ExprKind::Infix {left,op,right} => {
                let (a,b)=self.linear(left,depth+1)?;
                let (c,d)=self.linear(right,depth+1)?;
                let result = match op.as_str() {
                    "+" => (a+c,b+d), "-" => (a-c,b-d),
                    "*" if a.is_zero() => (&b*c,b*d),
                    "*" if c.is_zero() => (&d*a,d*b),
                    "/" if c.is_zero() && !d.is_zero() => (a/&d,b/d),
                    _ => return Err(failure(expr.span, "구간 끝점의 비선형·영분모·미지원 연산은 미확인입니다")),
                };
                Ok((self.bounded(result.0)?,self.bounded(result.1)?))
            }
            _ => Err(failure(expr.span, "다른 입력·호출·capture 또는 미지원 구간 식입니다")),
        }
    }
    // 원래 AST를 약분하거나 계산하지 않고 모든 하위 분모 의무를 수집/검사한다.
    // 첫 닫힌 규칙은 affine 분모의 정확 영점 배제다. 비선형은 미확인으로 거부.
    fn defined_value(&mut self, expr:&Expr, region:&Set, depth:usize)->Result<(),ParseError> {
        self.step(depth)?;
        if let Some(operand)=expr.source_negated_operand() {return self.defined_value(operand,region,depth+1);}
        match &expr.kind {
            ExprKind::Var(name) if name==self.pin=>Ok(()),
            ExprKind::Literal(_) => self.linear(expr,depth+1).map(|_|()),
            ExprKind::Infix{left,op,right} if matches!(op.as_str(),"+"|"-"|"*"|"/")=>{
                self.defined_value(left,region,depth+1)?;
                self.defined_value(right,region,depth+1)?;
                if op=="/" && !region.is_empty() {
                    let (a,b)=self.linear(right,depth+1)?;
                    let zero=if a.is_zero(){b.is_zero()}else{contains(region,&self.bounded(-b/a)?)};
                    if zero{return Err(failure(right.span,"활성 구간에서 원래 분모가 0이 될 수 있습니다"));}
                }
                Ok(())
            }
            _=>Err(failure(expr.span,"다른 입력·호출·효과·capture 또는 미지원 값식은 미확인입니다")),
        }
    }
    fn condition(&mut self, condition: &PiecewiseCondition, depth: usize) -> Result<Set,ParseError> {
        self.step(depth)?;
        match condition {
            PiecewiseCondition::ExplicitEmpty(_) => Ok(vec![]),
            PiecewiseCondition::Relations {terms,relations} => {
                if terms.len()!=relations.len()+1 || relations.is_empty() {
                    return Err(failure(self.span, "관계와 항의 실제 공급이 맞지 않습니다"));
                }
                let mut set=universe();
                for (pair,op) in terms.windows(2).zip(relations) {
                    let (a,b)=self.linear(&pair[0],depth+1)?;
                    let (c,d)=self.linear(&pair[1],depth+1)?;
                    let coefficient=self.bounded(a-c)?;
                    let constant=self.bounded(b-d)?;
                    let atom=relation(coefficient,constant,op,self.span)?;
                    set=self.meet(&set,&atom)?;
                }
                Ok(set)
            }
            PiecewiseCondition::And(a,b) => {
                let a=self.condition(a,depth+1)?; let b=self.condition(b,depth+1)?;
                self.meet(&a,&b)
            }
            PiecewiseCondition::Or(a,b) => {
                let mut a=self.condition(a,depth+1)?; a.extend(self.condition(b,depth+1)?);
                let result=normalize(a);
                if result.len()>128 {return Err(failure(self.span,"구간 수 한도 초과로 미확인입니다"));}
                Ok(result)
            }
            PiecewiseCondition::Not(a) => {
                let result=complement(&self.condition(a,depth+1)?);
                if result.len()>128 {return Err(failure(self.span,"구간 수 한도 초과로 미확인입니다"));}
                Ok(result)
            }
        }
    }
}

fn universe() -> Set { vec![Interval {low:None,high:None}] }
fn contains(set:&Set,value:&Q)->bool {
    set.iter().any(|i| i.low.as_ref().is_none_or(|b|value>&b.value || (value==&b.value&&b.closed))
        && i.high.as_ref().is_none_or(|b|value<&b.value || (value==&b.value&&b.closed)))
}
fn nonempty(i:&Interval)->bool {
    match (&i.low,&i.high) {
        (Some(a),Some(b)) => a.value<b.value || (a.value==b.value && a.closed && b.closed),
        _ => true,
    }
}
fn intersection(a:&Set,b:&Set)->Set {
    let mut out=vec![];
    for a in a { for b in b {
        let low=match (&a.low,&b.low) {
            (None,b)|(b,None)=>b.clone(),
            (Some(a),Some(b))=>Some(if a.value==b.value {Edge{value:a.value.clone(),closed:a.closed&&b.closed}}
                else if a.value>b.value {a.clone()}else{b.clone()}),
        };
        let high=match (&a.high,&b.high) {
            (None,b)|(b,None)=>b.clone(),
            (Some(a),Some(b))=>Some(if a.value==b.value {Edge{value:a.value.clone(),closed:a.closed&&b.closed}}
                else if a.value<b.value {a.clone()}else{b.clone()}),
        };
        let i=Interval{low,high};if nonempty(&i){out.push(i);}
    }}
    normalize(out)
}
fn normalize(mut set:Set)->Set {
    set.retain(nonempty);
    set.sort_by(|a,b| match (&a.low,&b.low) {
        (None,None)=>std::cmp::Ordering::Equal,(None,_)=>std::cmp::Ordering::Less,(_,None)=>std::cmp::Ordering::Greater,
        (Some(a),Some(b))=>a.value.cmp(&b.value).then(b.closed.cmp(&a.closed)),
    });
    let mut out:Set=vec![];
    for next in set {
        if let Some(last)=out.last_mut() {
            let joins=match (&last.high,&next.low) {
                (None,_)|(_,None)=>true,
                (Some(a),Some(b))=>a.value>b.value || (a.value==b.value&&(a.closed||b.closed)),
            };
            if joins {
                last.high=match (&last.high,&next.high) {
                    (None,_)|(_,None)=>None,
                    (Some(a),Some(b))=>Some(if a.value==b.value {Edge{value:a.value.clone(),closed:a.closed||b.closed}}
                        else if a.value>b.value {a.clone()}else{b.clone()}),
                };
                continue;
            }
        }
        out.push(next);
    }
    out
}
fn complement(set:&Set)->Set {
    let mut out=vec![];let mut low=None;
    for interval in normalize(set.clone()) {
        if let Some(start)=interval.low {
            let gap=Interval{low:low.clone(),high:Some(Edge{value:start.value,closed:!start.closed})};
            if nonempty(&gap){out.push(gap);}
        }
        match interval.high {
            None=>return out,
            Some(end)=>low=Some(Edge{value:end.value,closed:!end.closed}),
        }
    }
    out.push(Interval{low,high:None});out
}
fn relation(a:Q,b:Q,op:&str,span:Span)->Result<Set,ParseError> {
    if a.is_zero() {
        let yes=match op {"<"=>b<Q::zero(),"<="=>b<=Q::zero(),">"=>b>Q::zero(),">="=>b>=Q::zero(),
            "=:="=>b.is_zero(),_=>return Err(failure(span,"미지원 관계입니다"))};
        return Ok(if yes {universe()}else{vec![]});
    }
    let op=if a<Q::zero(){match op{"<"=>">","<="=>">=",">"=>"<",">="=>"<=",other=>other}}else{op};
    let value=-b/a;
    if value.numer().bits()>4096 || value.denom().bits()>4096 {
        return Err(failure(span,"정확 끝점 표현 한도 초과로 미확인입니다"));
    }
    let edge=Edge{value,closed:matches!(op,"<="|">="|"=:=")};
    Ok(match op {
        "<"|"<="=>vec![Interval{low:None,high:Some(edge)}],
        ">"|">="=>vec![Interval{low:Some(edge),high:None}],
        "=:="=>vec![Interval{low:Some(edge.clone()),high:Some(edge)}],
        _=>return Err(failure(span,"미지원 관계입니다")),
    })
}

/// 공통 원문 owner의 실제 부모/핀을 선택하여 분할만 재계산한다.
/// 성공은 계산·G-R·Φ·정의가능성 또는 formal Proof admission이 아니다.
pub(super) fn verify_source_partitions(program:&CanonProgram)->Result<(),ParseError> {
    let mut definitions:Vec<_>=program.origin.piecewise_definitions.iter().collect();
    definitions.sort_by_key(|(_,body)|body.span.start);
    for (id,body) in definitions {
        let mut parents=program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).filter(|seed|seed.id==*id);
        let seed=parents.next().ok_or_else(||failure(body.span,"실제 부모 정의가 없습니다"))?;
        if parents.next().is_some() || seed.params.len()!=1 || !seed.type_params.is_empty()
            || !seed.modifiers.is_empty() || seed.params[0].optional || seed.params[0].default_value.is_some() {
            return Err(failure(body.span,"유일한 필수 단항 부모/핀 공급이 아닙니다"));
        }
        let mut scope=Scope{pin:&seed.params[0].pin_name,span:body.span,remaining:4096};
        let domain=scope.condition(&body.domain,0)?;
        if matches!(body.domain,PiecewiseCondition::ExplicitEmpty(_)) {
            if !body.rows.is_empty(){return Err(failure(body.span,"명시 빈 정의역에는 행을 둘 수 없습니다"));}
            continue;
        }
        if domain.is_empty(){return Err(failure(body.span,"모순 조건을 명시 빈 함수로 대신하지 않습니다"));}
        if domain.iter().any(|i|i.low.is_none()||i.high.is_none()) {
            return Err(failure(body.span,"첫 지원 범위는 유한한 정확 유리수 정의역입니다"));
        }
        if body.rows.len()>128 {return Err(failure(body.span,"행 검사 한도 초과로 미확인입니다"));}
        let mut covered=vec![];
        for row in &body.rows {
            let region=scope.condition(&row.condition,0)?;
            if !scope.meet(&region,&complement(&domain))?.is_empty(){return Err(failure(row.value.span,"정의역 밖 조각을 자동 절단하지 않습니다"));}
            if !scope.meet(&region,&covered)?.is_empty(){return Err(failure(row.value.span,"값이 같아도 조각 겹침은 허용하지 않습니다"));}
            scope.defined_value(&row.value,&region,0)?;
            covered.extend(region);covered=normalize(covered);
        }
        if covered!=domain {return Err(failure(body.span,"정의역을 덮지 못하는 조각이 있습니다"));}
    }
    Ok(())
}

/// 같은 DdnProgram이 소유하는 실제 컴파일 결과. 외부 builder/serde/성공 bool
/// 입력이 없으며 source/parent/pin/원 식/숫자 owner를 확인한 compile만 생성한다.
#[derive(Default)]
pub(super) struct CheckedCalculations {
    source:String,
    parents:std::collections::BTreeMap<ddonirang_lang::NodeId,CheckedParent>,
}
struct CheckedParent {
    name:String, span:Span, pin:ddonirang_lang::ParamPin,
    kind:ddonirang_lang::SeedKind, result:Option<ddonirang_lang::TypeRef>, alternates:Vec<String>,
    domain:Set, rows:Vec<(Set,Expr)>,
}
impl CheckedCalculations {
    pub(super) fn parents_len(&self)->usize{self.parents.len()}
    #[cfg(test)] pub(super) fn count(&self)->usize{self.parents.len()}
    pub(super) fn compile(program:&CanonProgram,numeric:Option<&crate::numeric_constraint_plan::NumericConstraintPlan>,source:&str)
        ->Result<Self,ParseError> {
        verify_source_partitions(program)?;
        let mut result=Self{source:source.into(),parents:Default::default()};
        if program.origin.piecewise_definitions.is_empty(){return Ok(result);}
        let numeric=numeric.ok_or_else(||failure(Span{start:0,end:0},"같은 공통 숫자 owner가 없습니다"))?;
        for TopLevelItem::SeedDef(seed) in &program.items {
            let Some(body)=program.origin.piecewise_definitions.get(&seed.id) else{continue;};
            if !matches!(seed.seed_kind,ddonirang_lang::SeedKind::Semssi|ddonirang_lang::SeedKind::ValueFunc)
                || seed.body.is_some() || seed.proof_header.is_some() {
                return Err(failure(seed.span,"정적 단항 계산의 실제 부모 종류가 다릅니다"));
            }
            let pin=&seed.params[0]; // verify_source_partitions의 실제 단항/필수 검사 뒤.
            let q=crate::numeric_constraint_plan::NumericKind::Rational.type_ref();
            let owner=crate::numeric_constraint_plan::BindingOwner{scope:seed.id,declaration:pin.id};
            if numeric.bindings.get(&owner)!=Some(&q) || numeric.piecewise_results.get(&seed.id)!=Some(&q) {
                return Err(failure(seed.span,"같은 부모의 입력·결과 나눔수 owner 공급이 없습니다"));
            }
            typed_condition(numeric,&body.domain,&q)?;
            let mut scope=Scope{pin:&pin.pin_name,span:body.span,remaining:4096};
            let domain=scope.condition(&body.domain,0)?;
            let mut rows=vec![];
            for row in &body.rows {
                typed_condition(numeric,&row.condition,&q)?;
                typed_expression(numeric,&row.value,&q)?;
                let region=scope.condition(&row.condition,0)?;
                // 같은 불변 원래 식을 보존하여 공통 evaluator에 선택 후에만 전달.
                rows.push((region,row.value.clone()));
            }
            result.parents.insert(seed.id,CheckedParent{name:seed.canonical_name.clone(),span:seed.span,
                pin:pin.clone(),kind:seed.seed_kind.clone(),result:seed.result_type.clone(),
                alternates:seed.alternate_forms.clone(),domain,rows});
        }
        if result.parents.len()!=program.origin.piecewise_definitions.len(){
            return Err(failure(Span{start:0,end:0},"조건별 부모 공급이 빠졌습니다"));
        }
        Ok(result)
    }
    pub(super) fn selected_expression(&self,source:&str,seed:&ddonirang_lang::SeedDef,input:&super::Value)
        ->Result<Option<Expr>,String> {
        let Some(parent)=self.parents.get(&seed.id) else{return Ok(None);};
        if source!=self.source || seed.span!=parent.span || seed.canonical_name!=parent.name || seed.body.is_some()
            || seed.seed_kind!=parent.kind || seed.result_type!=parent.result || seed.alternate_forms!=parent.alternates
            || !seed.type_params.is_empty() || seed.proof_header.is_some() || !seed.modifiers.is_empty()
            || seed.params.len()!=1 || seed.params[0].id!=parent.pin.id || seed.params[0].pin_name!=parent.pin.pin_name
            || seed.params[0].span!=parent.pin.span || seed.params[0].type_ref!=parent.pin.type_ref
            || seed.params[0].josa_list!=parent.pin.josa_list || seed.params[0].optional || seed.params[0].default_value.is_some() {
            return Err("조건별 계산: 다른 원문·부모·입력 pin의 검사 결과는 소비할 수 없습니다".into());
        }
        // 정규 값 모양이 Φ를 발급하는 것이 아니다. compile의 실제 numeric owner를
        // 확인한 같은 필수 pin에 대한 valuation만 기존 정규 나눔수에서 읽는다.
        let super::Value::Rational(value)=input else{
            return Err("조건별 계산: 검증된 입력 pin의 정규 나눔수가 필요합니다".into());
        };
        let value=value.ratio();
        if value.numer().bits()>4096 || value.denom().bits()>4096 {
            return Err("조건별 계산: 입력 표현 한도 초과로 미확인입니다".into());
        }
        if parent.domain.is_empty(){return Err("조건별 계산: 명시 빈 함수에는 적용할 입력이 없습니다".into());}
        if !contains(&parent.domain,value){return Err("조건별 계산: 입력이 정의역 밖입니다".into());}
        let mut selected=parent.rows.iter().filter(|(region,_)|contains(region,value));
        let (_,expression)=selected.next().ok_or_else(||"조건별 계산: 검사한 행이 없습니다".to_string())?;
        if selected.next().is_some(){return Err("조건별 계산: 단일 행 결속이 깨졌습니다".into());}
        Ok(Some(expression.clone()))
    }
}
fn typed_condition(numeric:&crate::numeric_constraint_plan::NumericConstraintPlan,c:&PiecewiseCondition,q:&ddonirang_lang::TypeRef)
    ->Result<(),ParseError>{
    use PiecewiseCondition as C;
    match c {
        C::ExplicitEmpty(_)=>Ok(()),
        C::Relations{terms,..}=>{for term in terms{typed_expression(numeric,term,q)?;}Ok(())},
        C::And(a,b)|C::Or(a,b)=>{typed_condition(numeric,a,q)?;typed_condition(numeric,b,q)},
        C::Not(a)=>typed_condition(numeric,a,q),
    }
}
fn typed_expression(numeric:&crate::numeric_constraint_plan::NumericConstraintPlan,e:&Expr,q:&ddonirang_lang::TypeRef)
    ->Result<(),ParseError>{
    if numeric.expressions.get(&e.id)!=Some(q){return Err(failure(e.span,"실제 식의 나눔수 owner가 다릅니다"));}
    if let Some(operand)=e.source_negated_operand(){return typed_expression(numeric,operand,q);}
    match &e.kind {
        ExprKind::Var(_)=>Ok(()), // actual preflight의 유일 formal pin scope 검사와 함께만 사용.
        ExprKind::Literal(_) if numeric.literals.get(&e.id)==Some(&crate::numeric_constraint_plan::NumericKind::Rational)=>Ok(()),
        ExprKind::Infix{left,op,right}=>{
            typed_expression(numeric,left,q)?;typed_expression(numeric,right,q)?;
            if crate::numeric_constraint_plan::numeric_operation_result_type(q,q,op).as_ref()!=Some(q){
                return Err(failure(e.span,"공통 정확 연산 선택이 다릅니다"));
            }
            Ok(())
        }
        _=>Err(failure(e.span,"정확 값식 owner 공급이 없습니다")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const FEE:&str=include_str!("../../tests/fixtures/v2_piecewise_tariff.ddn");
    fn check(source:&str)->Result<(),String>{
        let p=ddonirang_lang::parse_v25_with_mode(source,"요금.ddn",ddonirang_lang::ParseMode::Strict).map_err(|e|e.message)?;
        verify_source_partitions(&p).map_err(|e|e.message)
    }
    #[test] fn exact_partition_accepts_shared_boundary_once(){assert!(check(FEE).is_ok());}
    #[test] fn equal_values_do_not_admit_overlap(){assert!(check(&FEE.replace("3 < 사용량","3 <= 사용량")).unwrap_err().contains("겹침"));}
    #[test] fn missing_boundary_is_a_gap(){assert!(check(&FEE.replace("사용량 <= 3","사용량 < 3")).unwrap_err().contains("덮지"));}
    #[test] fn outside_piece_is_not_clipped(){assert!(check(&FEE.replace("3 < 사용량 <= 6","3 < 사용량 <= 7")).unwrap_err().contains("밖 조각"));}
    #[test] fn contradictory_domain_is_not_authored_empty(){assert!(check(&FEE.replace("0 <= 사용량 <= 6","6 < 사용량 < 0")).unwrap_err().contains("모순"));}
    #[test] fn foreign_formal_is_not_a_binder(){assert!(check(&FEE.replace("3 < 사용량 <= 6","3 < 다른입력 <= 6")).unwrap_err().contains("다른 입력"));}
    #[test] fn fractional_endpoints_are_not_rounded(){
        assert!(check(&FEE.replace("사용량 <= 3", "사용량 <= 1 / 3")
            .replace("3 < 사용량", "1 / 3 < 사용량")).is_ok());
    }
    #[test] fn point_and_complement_cover_once(){
        let source="요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 0 <= 사용량 <= 6. 10 : (0 <= 사용량 <= 6) 그리고 아님(사용량 =:= 3). 20 : 사용량 =:= 3. }.";
        assert!(check(source).is_ok());
    }
    #[test] fn explicit_empty_with_zero_rows_remains_distinct(){
        assert!(check("빈요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 비어있음. }.").is_ok());
    }
    #[test] fn common_loader_requires_typed_owner_after_partition(){
        let error=crate::ddn_runtime::DdnProgram::from_v25_source(&FEE.replace("3 < 사용량","3 <= 사용량"),"요금.ddn")
            .err().expect("겹침 원문 거부");
        assert!(error.contains("겹침"),"{error}");
        let program=crate::ddn_runtime::DdnProgram::from_v25_source(FEE,"요금.ddn").unwrap();
        assert_eq!(program.piecewise_calculations.count(),1);
        assert!(crate::ddn_runtime::DdnProgram::from_v25_source(&FEE.replace("나눔수","셈수"),"다른씨.ddn").is_err());
        assert!(crate::ddn_runtime::DdnProgram::from_v25_source("(2 + 3) 보여주기.","일반.ddn").is_ok());
    }
    fn run(source:&str)->Result<Vec<String>,String>{
        run_entry(source,crate::runtime_surface::DEFAULT_UPDATE_NAME)
    }
    fn run_entry(source:&str,entry:&str)->Result<Vec<String>,String>{
        use crate::ddn_runtime::{DdnProgram,DdnRunner};
        let program=DdnProgram::from_v25_source(source,"실제요금.ddn")?;
        let input=ddonirang_core::InputSnapshot{tick_id:0,dt:ddonirang_core::Fixed64::from_i64(1),keys_pressed:0,
            last_key_name:String::new(),pointer_x_i32:0,pointer_y_i32:0,ai_injections:vec![],net_events:vec![],
            frame_source:ddonirang_core::InputSource::Person,rng_seed:0};
        let mut runner=DdnRunner::new(program,crate::runtime_surface::DEFAULT_UPDATE_NAME);
        if entry!=crate::runtime_surface::DEFAULT_UPDATE_NAME {runner.configure_project_entry(entry)?;}
        let output=runner.run_update(&ddonirang_core::NuriWorld::new(),
            &input,&std::collections::HashMap::new()).map_err(|error|format!("{error:?}"))?;
        Ok(output.display_outputs.iter().map(|v|v.text.clone()).collect())
    }
    #[test] fn actual_fee_interior_boundaries_and_outside(){
        for (input,expected) in [("0","0/1"),("3","30/1"),("4","50/1"),("6","90/1")] {
            let source=format!("{FEE}\n((사용량={input}) 요금) 보여주기.");
            assert_eq!(run(&source).unwrap(),[expected],"{source}");
        }
        assert!(run(&format!("{FEE}\n((사용량=7) 요금) 보여주기.")).unwrap_err().contains("정의역 밖"));
    }
    #[test] fn named_entry_tick_keeps_actual_rational_resource_write_owner(){
        let source=format!("{FEE}\n채비 {{ 횟수:바른수 := 0. 결과:나눔수 := 0. }}.\n앱:움직씨 := {{ (매마디)마다 {{ 횟수 <- 횟수 + 1. 결과 <- (사용량=4) 요금. 결과 보여주기. 횟수 보여주기. }} }}.");
        assert_eq!(run_entry(&source,"앱").unwrap(),["50/1","1"]);
        // A local declaration must not be projected as the root's write owner.
        let shadow=source.replace("횟수 <- 횟수 + 1.","채비 { 결과:나눔수 := 2. }. 횟수 <- 횟수 + 1.");
        assert!(run_entry(&shadow,"앱").is_err());
    }
    #[test] fn active_region_preserves_uncancelled_denominator_and_lazy_row(){
        let source="요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: -3 <= 사용량 <= 3. 1 / 사용량 : 0 < 사용량 <= 3. 10 : -3 <= 사용량 <= 0. }.\n((사용량=0) 요금) 보여주기.";
        assert_eq!(run(source).unwrap(),["10/1"]);
        let invalid=source.replace("1 / 사용량","사용량 / 사용량").replace("0 < 사용량 <= 3","0 <= 사용량 <= 3")
            .replace("사용량 <= 0","사용량 < 0");
        assert!(run(&invalid).unwrap_err().contains("원래 분모"));
    }
    #[test] fn fractional_value_is_exact_not_fixed64(){
        let source=FEE.replace("10 * 사용량","사용량 / 3");
        assert_eq!(run(&format!("{source}\n((사용량=1) 요금) 보여주기.")).unwrap(),["1/3"]);
    }
    #[test] fn stale_context_and_other_parent_pin_are_rejected(){
        let program=crate::ddn_runtime::DdnProgram::from_v25_source(FEE,"요금.ddn").unwrap();
        let seed=program.program.items.iter().map(|TopLevelItem::SeedDef(seed)|seed).find(|seed|seed.canonical_name=="요금").unwrap();
        let input=super::super::make_rational_pack_from_big_rational(&Q::from_integer(BigInt::from(4)));
        assert!(program.piecewise_calculations.selected_expression("stale",seed,&input).is_err());
        let mut other=seed.clone();other.params[0].id+=1;
        assert!(program.piecewise_calculations.selected_expression(&program.source_identity,&other,&input).is_err());
        let mut other=seed.clone();other.params[0].optional=true;
        assert!(program.piecewise_calculations.selected_expression(&program.source_identity,&other,&input).is_err());
        let mut other=seed.clone();other.result_type=Some(crate::numeric_constraint_plan::NumericKind::Approximate.type_ref());
        assert!(program.piecewise_calculations.selected_expression(&program.source_identity,&other,&input).is_err());
    }
    #[test] fn capture_and_unissued_numeric_owner_do_not_admit(){
        assert!(run(&FEE.replace("10 * 사용량","다른값 + 사용량")).unwrap_err().contains("capture"));
        let p=ddonirang_lang::parse_v25_with_mode(FEE,"요금.ddn",ddonirang_lang::ParseMode::Strict).unwrap();
        assert!(CheckedCalculations::compile(&p,Some(&Default::default()),"가짜 성공").is_err());
    }
    #[test] fn unsupported_denominator_and_budget_stay_unknown(){
        assert!(run(&FEE.replace("10 * 사용량","1 / (사용량 * 사용량 + 1)")).unwrap_err().contains("미확인"));
        let deep=format!("사용량{}"," + 1".repeat(70));
        assert!(run(&FEE.replace("10 * 사용량",&deep)).unwrap_err().contains("미확인"));
    }
    #[test] fn explicit_empty_application_is_not_a_success_value(){
        let source="빈요금 (사용량:나눔수~을 --> 나눔수) := 조건별 { 정의역: 비어있음. }.\n((사용량=0) 빈요금) 보여주기.";
        assert!(run(source).unwrap_err().contains("명시 빈 함수"));
    }
    #[test] fn interval_complement_keeps_open_closed_points(){
        let q=Q::from_integer(BigInt::from(3));
        let point=vec![Interval{low:Some(Edge{value:q.clone(),closed:true}),high:Some(Edge{value:q,closed:true})}];
        assert!(intersection(&point,&complement(&point)).is_empty());
        let mut union=point.clone();union.extend(complement(&point));assert_eq!(normalize(union),universe());
    }
}
