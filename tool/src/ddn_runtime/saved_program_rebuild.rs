//! Request-local pure compilation reuse. Never retains a runner, registration,
//! source admission, durable receipt, or execution permission across requests.
use super::{AgeTarget, DdnProgram, ParseMode};
use std::cell::RefCell;

// Internal cache eligibility, not an accepted-source or language limit. Larger
// sources still compile normally. One entry bounds retained compilation graphs.
const MAX_KEY_BYTES: usize = 64 * 1024;
struct Entry {
    source: String,
    path: String,
    mode: ParseMode,
    age: AgeTarget,
    template: DdnProgram,
}
thread_local! {
    static REQUEST: RefCell<Option<Option<Entry>>> = const { RefCell::new(None) };
}

// Private test observations only: no production counter, timer, output or wire.
#[cfg(test)]
#[derive(Default)]
struct Counts {
    calls:u64, hits:u64, builds:u64, failed:u64, ineligible:u64,
    refreshes:u64, build_ns:u128, refresh_ns:u128, clone_ns:u128,
}
#[cfg(test)]
thread_local! { static COUNTS: RefCell<Option<Counts>> = const { RefCell::new(None) }; }
#[cfg(test)]
fn observe(update:impl FnOnce(&mut Counts)) {
    COUNTS.with(|counts| {if let Some(counts)=counts.borrow_mut().as_mut(){update(counts);}});
}
#[cfg(test)]
pub(super) struct RegisteredRefreshObservation(Option<std::time::Instant>);
#[cfg(test)]
impl RegisteredRefreshObservation {
    pub(super) fn begin()->Self {
        Self(COUNTS.with(|counts|counts.borrow().as_ref().map(|_|std::time::Instant::now())))
    }
}
#[cfg(test)]
impl Drop for RegisteredRefreshObservation {
    fn drop(&mut self) {
        if let Some(start)=self.0.take(){observe(|counts|{
            counts.refreshes+=1;counts.refresh_ns+=start.elapsed().as_nanos();
        });}
    }
}

/// Constructed on the actual serialized Native owner worker, not HTTP threads.
/// Nested scopes preserve the outer scope; only its owner clears the entry.
pub(crate) struct RequestScope {
    owner: bool,
    // A TLS guard must be dropped on the same worker that created it.
    _worker: std::marker::PhantomData<std::rc::Rc<()>>,
}
impl RequestScope {
    pub(crate) fn begin() -> Self {
        let owner=REQUEST.with(|state| {
            let mut state=state.borrow_mut();
            if state.is_some() { false } else { *state=Some(None); true }
        });
        #[cfg(test)]
        if owner {
            COUNTS.with(|counts| *counts.borrow_mut()=
                (std::env::var("DDN_WO25_TEST_COMPILE_OBSERVATION").as_deref()==Ok("1"))
                    .then(Counts::default));
        }
        Self { owner, _worker: std::marker::PhantomData }
    }
}
impl Drop for RequestScope {
    fn drop(&mut self) {
        if self.owner {
            REQUEST.with(|state| *state.borrow_mut()=None);
            #[cfg(test)]
            COUNTS.with(|counts| {if let Some(counts)=counts.borrow_mut().take(){
                eprintln!("WO25_TEST_COMPILE_OBSERVATION {}",serde_json::json!({
                    "scope":"native_request_test_only","calls":counts.calls,"hits":counts.hits,
                    "builds":counts.builds,"failed":counts.failed,"ineligible":counts.ineligible,
                    "registered_refreshes":counts.refreshes,"build_ns":counts.build_ns,
                    "registered_refresh_ns":counts.refresh_ns,"hit_clone_ns":counts.clone_ns,
                }));
            }});
        }
    }
}

pub(super) fn compile(source:&str,path:&str,mode:ParseMode,age:AgeTarget)->Result<DdnProgram,String> {
    compile_with(source,path,mode,age,|| {
        DdnProgram::parse_captured_source(source,path,mode,true,age,None,
            Some(crate::runtime_surface::bogae_host_numeric_supply()))
    })
}

fn compile_with(
    source:&str,path:&str,mode:ParseMode,age:AgeTarget,
    build:impl FnOnce()->Result<DdnProgram,String>,
)->Result<DdnProgram,String> {
    let eligible=source.len().checked_add(path.len()).is_some_and(|len|len<=MAX_KEY_BYTES);
    #[cfg(test)]
    let lookup_start=std::time::Instant::now();
    #[cfg(test)]
    observe(|counts|{counts.calls+=1;if !eligible{counts.ineligible+=1;}});
    if eligible {
        if let Some(program)=REQUEST.with(|state| {
            let state=state.borrow();
            state.as_ref().and_then(Option::as_ref).filter(|entry|
                entry.source==source && entry.path==path && entry.mode==mode && entry.age==age
            ).map(|entry|entry.template.clone())
        }) {
            #[cfg(test)]
            observe(|counts|{counts.hits+=1;counts.clone_ns+=lookup_start.elapsed().as_nanos();});
            return Ok(program);
        }
    }
    // No RefCell borrow is held during compilation or a possible error/unwind.
    // Store only the unbound result. The caller replays seals before reaching
    // this function, then attaches its CURRENT saved sources and registration.
    #[cfg(test)]
    let build_start=std::time::Instant::now();
    let built=build();
    #[cfg(test)]
    observe(|counts|{
        counts.builds+=1;counts.build_ns+=build_start.elapsed().as_nanos();
        if built.is_err(){counts.failed+=1;}
    });
    let program=built?;
    if eligible {
        REQUEST.with(|state| {
            let mut state=state.borrow_mut();
            if let Some(slot)=state.as_mut() {
                *slot=Some(Entry{source:source.into(),path:path.into(),mode,age,template:program.clone()});
            }
        });
    }
    Ok(program)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    fn template()->DdnProgram {
        DdnProgram::parse_captured_source("값 := 1.","검사.ddn",ParseMode::Strict,true,
            AgeTarget::Age0,None,Some(crate::runtime_surface::bogae_host_numeric_supply())).unwrap()
    }
    fn request(source:&str,path:&str,mode:ParseMode,age:AgeTarget,count:&Cell<usize>)->DdnProgram {
        compile_with(source,path,mode,age,||{count.set(count.get()+1);Ok(template())}).unwrap()
    }
    #[test]
    fn saved_rebuild_request_scope_and_unbound_clones() {
        let count=Cell::new(0);
        request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
        request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
        assert_eq!(count.get(),2); // No ambient cache outside an owner request.
        {
            let _scope=RequestScope::begin();
            let mut first=request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
            first.source_identity="changed clone".into();
            first.registered_atom_literals.clear();
            {
                let _nested=RequestScope::begin();
                let second=request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
                assert_ne!(second.source_identity,first.source_identity);
                assert!(second.saved_project_sources.is_none());
                assert!(second.temporary_registration.is_none());
                assert!(second.temporary_subject_bootstrap.is_none());
                assert!(second.registered_nominal_catalog.is_none());
            }
            request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
            assert_eq!(count.get(),3);
        }
        let _scope=RequestScope::begin();
        request("a","p",ParseMode::Strict,AgeTarget::Age0,&count);
        assert_eq!(count.get(),4); // Another request must rebuild.
    }
    #[test]
    fn saved_rebuild_exact_keys_large_fallback_and_errors() {
        let _scope=RequestScope::begin();
        let count=Cell::new(0);
        for (source,path,mode,age) in [
            ("a","p",ParseMode::Strict,AgeTarget::Age0),
            ("b","p",ParseMode::Strict,AgeTarget::Age0),
            ("b","q",ParseMode::Strict,AgeTarget::Age0),
            ("b","q",ParseMode::HistoricalMigrationEvidence,AgeTarget::Age0),
            ("b","q",ParseMode::HistoricalMigrationEvidence,AgeTarget::Age1),
        ] { request(source,path,mode,age,&count); }
        assert_eq!(count.get(),5);
        let large="x".repeat(MAX_KEY_BYTES+1);
        request(&large,"p",ParseMode::Strict,AgeTarget::Age0,&count);
        request(&large,"p",ParseMode::Strict,AgeTarget::Age0,&count);
        assert_eq!(count.get(),7);
        for _ in 0..2 {
            let failed=compile_with("error","p",ParseMode::Strict,AgeTarget::Age0,||{
                count.set(count.get()+1);Err("original failure".into())
            });
            assert!(matches!(failed,Err(ref message) if message=="original failure"));
        }
        assert_eq!(count.get(),9);
    }
    #[test]
    fn saved_rebuild_unwind_and_thread_boundary_clear() {
        assert!(std::panic::catch_unwind(|| {
            let _scope=RequestScope::begin();
            request("a","p",ParseMode::Strict,AgeTarget::Age0,&Cell::new(0));
            panic!("test unwind");
        }).is_err());
        assert!(REQUEST.with(|state|state.borrow().is_none()));
        let _scope=RequestScope::begin();
        request("a","p",ParseMode::Strict,AgeTarget::Age0,&Cell::new(0));
        std::thread::spawn(||assert!(REQUEST.with(|state|state.borrow().is_none()))).join().unwrap();
    }
}
