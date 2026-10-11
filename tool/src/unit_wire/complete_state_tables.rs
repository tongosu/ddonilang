//! RD-670 tables: framing is BE; saved ResourceValue bodies retain their own
//! existing codec. A borrowed resource body is not typed/custody admission.
use crate::snapshot_v2_durable_transaction::SnapshotV2DurabilityFailureV1 as Failure;
use ddonirang_core::Fixed64;
use ddonirang_core::platform::ResourceValue;
use std::collections::BTreeMap;

const TOTAL_MAX: usize = 2_000_000;

struct Reader<'a> { bytes: &'a [u8], at: usize }
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], Failure> {
        let end = self.at.checked_add(n).ok_or(Failure::LimitExceeded)?;
        let out = self.bytes.get(self.at..end).ok_or(Failure::InvalidInput)?;
        self.at = end;
        Ok(out)
    }
    fn u64(&mut self) -> Result<u64, Failure> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?))
    }
    fn name(&mut self) -> Result<&'a str, Failure> {
        let n = u32::from_be_bytes(self.take(4)?.try_into().map_err(|_|Failure::InvalidInput)?);
        let n = usize::try_from(n).map_err(|_|Failure::LimitExceeded)?;
        std::str::from_utf8(self.take(n)?).map_err(|_|Failure::InvalidInput)
    }
    fn done(&self) -> Result<(), Failure> {
        if self.at == self.bytes.len() { Ok(()) } else { Err(Failure::InvalidInput) }
    }
}

fn reader(bytes: &[u8]) -> Result<(Reader<'_>, u64), Failure> {
    if bytes.len() > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    let mut r = Reader {bytes, at:0};
    let count = r.u64()?;
    // Both row forms have at least a four-byte name length and eight bytes.
    if count > (bytes.len().saturating_sub(8)/12) as u64 { return Err(Failure::InvalidInput); }
    Ok((r,count))
}

fn ordered<'a>(previous: &mut Option<&'a str>, name: &'a str) -> Result<(), Failure> {
    if previous.is_some_and(|p|p.as_bytes() >= name.as_bytes()) { return Err(Failure::InvalidInput); }
    *previous = Some(name);
    Ok(())
}

/// No allocation or value interpretation; caller must admit every value body
/// through the shared typed codec before publishing a restore candidate.
pub(crate) fn check_resource_table_rows(bytes: &[u8]) -> Result<(), Failure> {
    let (mut r,count) = reader(bytes)?;
    let mut previous = None;
    for _ in 0..count {
        let name = r.name()?;
        ordered(&mut previous,name)?;
        let length = usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        if length == 0 { return Err(Failure::InvalidInput); }
        if length > TOTAL_MAX { return Err(Failure::LimitExceeded); }
        r.take(length)?;
    }
    r.done()
}

/// Shared typed value decoding. Handle/Subject owner and complete-world custody
/// checks are subsequent gates; this map alone is never restore authorization.
pub(crate) fn decode_resource_table(bytes: &[u8]) -> Result<BTreeMap<String,ResourceValue>, Failure> {
    check_resource_table_rows(bytes)?;
    let (mut r,count)=reader(bytes)?;
    let mut values=BTreeMap::new();
    let mut budget=super::saved_resource_value::ValueDecodeBudget::new();
    for _ in 0..count {
        let name=r.name()?.to_owned();
        let length=usize::try_from(r.u64()?).map_err(|_|Failure::LimitExceeded)?;
        let value=super::saved_resource_value::decode_saved_resource_value_with_budget(r.take(length)?,&mut budget)?;
        values.insert(name,value);
    }
    r.done()?;
    Ok(values)
}

pub(crate) fn check_parameter_table(bytes: &[u8]) -> Result<(), Failure> {
    let (mut r,count) = reader(bytes)?;
    let mut previous = None;
    for _ in 0..count {
        let name = r.name()?;
        ordered(&mut previous,name)?;
        r.take(8)?;
    }
    r.done()
}

/// Validate the whole table before materializing; no last-wins or partial map.
pub(crate) fn decode_parameter_table(bytes: &[u8]) -> Result<BTreeMap<String,Fixed64>, Failure> {
    check_parameter_table(bytes)?;
    let (mut r,count) = reader(bytes)?;
    let mut values = BTreeMap::new();
    for _ in 0..count {
        let name = r.name()?.to_owned();
        let raw = i64::from_be_bytes(r.take(8)?.try_into().map_err(|_|Failure::InvalidInput)?);
        values.insert(name,Fixed64::from_raw_i64(raw));
    }
    r.done()?;
    Ok(values)
}

pub(crate) fn encode_parameter_table(values: &BTreeMap<String,Fixed64>) -> Result<Vec<u8>, Failure> {
    let mut size = 8usize;
    for name in values.keys() {
        u32::try_from(name.len()).map_err(|_|Failure::LimitExceeded)?;
        size = size.checked_add(12).and_then(|s|s.checked_add(name.len())).ok_or(Failure::LimitExceeded)?;
        if size > TOTAL_MAX { return Err(Failure::LimitExceeded); }
    }
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(size).map_err(|_|Failure::LimitExceeded)?;
    bytes.extend_from_slice(&(values.len() as u64).to_be_bytes());
    for (name,value) in values {
        bytes.extend_from_slice(&(name.len() as u32).to_be_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.extend_from_slice(&value.raw_i64().to_be_bytes());
    }
    check_parameter_table(&bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rd670_resource_table_uses_actual_core_value_bytes_and_rejects_invalid_leaf() {
        let value=ResourceValue::List(vec![ResourceValue::Fixed64(Fixed64::from_raw_i64(-1)),ResourceValue::Bool(true)]);
        let mut raw=Vec::new();value.encode_canon(&mut raw);
        let mut table=1u64.to_be_bytes().to_vec();table.extend_from_slice(&3u32.to_be_bytes());table.extend_from_slice("값".as_bytes());
        table.extend_from_slice(&(raw.len() as u64).to_be_bytes());table.extend_from_slice(&raw);
        assert_eq!(decode_resource_table(&table).unwrap()["값"],value);
        *table.last_mut().unwrap()=2;
        assert!(check_resource_table_rows(&table).is_ok());
        assert!(decode_resource_table(&table).is_err());
    }
    #[test]
    fn rd670_parameters_exact_raw_roundtrip_and_independent_be_fixture() {
        let values = BTreeMap::from([("a".into(),Fixed64::from_raw_i64(i64::MIN)),
            ("속도".into(),Fixed64::from_raw_i64(i64::MAX))]);
        let bytes = encode_parameter_table(&values).unwrap();
        let mut expected = 2u64.to_be_bytes().to_vec();
        expected.extend_from_slice(&1u32.to_be_bytes()); expected.push(b'a');
        expected.extend_from_slice(&i64::MIN.to_be_bytes());
        expected.extend_from_slice(&6u32.to_be_bytes()); expected.extend_from_slice("속도".as_bytes());
        expected.extend_from_slice(&i64::MAX.to_be_bytes());
        assert_eq!(bytes,expected);
        assert_eq!(decode_parameter_table(&bytes).unwrap(),values);
        for n in 0..bytes.len() { assert!(decode_parameter_table(&bytes[..n]).is_err()); }
        let mut bad=bytes.clone();bad.push(0);assert!(decode_parameter_table(&bad).is_err());
    }
    #[test]
    fn rd670_tables_reject_duplicates_wrong_order_utf8_and_empty_values() {
        fn rows(names:&[&[u8]], resource:bool)->Vec<u8> {
            let mut bytes=(names.len() as u64).to_be_bytes().to_vec();
            for name in names {
                bytes.extend_from_slice(&(name.len() as u32).to_be_bytes()); bytes.extend_from_slice(name);
                if resource {bytes.extend_from_slice(&1u64.to_be_bytes());bytes.push(0);}
                else {bytes.extend_from_slice(&0i64.to_be_bytes());}
            }
            bytes
        }
        for resource in [false,true] {
            let check=if resource {check_resource_table_rows} else {check_parameter_table};
            assert!(check(&rows(&[b"a",b"b"],resource)).is_ok());
            for names in [vec![&b"a"[..],&b"a"[..]],vec![&b"b"[..],&b"a"[..]],vec![&[255][..]]] {
                assert!(check(&rows(&names,resource)).is_err());
            }
            let mut suffix=rows(&[b"a"],resource);suffix.push(0);assert!(check(&suffix).is_err());
        }
        let mut empty=1u64.to_be_bytes().to_vec();empty.extend_from_slice(&1u32.to_be_bytes());empty.push(b'a');
        empty.extend_from_slice(&0u64.to_be_bytes());assert!(check_resource_table_rows(&empty).is_err());
    }
}
