use crate::units::UnitDim;

pub type TickId = u64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaultContext {
    pub tick_id: TickId,
    /// SSOT의 "location"에 해당 (엔진/런타임에서 더 정교한 좌표로 확장 가능)
    pub location: &'static str,
    pub source_span: Option<SourceSpan>,
    pub expr: Option<ExprTrace>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: String,
    pub start_line: u32,
    pub start_col: Option<u32>,
    pub end_line: u32,
    pub end_col: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExprTrace {
    pub tag: String,
    pub text: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArithmeticFaultKind {
    DivByZero,
    DimensionMismatch { left: UnitDim, right: UnitDim },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Signal {
    /// SSOT: 산술고장
    ArithmeticFault { ctx: FaultContext, kind: ArithmeticFaultKind },
    /// SSOT: 알림(이름 기반 최소 이벤트)
    Alrim { name: &'static str },
    /// SSOT: 진단말 이벤트(geoul.diag.jsonl)
    Diag { event: DiagEvent },
}

impl Signal {
    pub fn name(&self) -> &'static str {
        match self {
            Signal::ArithmeticFault {
                kind: ArithmeticFaultKind::DimensionMismatch { .. },
                ..
            } => "차원고장",
            Signal::ArithmeticFault { .. } => "산술고장",
            Signal::Alrim { name } => name,
            Signal::Diag { .. } => "diag",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagEvent {
    pub madi: TickId,
    pub seq: u64,
    pub fault_id: String,
    pub rule_id: String,
    pub reason: String,
    pub sub_reason: Option<String>,
    pub mode: Option<String>,
    pub contract_kind: Option<String>,
    pub origin: String,
    pub targets: Vec<String>,
    pub sam_hash: Option<String>,
    pub source_span: Option<SourceSpan>,
    pub expr: Option<ExprTrace>,
    pub message: Option<String>,
}

pub trait SignalSink {
    fn emit(&mut self, signal: Signal);
}

#[derive(Default)]
pub struct VecSignalSink {
    pub signals: Vec<Signal>,
    pub diag_events: Vec<DiagEvent>,
}

impl SignalSink for VecSignalSink {
    fn emit(&mut self, signal: Signal) {
        match &signal {
            Signal::Diag { event } => self.diag_events.push(event.clone()),
            _ => self.signals.push(signal),
        }
    }
}
