//! Approved S R1.1/R2: exact numeric brand and bounded saved validation.
//! Numeric validity is not source/type/custody or world publication authority.
use num_bigint::{BigInt,Sign};
use num_rational::BigRational;
use num_traits::Zero;
use std::{cell::Cell,marker::PhantomPinned,pin::Pin,sync::Arc};

const HEADER:&[u8]=b"\x10DDN-RATIONAL-SAVED-V1\0\0\x01";
const FRAME_LIMIT:usize=2_000_000;
const WORK_LIMIT:u64=8_000_000;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SavedRationalError { Invalid, NonCanonical, LimitExceeded }

#[derive(Debug,PartialEq,Eq)]
pub struct NormalizedRationalValueV1 { ratio:BigRational }
impl NormalizedRationalValueV1 {
    /// Actual exact-number factory. Execution normalization has its existing
    /// arithmetic cost owner; saved admission never calls this unmetered GCD.
    pub fn from_parts(numerator:BigInt,denominator:BigInt)->Result<Arc<Self>,SavedRationalError> {
        if denominator.is_zero() {return Err(SavedRationalError::NonCanonical);}
        Ok(Arc::new(Self{ratio:BigRational::new(numerator,denominator)}))
    }
    pub fn ratio(&self)->&BigRational {&self.ratio}
    /// Intrinsic canonical projection, NOT saved admission. Saved producers
    /// still run the shared root work/materialization profile before issuing
    /// a file. Counting this projection does not allocate magnitudes.
    pub fn canonical_projection_length(&self)->Option<usize> {
        let length=|value:&BigInt|usize::try_from(value.bits().checked_add(7)?/8).ok();
        42usize.checked_add(length(self.ratio.numer())?)?.checked_add(length(self.ratio.denom())?)
    }
    pub fn write_canonical_projection(&self,write:impl FnMut(&[u8])) {
        let (sign,mut n)=self.ratio.numer().to_bytes_be();
        if sign==Sign::NoSign {n.clear();}
        let (_,d)=self.ratio.denom().to_bytes_be();
        let sign=match sign {Sign::NoSign=>0,Sign::Plus=>1,Sign::Minus=>2};
        write_frame(sign,&n,&d,write);
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Primitive {Inspect,Small,Scan,Shift,Compare,Swap,Subtract,Trim,Reencode}

/// One pinned, non-Clone root context. No product quota override/reset API.
pub struct SavedRationalValidationContextV1 {
    remaining:Cell<u64>,spent:Cell<u64>,shifts:Cell<u64>,_pin:PhantomPinned,
}
impl SavedRationalValidationContextV1 {
    pub fn new_root()->Pin<Box<Self>> {Self::create(WORK_LIMIT)}
    fn create(limit:u64)->Pin<Box<Self>> {Box::pin(Self{remaining:Cell::new(limit),spent:Cell::new(0),shifts:Cell::new(0),_pin:PhantomPinned})}
    pub fn credits_spent(self:Pin<&Self>)->u64 {self.spent.get()}
    fn charge(self:Pin<&mut Self>,primitive:Primitive,credits:usize)->Result<(),SavedRationalError> {
        let credits=u64::try_from(credits).map_err(|_|SavedRationalError::LimitExceeded)?;
        let this=self.as_ref().get_ref();
        let remaining=this.remaining.get().checked_sub(credits).ok_or(SavedRationalError::LimitExceeded)?;
        let spent=this.spent.get().checked_add(credits).ok_or(SavedRationalError::LimitExceeded)?;
        this.remaining.set(remaining);this.spent.set(spent);
        if primitive==Primitive::Shift {this.shifts.set(this.shifts.get()+1);}
        Ok(())
    }
}

struct Parts<'a> {sign:u8,n:&'a [u8],d:&'a [u8],length:usize}
fn read_length(bytes:&[u8],at:usize)->Result<usize,SavedRationalError> {
    let end=at.checked_add(8).ok_or(SavedRationalError::LimitExceeded)?;
    let raw=bytes.get(at..end).ok_or(SavedRationalError::Invalid)?;
    let raw=u64::from_be_bytes(raw.try_into().map_err(|_|SavedRationalError::Invalid)?);
    if raw>FRAME_LIMIT as u64 {return Err(SavedRationalError::LimitExceeded);}
    usize::try_from(raw).map_err(|_|SavedRationalError::LimitExceeded)
}
fn parse_parts(bytes:&[u8])->Result<Parts<'_>,SavedRationalError> {
    // Bounded metadata only, no magnitude traversal/allocation before charge.
    if bytes.len()<42 {return Err(SavedRationalError::Invalid);}
    let sign=bytes[25];let nlen=read_length(bytes,26)?;
    let nend=34usize.checked_add(nlen).ok_or(SavedRationalError::LimitExceeded)?;
    let dlen=read_length(bytes,nend)?;
    let dstart=nend.checked_add(8).ok_or(SavedRationalError::LimitExceeded)?;
    let end=dstart.checked_add(dlen).ok_or(SavedRationalError::LimitExceeded)?;
    if end>FRAME_LIMIT {return Err(SavedRationalError::LimitExceeded);}
    Ok(Parts{sign,n:bytes.get(34..nend).ok_or(SavedRationalError::Invalid)?,
        d:bytes.get(dstart..end).ok_or(SavedRationalError::Invalid)?,length:end})
}
fn materialization(n:usize,d:usize,frame:usize)->Result<usize,SavedRationalError> {
    let align=|n:usize|n.checked_add(7).map(|n|n/8*8).ok_or(SavedRationalError::LimitExceeded);
    let opaque_shell=std::mem::size_of::<NormalizedRationalValueV1>().checked_add(2*std::mem::size_of::<usize>()).ok_or(SavedRationalError::LimitExceeded)?;
    // Additional Arc/ratio metadata is reserved before allocation on BOTH
    // sides, so save cannot admit a value that restore cannot materialize.
    align(n)?.checked_add(align(d)?).and_then(|n|n.checked_mul(3)).and_then(|n|n.checked_add(frame))
        .and_then(|n|n.checked_add(opaque_shell)).ok_or(SavedRationalError::LimitExceeded)
}
fn copy_bytes(bytes:&[u8])->Result<Vec<u8>,SavedRationalError> {
    let mut value=Vec::new();value.try_reserve_exact(bytes.len()).map_err(|_|SavedRationalError::LimitExceeded)?;
    value.extend_from_slice(bytes);Ok(value)
}
fn charge_scale(mut work:Pin<&mut SavedRationalValidationContextV1>,p:Primitive,n:usize,multiplier:usize,extra:usize)->Result<(),SavedRationalError> {
    work.as_mut().charge(p,n.checked_mul(multiplier).and_then(|n|n.checked_add(extra)).ok_or(SavedRationalError::LimitExceeded)?)
}
fn is_one(value:&[u8])->bool {value==[1]}
fn strip_twos(value:&mut Vec<u8>,mut work:Pin<&mut SavedRationalValidationContextV1>)->Result<usize,SavedRationalError> {
    let original=value.len();charge_scale(work.as_mut(),Primitive::Scan,original,1,1)?;
    let mut shift=0usize;
    for byte in value.iter().rev() {
        if *byte==0 {shift+=8;}else{shift+=byte.trailing_zeros() as usize;break;}
    }
    if shift==0 {return Ok(0);}
    charge_scale(work.as_mut(),Primitive::Shift,original,2,1)?;
    let old_bits=8*(original-1)+(8-value[0].leading_zeros() as usize);
    let new_length=(old_bits-shift+7)/8;
    value.truncate(original-shift/8);
    let bits=shift%8;
    if bits!=0 {
        let mut carry=0;
        for byte in value.iter_mut() {let next=*byte<<(8-bits);*byte=(*byte>>bits)|carry;carry=next;}
    }
    let start=value.len()-new_length;
    value.copy_within(start..,0);value.truncate(new_length);
    Ok(shift)
}
fn subtract(a:&mut Vec<u8>,b:&[u8],mut work:Pin<&mut SavedRationalValidationContextV1>)->Result<(),SavedRationalError> {
    let length=a.len();charge_scale(work.as_mut(),Primitive::Subtract,length,3,1)?;
    let mut borrow=0i16;
    for index in 0..length {
        let ai=length-1-index;
        let other=b.len().checked_sub(index+1).map(|i|b[i] as i16).unwrap_or(0);
        let value=a[ai] as i16-other-borrow;
        a[ai]=value as u8;borrow=if value<0 {1}else{0};
    }
    debug_assert_eq!(borrow,0);
    charge_scale(work.as_mut(),Primitive::Trim,length,1,1)?;
    let start=a.iter().position(|byte|*byte!=0).unwrap_or(length);
    a.copy_within(start..,0);a.truncate(length-start);Ok(())
}
fn coprime(n:&[u8],d:&[u8],mut work:Pin<&mut SavedRationalValidationContextV1>)->Result<(),SavedRationalError> {
    work.as_mut().charge(Primitive::Small,2)?;
    if n.is_empty() {return if is_one(d){Ok(())}else{Err(SavedRationalError::NonCanonical)};}
    if is_one(n)||is_one(d) {return Ok(());}
    let mut a=copy_bytes(n)?;let mut b=copy_bytes(d)?;
    let sa=strip_twos(&mut a,work.as_mut())?;
    let sb=strip_twos(&mut b,work.as_mut())?;
    if sa!=0&&sb!=0 {return Err(SavedRationalError::NonCanonical);}
    let bit_length=|v:&[u8]|8*(v.len()-1)+8-v[0].leading_zeros() as usize;
    let invariant=2*(bit_length(n)+bit_length(d))+4;
    let mut iterations=0;
    loop {
        iterations+=1;debug_assert!(iterations<=invariant,"Stein progress invariant, not an admission cap");
        work.as_mut().charge(Primitive::Small,2)?;
        if is_one(&a)||is_one(&b) {return Ok(());}
        let credits=a.len().checked_add(b.len()).and_then(|n|n.checked_add(1)).ok_or(SavedRationalError::LimitExceeded)?;
        work.as_mut().charge(Primitive::Compare,credits)?;
        let ordering=a.len().cmp(&b.len()).then_with(||a.cmp(&b));
        if ordering.is_eq() {return Err(SavedRationalError::NonCanonical);}
        if ordering.is_lt() {work.as_mut().charge(Primitive::Swap,1)?;std::mem::swap(&mut a,&mut b);}
        subtract(&mut a,&b,work.as_mut())?;
        strip_twos(&mut a,work.as_mut())?;
    }
}
fn frame(sign:u8,n:&[u8],d:&[u8])->Result<Vec<u8>,SavedRationalError> {
    let length=42usize.checked_add(n.len()).and_then(|n|n.checked_add(d.len())).ok_or(SavedRationalError::LimitExceeded)?;
    if length>FRAME_LIMIT {return Err(SavedRationalError::LimitExceeded);}
    let mut bytes=Vec::new();bytes.try_reserve_exact(length).map_err(|_|SavedRationalError::LimitExceeded)?;
    write_frame(sign,n,d,|part|bytes.extend_from_slice(part));Ok(bytes)
}
fn write_frame(sign:u8,n:&[u8],d:&[u8],mut write:impl FnMut(&[u8])) {
    write(HEADER);write(&[sign]);write(&(n.len() as u64).to_be_bytes());write(n);
    write(&(d.len() as u64).to_be_bytes());write(d);
}
fn validate(bytes:&[u8],parts:&Parts<'_>,mut work:Pin<&mut SavedRationalValidationContextV1>)->Result<(),SavedRationalError> {
    work.as_mut().charge(Primitive::Inspect,parts.length)?;
    if bytes.get(..25)!=Some(HEADER) {return Err(SavedRationalError::Invalid);}
    let minimal=|value:&[u8]|!value.is_empty()&&value[0]!=0;
    if !minimal(parts.d) || !matches!((parts.sign,parts.n.is_empty()),(0,true)|(1|2,false))
        || (!parts.n.is_empty()&&!minimal(parts.n)) {return Err(SavedRationalError::NonCanonical);}
    coprime(parts.n,parts.d,work.as_mut())?;
    work.as_mut().charge(Primitive::Reencode,parts.length)?;
    // The same canonical writer streams into an equality sink, avoiding a
    // second full-frame allocation beyond the reserved output buffer.
    let mut cursor=0usize;let mut equal=true;
    write_frame(parts.sign,parts.n,parts.d,|part| {
        let end=cursor+part.len();equal&=bytes.get(cursor..end)==Some(part);cursor=end;
    });
    if !equal||cursor!=parts.length {return Err(SavedRationalError::NonCanonical);}
    Ok(())
}

/// A decoded number remains staging data until the enclosing owner admits it.
pub struct DecodedRationalCandidateV1 {value:Arc<NormalizedRationalValueV1>,consumed:usize}
impl DecodedRationalCandidateV1 {
    pub fn value(&self)->&Arc<NormalizedRationalValueV1> {&self.value}
    pub fn consumed(&self)->usize {self.consumed}
}
pub fn decode_prefix(bytes:&[u8],work:Pin<&mut SavedRationalValidationContextV1>,
    reserve:&mut impl FnMut(usize)->Result<(),SavedRationalError>,
)->Result<DecodedRationalCandidateV1,SavedRationalError> {
    let parts=parse_parts(bytes)?;
    reserve(materialization(parts.n.len(),parts.d.len(),parts.length)?)?;
    validate(bytes,&parts,work)?;
    let sign=match parts.sign {0=>Sign::NoSign,1=>Sign::Plus,2=>Sign::Minus,_=>return Err(SavedRationalError::Invalid)};
    // Strict metered coprimality was established; no hidden BigInt GCD here.
    let ratio=BigRational::new_raw(BigInt::from_bytes_be(sign,parts.n),BigInt::from_bytes_be(Sign::Plus,parts.d));
    Ok(DecodedRationalCandidateV1{value:Arc::new(NormalizedRationalValueV1{ratio}),consumed:parts.length})
}
pub fn encode(value:&NormalizedRationalValueV1,work:Pin<&mut SavedRationalValidationContextV1>,
    reserve:&mut impl FnMut(usize)->Result<(),SavedRationalError>,
)->Result<Vec<u8>,SavedRationalError> {
    let magnitude_len=|value:&BigInt|usize::try_from((value.bits()+7)/8).map_err(|_|SavedRationalError::LimitExceeded);
    let nlen=magnitude_len(value.ratio.numer())?;let dlen=magnitude_len(value.ratio.denom())?;
    let length=42usize.checked_add(nlen).and_then(|n|n.checked_add(dlen)).ok_or(SavedRationalError::LimitExceeded)?;
    if length>FRAME_LIMIT {return Err(SavedRationalError::LimitExceeded);}
    reserve(materialization(nlen,dlen,length)?)?;
    let (sign,mut n)=value.ratio.numer().to_bytes_be();
    if sign==Sign::NoSign {n.clear();}
    let (_,d)=value.ratio.denom().to_bytes_be();
    let sign=match sign {Sign::NoSign=>0,Sign::Plus=>1,Sign::Minus=>2};
    let bytes=frame(sign,&n,&d)?;
    let parts=Parts{sign,n:&n,d:&d,length};validate(&bytes,&parts,work)?;Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest,Sha256};
    fn reserve(_:usize)->Result<(),SavedRationalError>{Ok(())}
    #[test]
    fn saved_rational_exact_frames_and_profile_primitive_costs() {
        for (n,d,cost) in [(1,3,90),(3,10,116),(-2,3,99),(0,1,88),(2,1,90)] {
            let value=NormalizedRationalValueV1::from_parts(n.into(),d.into()).unwrap();
            let mut work=SavedRationalValidationContextV1::new_root();
            let bytes=encode(&value,work.as_mut(),&mut reserve).unwrap();
            assert_eq!(work.as_ref().credits_spent(),cost);
            if n==1&&d==3 {assert_eq!(format!("{:x}",Sha256::digest(&bytes)),"7022e00436f7ed8aa070fbcc7c8f1dfd32a02a96a02a6b152d1a371d2e3d7fa6");}
            let mut read_work=SavedRationalValidationContextV1::new_root();
            let read=decode_prefix(&bytes,read_work.as_mut(),&mut reserve).unwrap();
            assert_eq!(read.value().ratio(),value.ratio());assert_eq!(read.consumed(),bytes.len());
            assert_eq!(read_work.as_ref().credits_spent(),cost);
        }
    }
    #[test]
    fn saved_rational_shared_context_charges_before_primitive() {
        let bytes=frame(1,&[3],&[10]).unwrap();
        let mut exact=SavedRationalValidationContextV1::create(116);
        assert!(decode_prefix(&bytes,exact.as_mut(),&mut reserve).is_ok());
        let mut short=SavedRationalValidationContextV1::create(115);
        assert!(matches!(decode_prefix(&bytes,short.as_mut(),&mut reserve),Err(SavedRationalError::LimitExceeded)));
        let mut shared=SavedRationalValidationContextV1::create(205);
        assert!(decode_prefix(&frame(1,&[1],&[3]).unwrap(),shared.as_mut(),&mut reserve).is_ok());
        assert!(matches!(decode_prefix(&bytes,shared.as_mut(),&mut reserve),Err(SavedRationalError::LimitExceeded)));
        assert_eq!(shared.as_ref().credits_spent(),162);
        let mut n=vec![0;33];n[0]=1;let mut d=vec![0;32];d[0]=128;d[31]=1;
        let mut shift=SavedRationalValidationContextV1::create(143);
        assert!(matches!(decode_prefix(&frame(1,&n,&d).unwrap(),shift.as_mut(),&mut reserve),Err(SavedRationalError::LimitExceeded)));
        assert_eq!(shift.as_ref().get_ref().shifts.get(),0);
    }
    #[test]
    fn saved_rational_strict_candidate_and_preallocation_rejection() {
        for bytes in [frame(1,&[2],&[6]).unwrap(),frame(0,&[],&[2]).unwrap(),frame(1,&[0,1],&[3]).unwrap(),frame(3,&[1],&[3]).unwrap(),frame(1,&[1],&[]).unwrap()] {
            let mut work=SavedRationalValidationContextV1::new_root();
            assert!(decode_prefix(&bytes,work.as_mut(),&mut reserve).is_err());
        }
        let mut bytes=frame(1,&[1],&[3]).unwrap();bytes[26..34].copy_from_slice(&u64::MAX.to_be_bytes());
        let mut work=SavedRationalValidationContextV1::new_root();
        let mut allocated=false;
        assert!(matches!(decode_prefix(&bytes,work.as_mut(),&mut |_|{allocated=true;Ok(())}),Err(SavedRationalError::LimitExceeded)));
        assert!(!allocated);
        let mut work=SavedRationalValidationContextV1::new_root();
        assert!(matches!(decode_prefix(&frame(1,&[1],&[3]).unwrap(),work.as_mut(),&mut |_|Err(SavedRationalError::LimitExceeded)),Err(SavedRationalError::LimitExceeded)));
        assert_eq!(work.as_ref().credits_spent(),0);
    }
    #[test]
    fn saved_rational_large_valid_values_and_limit_before_expensive_subtract() {
        let mut a=BigInt::from(0);let mut b=BigInt::from(1);
        let mut previous=b.clone();
        for index in 1..=4097 {
            let next=&a+&b;a=b;b=next;
            if [257,1025,4097].contains(&index) {
                let (_,n)=a.to_bytes_be();let (_,d)=previous.to_bytes_be();
                let mut work=SavedRationalValidationContextV1::new_root();
                decode_prefix(&frame(1,&n,&d).unwrap(),work.as_mut(),&mut reserve).unwrap();
                assert_eq!(work.as_ref().credits_spent(),match index {257=>15326,1025=>204218,_=>3216249});
            }
            previous=a.clone();
        }
        let n=vec![255;999979];let mut d=n.clone();d[0]=127;
        let bytes=frame(1,&n,&d).unwrap();assert_eq!(bytes.len(),FRAME_LIMIT);
        let mut work=SavedRationalValidationContextV1::new_root();
        assert!(matches!(decode_prefix(&bytes,work.as_mut(),&mut reserve),Err(SavedRationalError::LimitExceeded)));
        assert_eq!(work.as_ref().credits_spent(),5_999_923);
    }
}
