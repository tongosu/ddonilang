#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: &'static str,
    pub params: &'static [&'static str],
    pub ret: &'static str,
}

pub fn canonicalize_stdlib_alias(name: &str) -> &str {
    match name {
        "갈라놓기" => "자르기",
        "이어붙이기" => "붙이기",
        "길이세기" => "길이",
        "값뽑기" => "차림.값",
        "번째" => "차림.값",
        "올림" => "천장",
        "내림" => "바닥",
        "절댓값" => "abs",
        "제곱근" => "sqrt",
        "제곱" => "powi",
        "거듭제곱" => "powi",
        "최댓값" => "max",
        "최솟값" => "min",
        "지니계수" => "지니",
        "백분위수" => "분위수",
        "쓸감" => "자원",
        "겹차림.형상" => "텐서.형상",
        "겹차림.자료" => "텐서.자료",
        "겹차림.배치" => "텐서.배치",
        "겹차림.값" => "텐서.값",
        "겹차림.바꾼값" => "텐서.바꾼값",
        _ => name,
    }
}

pub fn string_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "길이",
            params: &["글"],
            ret: "정수",
        },
        FunctionSig {
            name: "대문자로바꾸기",
            params: &["글"],
            ret: "글",
        },
        FunctionSig {
            name: "소문자로바꾸기",
            params: &["글"],
            ret: "글",
        },
        FunctionSig {
            name: "다듬기",
            params: &["글"],
            ret: "글",
        },
        FunctionSig {
            name: "되풀이하기",
            params: &["글", "횟수"],
            ret: "글",
        },
        FunctionSig {
            name: "합치기",
            params: &["앞", "뒤"],
            ret: "글",
        },
        FunctionSig {
            name: "포함하나",
            params: &["글", "패턴"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "시작하나",
            params: &["글", "패턴"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "끝나나",
            params: &["글", "패턴"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "자르기",
            params: &["글", "구분"],
            ret: "차림<글>",
        },
        FunctionSig {
            name: "붙이기",
            params: &["차림<글>", "구분"],
            ret: "글",
        },
        FunctionSig {
            name: "글자뽑기",
            params: &["글", "번째"],
            ret: "글?",
        },
        FunctionSig {
            name: "숫자로",
            params: &["글"],
            ret: "수?",
        },
        FunctionSig {
            name: "글로",
            params: &["값"],
            ret: "글",
        },
        FunctionSig {
            name: "찾기",
            params: &["글", "찾을글"],
            ret: "정수",
        },
        FunctionSig {
            name: "바꾸기",
            params: &["글", "원본", "바꿀것"],
            ret: "글",
        },
        FunctionSig {
            name: "글바꾸기",
            params: &["글", "인덱스", "새글"],
            ret: "글",
        },
    ]
}

pub fn list_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "목록",
            params: &["요소들..."],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "차림",
            params: &["요소들..."],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "차림.값",
            params: &["대상", "i"],
            ret: "T?",
        },
        FunctionSig {
            name: "차림.바꾼값",
            params: &["대상", "i", "값"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "토막내기",
            params: &["차림", "시작", "끝"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "들어있나",
            params: &["차림", "값"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "찾아보기",
            params: &["차림", "값"],
            ret: "정수",
        },
        FunctionSig {
            name: "번째",
            params: &["차림", "인덱스"],
            ret: "T?",
        },
        FunctionSig {
            name: "길이",
            params: &["차림"],
            ret: "정수",
        },
        FunctionSig {
            name: "범위",
            params: &["시작", "끝", "간격?"],
            ret: "차림<수>",
        },
        FunctionSig {
            name: "표준.범위",
            params: &["시작", "끝", "끝포함"],
            ret: "차림<수>",
        },
        FunctionSig {
            name: "첫번째",
            params: &["차림"],
            ret: "T?",
        },
        FunctionSig {
            name: "마지막",
            params: &["차림"],
            ret: "T?",
        },
        FunctionSig {
            name: "뒤집기",
            params: &["차림"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "추가",
            params: &["차림", "요소"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "제거",
            params: &["차림", "인덱스"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "붙이기",
            params: &["차림<T>", "차림<T>"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "펼치기",
            params: &["차림<차림<T>>"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "정렬",
            params: &["차림", "기준"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "거르기",
            params: &["차림", "조건"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "변환",
            params: &["차림", "함수"],
            ret: "차림<U>",
        },
        FunctionSig {
            name: "바꾸기",
            params: &["차림", "함수"],
            ret: "차림<U>",
        },
        FunctionSig {
            name: "각각돌며",
            params: &["차림", "함수"],
            ret: "공",
        },
        FunctionSig {
            name: "합치기",
            params: &["차림", "초기", "함수"],
            ret: "U",
        },
    ]
}

pub fn container_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "모음",
            params: &["요소들..."],
            ret: "모음<T>",
        },
        FunctionSig {
            name: "짝맞춤",
            params: &["열쇠", "값", "..."],
            ret: "짝맞춤<K,V>",
        },
    ]
}

pub fn input_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "눌렸나",
            params: &["키"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "막눌렸나",
            params: &["키"],
            ret: "참거짓",
        },
    ]
}

pub fn resource_function_sigs() -> Vec<FunctionSig> {
    vec![FunctionSig {
        name: "자원",
        params: &["글"],
        ret: "자원핸들",
    }]
}

pub fn random_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "무작위",
            params: &[],
            ret: "수",
        },
        FunctionSig {
            name: "무작위정수",
            params: &["최소", "최대"],
            ret: "정수",
        },
        FunctionSig {
            name: "무작위선택",
            params: &["차림"],
            ret: "T",
        },
    ]
}

pub fn math_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "바닥",
            params: &["수"],
            ret: "수",
        },
        FunctionSig {
            name: "천장",
            params: &["수"],
            ret: "수",
        },
        FunctionSig {
            name: "반올림",
            params: &["수"],
            ret: "수",
        },
        FunctionSig {
            name: "합계",
            params: &["차림"],
            ret: "수",
        },
        FunctionSig {
            name: "평균",
            params: &["차림"],
            ret: "수?",
        },
        FunctionSig {
            name: "지니",
            params: &["차림<수>"],
            ret: "수?",
        },
        FunctionSig {
            name: "분위수",
            params: &["차림<수>", "p(0..1)", "mode?"],
            ret: "수?",
        },
    ]
}

pub fn pack_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "키목록",
            params: &["묶음"],
            ret: "차림<글>",
        },
        FunctionSig {
            name: "값목록",
            params: &["묶음"],
            ret: "차림<값>",
        },
        FunctionSig {
            name: "쌍목록",
            params: &["묶음"],
            ret: "차림<차림>",
        },
        FunctionSig {
            name: "합치기",
            params: &["묶음", "묶음"],
            ret: "묶음",
        },
    ]
}

pub fn transform_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "채우기",
            params: &["무늬", "주입"],
            ret: "글",
        },
        FunctionSig {
            name: "풀기",
            params: &["식", "주입"],
            ret: "수",
        },
        FunctionSig {
            name: "미분하기",
            params: &["식", "옵션"],
            ret: "식",
        },
        FunctionSig {
            name: "적분하기",
            params: &["식", "옵션"],
            ret: "식",
        },
    ]
}

pub fn logic_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "그리고",
            params: &["왼쪽", "오른쪽"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "또는",
            params: &["왼쪽", "오른쪽"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "아님",
            params: &["값"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "아니다",
            params: &["값"],
            ret: "참거짓",
        },
    ]
}

pub fn minimal_stdlib_sigs() -> Vec<FunctionSig> {
    let mut out = Vec::new();
    out.extend(string_function_sigs());
    out.extend(list_function_sigs());
    out.extend(container_function_sigs());
    out.extend(input_function_sigs());
    out.extend(resource_function_sigs());
    out.extend(random_function_sigs());
    out.extend(math_function_sigs());
    out.extend(pack_function_sigs());
    out.extend(transform_function_sigs());
    out.extend(logic_function_sigs());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stdlib_includes_list_and_string() {
        let sigs = minimal_stdlib_sigs();
        assert!(sigs.iter().any(|s| s.name == "목록"));
        assert!(sigs.iter().any(|s| s.name == "차림"));
        assert!(sigs.iter().any(|s| s.name == "차림.값"));
        assert!(sigs.iter().any(|s| s.name == "첫번째"));
        assert!(sigs.iter().any(|s| s.name == "정렬"));
        assert!(sigs.iter().any(|s| s.name == "포함하나"));
        assert!(sigs.iter().any(|s| s.name == "숫자로"));
        assert!(sigs.iter().any(|s| s.name == "지니"));
        assert!(sigs.iter().any(|s| s.name == "분위수"));
    }

    #[test]
    fn canonicalize_stdlib_aliases_map_to_single_canonical_names() {
        assert_eq!(canonicalize_stdlib_alias("길이세기"), "길이");
        assert_eq!(canonicalize_stdlib_alias("값뽑기"), "차림.값");
        assert_eq!(canonicalize_stdlib_alias("번째"), "차림.값");
        assert_eq!(canonicalize_stdlib_alias("절댓값"), "abs");
        assert_eq!(canonicalize_stdlib_alias("최솟값"), "min");
        assert_eq!(canonicalize_stdlib_alias("지니계수"), "지니");
        assert_eq!(canonicalize_stdlib_alias("백분위수"), "분위수");
        assert_eq!(canonicalize_stdlib_alias("쓸감"), "자원");
        assert_eq!(canonicalize_stdlib_alias("겹차림.형상"), "텐서.형상");
        assert_eq!(canonicalize_stdlib_alias("겹차림.자료"), "텐서.자료");
        assert_eq!(canonicalize_stdlib_alias("겹차림.배치"), "텐서.배치");
        assert_eq!(canonicalize_stdlib_alias("겹차림.값"), "텐서.값");
        assert_eq!(canonicalize_stdlib_alias("겹차림.바꾼값"), "텐서.바꾼값");
        assert_eq!(canonicalize_stdlib_alias("차림.값"), "차림.값");
    }
}
