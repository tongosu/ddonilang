//! Opt-in host observations only. No source, identity, seed or storage bytes.
//! Buffer spans on the owner thread; emit only after dispatch has returned.
use std::{cell::RefCell, io::Write, time::Instant};

thread_local! {
    static ACTIVE: RefCell<Option<Vec<(&'static str, u128)>>> = const { RefCell::new(None) };
}

pub(crate) fn submitted() -> Option<Instant> {
    std::env::var_os("DDN_PRIVATE_STORAGE_TIMING")
        .is_some_and(|value| value == "1").then(Instant::now)
}

pub(crate) struct Request(Option<(Instant, u128)>);
impl Request {
    pub(crate) fn begin(submitted: Option<Instant>) -> Self {
        let Some(submitted) = submitted else { return Self(None) };
        let now = Instant::now();
        let enabled = ACTIVE.with(|active| {
            let Ok(mut active) = active.try_borrow_mut() else { return false };
            if active.is_some() { return false }
            *active = Some(Vec::new());
            true
        });
        Self(enabled.then_some((now, now.duration_since(submitted).as_micros())))
    }
}

pub(crate) struct Span(Option<(&'static str, Instant)>);
impl Span {
    pub(crate) fn start(name: &'static str) -> Self {
        let enabled = ACTIVE.with(|active| active.try_borrow().is_ok_and(|v| v.is_some()));
        Self(enabled.then(|| (name, Instant::now())))
    }
}
impl Drop for Span {
    fn drop(&mut self) {
        if let Some((name, started)) = self.0 {
            ACTIVE.with(|active| {
                if let Ok(mut active) = active.try_borrow_mut() {
                    if let Some(samples) = active.as_mut() {
                        // Only diagnostic samples are bounded; product data is untouched.
                        if samples.len() < 128 { samples.push((name, started.elapsed().as_micros())); }
                    }
                }
            });
        }
    }
}
impl Drop for Request {
    fn drop(&mut self) {
        let Some((started, queued_us)) = self.0 else { return };
        let handler_us = started.elapsed().as_micros();
        let samples = ACTIVE.with(|active| active.try_borrow_mut().ok().and_then(|mut v| v.take()));
        let Some(samples) = samples else { return };
        let mut line = format!("DDN_PRIVATE_STORAGE_TIMING {{\"phase\":\"owner_request\",\"queued_us\":{queued_us},\"elapsed_us\":{handler_us},\"samples\":[");
        for (index, (name, elapsed)) in samples.iter().enumerate() {
            if index != 0 { line.push(','); }
            line.push_str(&format!("{{\"phase\":\"{name}\",\"elapsed_us\":{elapsed}}}"));
        }
        line.push_str("]}\n");
        // Never replace a product result on diagnostic I/O failure.
        let _ = std::io::stderr().write_all(line.as_bytes());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inactive_span_has_no_clock_or_buffer() {
        assert!(Span::start("inactive").0.is_none());
        assert!(ACTIVE.with(|v| v.borrow().is_none()));
    }
    #[test]
    fn request_captures_nested_spans_without_emitting_in_span_drop() {
        let request = Request::begin(Some(Instant::now()));
        { let _outer = Span::start("outer"); { let _inner = Span::start("inner"); } }
        ACTIVE.with(|v| {
            let data = v.borrow();
            let names: Vec<_> = data.as_ref().into_iter().flatten().map(|v| v.0).collect();
            assert_eq!(names, ["inner", "outer"]);
        });
        drop(request);
        assert!(ACTIVE.with(|v| v.borrow().is_none()));
    }
}
