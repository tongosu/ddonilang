//! RD706 authored conditions in the common parser. These are syntax facts,
//! never a resolved input owner, mathematical participation or checked proof.
use super::*;

impl Parser {
    pub(super) fn parse_piecewise_definition(&mut self,start:Span,name:String,
        alternate_forms:Vec<String>,kind:SeedKind,params:Vec<ParamPin>,
        result_type:Option<TypeRef>,type_params:Vec<TypeParamDef>,
        proof_header:Option<ProofHeader>)->Result<SeedDef,ParseError>{
        if params.len()!=1 || result_type.is_none() || !type_params.is_empty()
            || proof_header.is_some() || params[0].optional || params[0].default_value.is_some() {
            return Err(self.error("조건별 함수의 필수 입력 하나와 명시 결과씨"));
        }
        self.advance();self.expect(&TokenKind::LBrace,"조건별 본문의 {")?;
        if self.current().raw!="정의역" {return Err(self.error("첫 항목 정의역: 조건."));}
        self.advance();self.expect(&TokenKind::Colon,"정의역의 :")?;
        let domain=self.parse_piecewise_or()?;
        self.expect(&TokenKind::Dot,"정의역 조건의 마침표")?;
        let mut rows=Vec::new();
        while !self.check(&TokenKind::RBrace) {
            if self.check(&TokenKind::Eof) {return Err(self.error("조건별 본문의 }"));}
            if self.current().raw=="정의역" {return Err(self.error("정의역 항목은 하나만"));}
            let value=self.parse_addition()?;
            self.expect(&TokenKind::Colon,"값식과 조건 사이의 :")?;
            let condition=self.parse_piecewise_or()?;
            self.expect(&TokenKind::Dot,"조건별 행의 마침표")?;
            if matches!(condition,PiecewiseCondition::ExplicitEmpty(_)) {
                return Err(self.error("빈 조각 행은 쓸 수 없습니다"));
            }
            rows.push(PiecewiseRow{value,condition});
        }
        self.expect(&TokenKind::RBrace,"조건별 본문의 }")?;
        self.expect(&TokenKind::Dot,"조건별 선언의 마침표")?;
        if matches!(domain,PiecewiseCondition::ExplicitEmpty(_))&&!rows.is_empty(){
            return Err(self.error("명시 빈 정의역에는 행을 쓰지 않습니다"));
        }
        let id=self.next_id();let span=start.merge(&self.previous_span());
        let raw=self.source.get(span.start..span.end).ok_or_else(||self.error("조건별 선언의 원문 범위"))?.to_owned();
        self.piecewise_definitions.insert(id,PiecewiseDefinition{domain,rows,span,source_declaration:raw});
        Ok(SeedDef{type_params,proof_header,id,span,canonical_name:name,alternate_forms,
            seed_kind:kind,params,result_type,body:None,modifiers:Vec::new()})
    }
    fn parse_piecewise_or(&mut self)->Result<PiecewiseCondition,ParseError>{
        let mut left=self.parse_piecewise_and()?;
        while self.current().raw=="또는" {self.advance();let right=self.parse_piecewise_and()?;
            left=PiecewiseCondition::Or(Box::new(left),Box::new(right));}Ok(left)
    }
    fn parse_piecewise_and(&mut self)->Result<PiecewiseCondition,ParseError>{
        let mut left=self.parse_piecewise_atom()?;
        while self.current().raw=="그리고" {self.advance();let right=self.parse_piecewise_atom()?;
            left=PiecewiseCondition::And(Box::new(left),Box::new(right));}Ok(left)
    }
    fn parse_piecewise_atom(&mut self)->Result<PiecewiseCondition,ParseError>{
        if self.current().raw=="비어있음" {let span=self.current_span();self.advance();return Ok(PiecewiseCondition::ExplicitEmpty(span));}
        if self.current().raw=="아님" {self.advance();self.expect(&TokenKind::LParen,"아님(조건)")?;
            let inner=self.parse_piecewise_or()?;self.expect(&TokenKind::RParen,"아님 조건의 )")?;
            return Ok(PiecewiseCondition::Not(Box::new(inner)));}
        if self.check(&TokenKind::LParen) {self.advance();let inner=self.parse_piecewise_or()?;
            self.expect(&TokenKind::RParen,"묶은 조건의 )")?;return Ok(inner);}
        let mut terms=vec![self.parse_addition()?];let mut relations=Vec::new();
        while matches!(self.current().raw.as_str(),"<"|"<="|">"|">="|"=:=") {
            relations.push(self.advance().raw);terms.push(self.parse_addition()?);
        }
        if relations.is_empty(){return Err(self.error("조건별의 수학 비교 관계"));}
        if relations.len()>1 {
            let increasing=relations.iter().all(|op|matches!(op.as_str(),"<"|"<="));
            let decreasing=relations.iter().all(|op|matches!(op.as_str(),">"|">="));
            if !increasing&&!decreasing{return Err(self.error("같은 방향의 연속 부등식"));}
        }
        Ok(PiecewiseCondition::Relations{terms,relations})
    }
}
