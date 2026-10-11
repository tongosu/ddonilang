use ddonirang_lang::{parse_v25, canonicalize, normalize, NormalizationLevel};

#[test]
fn rd510_tensor_conversion_and_static_type_canon_roundtrip() {
    for source in [
        "판 <- (차림=차림[차림[1,2],차림[3,4]],원소씨=셈수,모양=(2,2)) 텐서로. 목록 <- (텐서=판) 차림으로.",
        "판:(원소씨=셈수,모양=(2,)) 텐서씨 := (모양=(2,),원소씨=셈수,차림=차림[1,2]) 텐서로.",
        "(판:(원소씨=셈수,모양=()) 텐서씨) 풀이:셈수:셈씨 := { 풀이 <- (텐서=판) 차림으로. }.",
        "빈판 <- (차림=() 차림,원소씨=셈수,모양=(0,3)) 텐서로.",
    ] {
        let mut ast = parse_v25(source,"tensor.ddn").unwrap_or_else(|e| panic!("{source}: {e:?}"));
        canonicalize(&mut ast).unwrap();
        let canon = normalize(&ast, NormalizationLevel::N1);
        let mut again = parse_v25(&canon,"tensor.ddn").unwrap();
        canonicalize(&mut again).unwrap();
        assert_eq!(canon,normalize(&again, NormalizationLevel::N1));
    }
}

#[test]
fn rd510_tensor_rejects_positional_alias_dynamic_duplicate_and_rank1_ambiguity() {
    for source in [
        "판 <- (차림[1],셈수,(1,)) 텐서로.",
        "판 <- (차림=차림[1],원소씨=수,모양=(1,)) 텐서로.",
        "판 <- (차림=차림[1],원소씨=셈수,모양=(n,)) 텐서로.",
        "판 <- (차림=차림[1],원소씨=셈수,모양=(1,),모양=(1,)) 텐서로.",
        "판 <- (차림=차림[1],원소씨=셈수,모양=(1)) 텐서로.",
        "판:(원소씨=셈수,모양=(-1,)) 텐서씨 := 1.",
        "판:(원소씨=셈수,모양=(2,),모양=(2,)) 텐서씨 := 1.",
        "판:(원소씨=셈수,모양=(18446744073709551615,2)) 텐서씨 := 1.",
    ] {
        assert!(parse_v25(source,"bad.ddn").is_err(), "must reject: {source}");
    }
}

#[test]
fn rd388_tensor_multi_axis_source_and_static_ranges_roundtrip() {
    for source in [
        "값 <- 판[1,2].",
        "값 <- 판[1..2,:].",
        "판[:,1..2] <- 새판.",
        "판[1..2,:][0][1] <- 7.",
        "새판 -> 판[:,1..2].",
        "값 <- 판[n,:].",
    ] {
        let mut ast = parse_v25(source,"selection.ddn").unwrap_or_else(|e| panic!("{source}: {e:?}"));
        canonicalize(&mut ast).unwrap();
        let canon = normalize(&ast,NormalizationLevel::N1);
        let mut again = parse_v25(&canon,"selection.ddn").unwrap_or_else(|e| panic!("{canon}: {e:?}"));
        canonicalize(&mut again).unwrap();
        assert_eq!(canon,normalize(&again,NormalizationLevel::N1));
    }
    for source in ["값 <- 판[].","값 <- 판[2..1,:].","값 <- 판[n..2,:].","값 <- 판[0..=1,:].","값 <- 판[-1..2,:].","(차림=차림[1],원소씨=셈수,모양=(1,)) 텐서로[:] <- 2."] {
        assert!(parse_v25(source,"bad-selection.ddn").is_err(),"must reject: {source}");
    }
}
