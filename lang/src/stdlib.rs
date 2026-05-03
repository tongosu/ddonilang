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
        "흐름만들기" => "흐름.만들기",
        "흐름넣기" => "흐름.밀어넣기",
        "흐름추가" => "흐름.밀어넣기",
        "흐름값들" => "흐름.차림",
        "흐름최근" => "흐름.최근값",
        "흐름길이" => "흐름.길이",
        "흐름용량" => "흐름.용량",
        "흐름비우기" => "흐름.비우기",
        "흐름잘라보기" => "흐름.잘라보기",
        "흐름최근N" => "흐름.잘라보기",
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
        // 수 타입 생성자 별칭
        "fixed64" => "셈수",
        "sim_num" => "셈수",
        "셈수" => "셈수",
        "셈수2" => "셈수",
        "셈수4" => "셈수",
        "셈수8" => "셈수",
        "int" => "바른수",
        "int64" => "바른수",
        "정수" => "바른수",
        "바른수1" => "바른수",
        "바른수2" => "바른수",
        "바른수4" => "바른수",
        "바른수8" => "바른수",
        "bigint" => "큰바른수",
        "big_int" => "큰바른수",
        "유리수" => "나눔수",
        "rational" => "나눔수",
        "ratio" => "나눔수",
        "frac" => "나눔수",
        "factor" => "곱수",
        "factorized" => "곱수",
        "primepow" => "곱수",
        "쓸감" => "자원",
        "겹차림.형상" => "텐서.형상",
        "겹차림.자료" => "텐서.자료",
        "겹차림.배치" => "텐서.배치",
        "겹차림.값" => "텐서.값",
        "겹차림.바꾼값" => "텐서.바꾼값",
        "너머.시각.지금" => "열림.시각.지금",
        "너머.파일.읽기" => "열림.파일.읽기",
        "너머.난수" => "열림.난수",
        "너머.난수.하나" => "열림.난수.하나",
        "너머.난수.뽑기" => "열림.난수.뽑기",
        "너머.네트워크.요청" => "열림.네트워크.요청",
        "너머.호스트FFI.호출" => "열림.호스트FFI.호출",
        "너머.GPU.실행" => "열림.GPU.실행",
        "너머.풀이.확인" => "열림.풀이.확인",
        "효과.시각.지금" => "열림.시각.지금",
        "효과.파일.읽기" => "열림.파일.읽기",
        "효과.난수" => "열림.난수",
        "효과.난수.하나" => "열림.난수.하나",
        "효과.난수.뽑기" => "열림.난수.뽑기",
        "효과.네트워크.요청" => "열림.네트워크.요청",
        "효과.호스트FFI.호출" => "열림.호스트FFI.호출",
        "효과.GPU.실행" => "열림.GPU.실행",
        "효과.풀이.확인" => "열림.풀이.확인",
        "바깥.시각.지금" => "열림.시각.지금",
        "바깥.파일.읽기" => "열림.파일.읽기",
        "바깥.난수" => "열림.난수",
        "바깥.난수.하나" => "열림.난수.하나",
        "바깥.난수.뽑기" => "열림.난수.뽑기",
        "바깥.네트워크.요청" => "열림.네트워크.요청",
        "바깥.호스트FFI.호출" => "열림.호스트FFI.호출",
        "바깥.GPU.실행" => "열림.GPU.실행",
        "바깥.풀이.확인" => "열림.풀이.확인",
        _ => name,
    }
}

pub fn canonicalize_type_alias(name: &str) -> &str {
    let canonical = canonicalize_stdlib_alias(name.trim());
    match canonical {
        "논" | "bool" | "boolean" => "참거짓",
        "string" => "글",
        "목록" | "list" => "차림",
        "모둠" | "set" => "모음",
        "그림표" | "map" => "짝맞춤",
        "값꾸러미" | "pack" => "묶음",
        "none" | "non" => "없음",
        other => other,
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
        FunctionSig {
            name: "글바꾸기!",
            params: &["글", "인덱스", "새글"],
            ret: "글",
        },
    ]
}

pub fn regex_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "정규맞추기",
            params: &["대상", "패턴"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "정규찾기",
            params: &["대상", "패턴"],
            ret: "글?",
        },
        FunctionSig {
            name: "정규캡처하기",
            params: &["대상", "패턴"],
            ret: "차림<글>",
        },
        FunctionSig {
            name: "정규이름캡처하기",
            params: &["대상", "패턴"],
            ret: "짝맞춤<글,글>",
        },
        FunctionSig {
            name: "정규바꾸기",
            params: &["대상", "패턴", "바꿈"],
            ret: "글",
        },
        FunctionSig {
            name: "정규나누기",
            params: &["대상", "패턴"],
            ret: "차림<글>",
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
        FunctionSig {
            name: "찾기?",
            params: &["짝맞춤", "열쇠"],
            ret: "값?",
        },
        FunctionSig {
            name: "짝맞춤.값",
            params: &["짝맞춤", "열쇠"],
            ret: "값?",
        },
        FunctionSig {
            name: "짝맞춤.필수값",
            params: &["짝맞춤", "열쇠"],
            ret: "값",
        },
        FunctionSig {
            name: "짝맞춤.바꾼값",
            params: &["짝맞춤", "열쇠", "값"],
            ret: "짝맞춤<K,V>",
        },
    ]
}

pub fn stream_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "흐름.만들기",
            params: &["용량", "초기값차림?"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름.밀어넣기",
            params: &["흐름", "값"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름.차림",
            params: &["흐름"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "흐름.최근값",
            params: &["흐름"],
            ret: "T?",
        },
        FunctionSig {
            name: "흐름.길이",
            params: &["흐름"],
            ret: "정수",
        },
        FunctionSig {
            name: "흐름.용량",
            params: &["흐름"],
            ret: "정수",
        },
        FunctionSig {
            name: "흐름.비우기",
            params: &["흐름"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름.잘라보기",
            params: &["흐름", "개수"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "흐름만들기",
            params: &["용량", "초기값차림?"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름넣기",
            params: &["흐름", "값"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름추가",
            params: &["흐름", "값"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름값들",
            params: &["흐름"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "흐름최근",
            params: &["흐름"],
            ret: "T?",
        },
        FunctionSig {
            name: "흐름길이",
            params: &["흐름"],
            ret: "정수",
        },
        FunctionSig {
            name: "흐름용량",
            params: &["흐름"],
            ret: "정수",
        },
        FunctionSig {
            name: "흐름비우기",
            params: &["흐름"],
            ret: "흐름",
        },
        FunctionSig {
            name: "흐름잘라보기",
            params: &["흐름", "개수"],
            ret: "차림<T>",
        },
        FunctionSig {
            name: "흐름최근N",
            params: &["흐름", "개수"],
            ret: "차림<T>",
        },
    ]
}

pub fn input_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "입력키",
            params: &[],
            ret: "글",
        },
        FunctionSig {
            name: "입력키?",
            params: &[],
            ret: "글?",
        },
        FunctionSig {
            name: "입력키!",
            params: &[],
            ret: "글",
        },
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
        FunctionSig {
            name: "주사위씨.만들기",
            params: &["시앗"],
            ret: "주사위씨",
        },
        FunctionSig {
            name: "주사위씨.뽑기",
            params: &["주사위씨", "최소", "최대"],
            ret: "정수",
        },
        FunctionSig {
            name: "주사위씨.실수뽑기",
            params: &["주사위씨", "최소", "최대"],
            ret: "수",
        },
        FunctionSig {
            name: "주사위씨.골라뽑기",
            params: &["주사위씨", "후보"],
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
        FunctionSig {
            name: "적분.오일러",
            params: &["값", "변화율", "dt"],
            ret: "값",
        },
        FunctionSig {
            name: "적분.반암시적오일러",
            params: &["위치", "속도", "가속도", "dt"],
            ret: "(위치, 속도)",
        },
        FunctionSig {
            name: "보간.선형",
            params: &["시작", "끝", "t(0..1)"],
            ret: "값",
        },
        FunctionSig {
            name: "보간.계단",
            params: &["시작", "끝", "t", "문턱"],
            ret: "값",
        },
        FunctionSig {
            name: "필터.이동평균",
            params: &["창", "새값"],
            ret: "(창, 평균)",
        },
        FunctionSig {
            name: "필터.지수평활",
            params: &["이전", "새값", "alpha(0..1)"],
            ret: "값",
        },
        FunctionSig {
            name: "미분.중앙차분",
            params: &["수식값", "변수", "점", "스텝"],
            ret: "(근사값, 오차추정, 사용한방법)",
        },
        FunctionSig {
            name: "적분.사다리꼴",
            params: &["수식값", "변수", "시작", "끝", "스텝"],
            ret: "(근사값, 오차추정, 사용한방법)",
        },
    ]
}

pub fn pack_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "묶음값",
            params: &["묶음", "열쇠"],
            ret: "값?",
        },
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
            name: "정리하기",
            params: &["식"],
            ret: "식",
        },
        FunctionSig {
            name: "전개하기",
            params: &["식"],
            ret: "식",
        },
        FunctionSig {
            name: "인수분해하기",
            params: &["식"],
            ret: "식",
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
        FunctionSig {
            name: "동치인가",
            params: &["왼쪽", "오른쪽"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "잇기",
            params: &["왼쪽", "오른쪽"],
            ret: "묶음",
        },
        FunctionSig {
            name: "증명하기",
            params: &["증명요청"],
            ret: "묶음",
        },
        FunctionSig {
            name: "방정식풀기",
            params: &["관계"],
            ret: "묶음",
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

pub fn age1_runtime_function_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "살피기",
            params: &["세움", "값들"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "마당다시",
            params: &[],
            ret: "없음",
        },
        FunctionSig {
            name: "판다시",
            params: &[],
            ret: "없음",
        },
        FunctionSig {
            name: "누리다시",
            params: &[],
            ret: "없음",
        },
        FunctionSig {
            name: "보개다시",
            params: &[],
            ret: "없음",
        },
        FunctionSig {
            name: "모두다시",
            params: &[],
            ret: "없음",
        },
        FunctionSig {
            name: "시작하기",
            params: &["대상?"],
            ret: "없음",
        },
        FunctionSig {
            name: "넘어가기",
            params: &["대상?"],
            ret: "없음",
        },
        FunctionSig {
            name: "불러오기",
            params: &["대상?"],
            ret: "없음",
        },
        FunctionSig {
            name: "처음으로",
            params: &["상태머신"],
            ret: "글",
        },
        FunctionSig {
            name: "지금상태",
            params: &["상태머신", "상태"],
            ret: "글",
        },
        FunctionSig {
            name: "다음으로",
            params: &["상태머신", "상태"],
            ret: "글",
        },
        FunctionSig {
            name: "열림.풀이.확인",
            params: &["질의"],
            ret: "참거짓",
        },
        FunctionSig {
            name: "반례찾기",
            params: &["질의"],
            ret: "묶음",
        },
        FunctionSig {
            name: "해찾기",
            params: &["질의"],
            ret: "묶음",
        },
    ]
}

pub fn numeric_type_constructor_sigs() -> Vec<FunctionSig> {
    vec![
        FunctionSig {
            name: "수",
            params: &["값"],
            ret: "수",
        },
        FunctionSig {
            name: "바른수",
            params: &["값"],
            ret: "바른수",
        },
        FunctionSig {
            name: "큰바른수",
            params: &["값"],
            ret: "큰바른수",
        },
        FunctionSig {
            name: "나눔수",
            params: &["분자", "분모"],
            ret: "나눔수",
        },
        FunctionSig {
            name: "곱수",
            params: &["값"],
            ret: "곱수",
        },
    ]
}

pub fn minimal_stdlib_sigs() -> Vec<FunctionSig> {
    let mut out = Vec::new();
    out.extend(string_function_sigs());
    out.extend(regex_function_sigs());
    out.extend(list_function_sigs());
    out.extend(container_function_sigs());
    out.extend(stream_function_sigs());
    out.extend(input_function_sigs());
    out.extend(resource_function_sigs());
    out.extend(random_function_sigs());
    out.extend(math_function_sigs());
    out.extend(pack_function_sigs());
    out.extend(transform_function_sigs());
    out.extend(logic_function_sigs());
    out.extend(age1_runtime_function_sigs());
    out.extend(numeric_type_constructor_sigs());
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
        assert!(sigs.iter().any(|s| s.name == "흐름.만들기"));
        assert!(sigs.iter().any(|s| s.name == "흐름.밀어넣기"));
        assert!(sigs.iter().any(|s| s.name == "흐름.차림"));
        assert!(sigs.iter().any(|s| s.name == "흐름.비우기"));
        assert!(sigs.iter().any(|s| s.name == "흐름.잘라보기"));
        assert!(sigs.iter().any(|s| s.name == "첫번째"));
        assert!(sigs.iter().any(|s| s.name == "정렬"));
        assert!(sigs.iter().any(|s| s.name == "포함하나"));
        assert!(sigs.iter().any(|s| s.name == "숫자로"));
        assert!(sigs.iter().any(|s| s.name == "지니"));
        assert!(sigs.iter().any(|s| s.name == "분위수"));
        assert!(sigs.iter().any(|s| s.name == "글바꾸기!"));
        assert!(sigs.iter().any(|s| s.name == "묶음값"));
        assert!(sigs.iter().any(|s| s.name == "키목록"));
        assert!(sigs.iter().any(|s| s.name == "주사위씨.만들기"));
        assert!(sigs.iter().any(|s| s.name == "주사위씨.뽑기"));
        assert!(sigs.iter().any(|s| s.name == "마당다시"));
        assert!(sigs.iter().any(|s| s.name == "판다시"));
        assert!(sigs.iter().any(|s| s.name == "누리다시"));
        assert!(sigs.iter().any(|s| s.name == "보개다시"));
        assert!(sigs.iter().any(|s| s.name == "모두다시"));
        assert!(sigs.iter().any(|s| s.name == "시작하기"));
        assert!(sigs.iter().any(|s| s.name == "넘어가기"));
        assert!(sigs.iter().any(|s| s.name == "불러오기"));
        assert!(sigs.iter().any(|s| s.name == "수"));
        assert!(sigs.iter().any(|s| s.name == "바른수"));
        assert!(sigs.iter().any(|s| s.name == "큰바른수"));
        assert!(sigs.iter().any(|s| s.name == "나눔수"));
        assert!(sigs.iter().any(|s| s.name == "곱수"));
    }

    #[test]
    fn canonicalize_stdlib_aliases_map_to_single_canonical_names() {
        assert_eq!(canonicalize_stdlib_alias("길이세기"), "길이");
        assert_eq!(canonicalize_stdlib_alias("값뽑기"), "차림.값");
        assert_eq!(canonicalize_stdlib_alias("번째"), "차림.값");
        assert_eq!(canonicalize_stdlib_alias("흐름만들기"), "흐름.만들기");
        assert_eq!(canonicalize_stdlib_alias("흐름추가"), "흐름.밀어넣기");
        assert_eq!(canonicalize_stdlib_alias("흐름값들"), "흐름.차림");
        assert_eq!(canonicalize_stdlib_alias("흐름최근"), "흐름.최근값");
        assert_eq!(canonicalize_stdlib_alias("흐름비우기"), "흐름.비우기");
        assert_eq!(canonicalize_stdlib_alias("흐름잘라보기"), "흐름.잘라보기");
        assert_eq!(canonicalize_stdlib_alias("흐름최근N"), "흐름.잘라보기");
        assert_eq!(canonicalize_stdlib_alias("절댓값"), "abs");
        assert_eq!(canonicalize_stdlib_alias("최솟값"), "min");
        assert_eq!(canonicalize_stdlib_alias("지니계수"), "지니");
        assert_eq!(canonicalize_stdlib_alias("백분위수"), "분위수");
        assert_eq!(canonicalize_stdlib_alias("fixed64"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("sim_num"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("셈수"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("셈수2"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("셈수4"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("셈수8"), "셈수");
        assert_eq!(canonicalize_stdlib_alias("int"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("int64"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("정수"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("바른수1"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("바른수2"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("바른수4"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("바른수8"), "바른수");
        assert_eq!(canonicalize_stdlib_alias("bigint"), "큰바른수");
        assert_eq!(canonicalize_stdlib_alias("big_int"), "큰바른수");
        assert_eq!(canonicalize_stdlib_alias("유리수"), "나눔수");
        assert_eq!(canonicalize_stdlib_alias("rational"), "나눔수");
        assert_eq!(canonicalize_stdlib_alias("ratio"), "나눔수");
        assert_eq!(canonicalize_stdlib_alias("frac"), "나눔수");
        assert_eq!(canonicalize_stdlib_alias("factor"), "곱수");
        assert_eq!(canonicalize_stdlib_alias("factorized"), "곱수");
        assert_eq!(canonicalize_stdlib_alias("primepow"), "곱수");
        assert_eq!(canonicalize_stdlib_alias("쓸감"), "자원");
        assert_eq!(canonicalize_stdlib_alias("겹차림.형상"), "텐서.형상");
        assert_eq!(canonicalize_stdlib_alias("겹차림.자료"), "텐서.자료");
        assert_eq!(canonicalize_stdlib_alias("겹차림.배치"), "텐서.배치");
        assert_eq!(canonicalize_stdlib_alias("겹차림.값"), "텐서.값");
        assert_eq!(canonicalize_stdlib_alias("겹차림.바꾼값"), "텐서.바꾼값");
        assert_eq!(
            canonicalize_stdlib_alias("너머.시각.지금"),
            "열림.시각.지금"
        );
        assert_eq!(
            canonicalize_stdlib_alias("효과.호스트FFI.호출"),
            "열림.호스트FFI.호출"
        );
        assert_eq!(canonicalize_stdlib_alias("바깥.GPU.실행"), "열림.GPU.실행");
        assert_eq!(
            canonicalize_stdlib_alias("너머.풀이.확인"),
            "열림.풀이.확인"
        );
        assert_eq!(canonicalize_stdlib_alias("차림.값"), "차림.값");
    }

    #[test]
    fn canonicalize_type_aliases_map_to_single_canonical_names() {
        assert_eq!(canonicalize_type_alias(" bool "), "참거짓");
        assert_eq!(canonicalize_type_alias("논"), "참거짓");
        assert_eq!(canonicalize_type_alias("string"), "글");
        assert_eq!(canonicalize_type_alias("none"), "없음");
        assert_eq!(canonicalize_type_alias("non"), "없음");
        assert_eq!(canonicalize_type_alias("list"), "차림");
        assert_eq!(canonicalize_type_alias("목록"), "차림");
        assert_eq!(canonicalize_type_alias("set"), "모음");
        assert_eq!(canonicalize_type_alias("모둠"), "모음");
        assert_eq!(canonicalize_type_alias("map"), "짝맞춤");
        assert_eq!(canonicalize_type_alias("그림표"), "짝맞춤");
        assert_eq!(canonicalize_type_alias("pack"), "묶음");
        assert_eq!(canonicalize_type_alias("값꾸러미"), "묶음");
        assert_eq!(canonicalize_type_alias("fixed64"), "셈수");
        assert_eq!(canonicalize_type_alias("rational"), "나눔수");
        assert_eq!(canonicalize_type_alias("factorized"), "곱수");
    }
}
