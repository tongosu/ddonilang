#[derive(Clone, Copy)]
pub struct TermEntry {
    pub code: &'static str,
    pub input: &'static str,
    pub canonical: &'static str,
}

pub const TERM_MAP_VERSION: &str = "tm-1";

pub const FATAL_TERMS: [TermEntry; 12] = [
    TermEntry {
        code: "TERM-FATAL-001",
        input: "자산",
        canonical: "쓸감",
    },
    TermEntry {
        code: "TERM-FATAL-002",
        input: "객체",
        canonical: "임자",
    },
    TermEntry {
        code: "TERM-FATAL-003",
        input: "리소스",
        canonical: "바탕",
    },
    TermEntry {
        code: "TERM-FATAL-004",
        input: "프레임",
        canonical: "마디",
    },
    TermEntry {
        code: "TERM-FATAL-005",
        input: "입력",
        canonical: "샘",
    },
    TermEntry {
        code: "TERM-FATAL-006",
        input: "가드",
        canonical: "지킴이",
    },
    TermEntry {
        code: "TERM-FATAL-007",
        input: "에러",
        canonical: "고장",
    },
    TermEntry {
        code: "TERM-FATAL-008",
        input: "랜덤",
        canonical: "주사위",
    },
    TermEntry {
        code: "TERM-FATAL-009",
        input: "디버그",
        canonical: "거울",
    },
    TermEntry {
        code: "TERM-FATAL-010",
        input: "상태",
        canonical: "누리",
    },
    TermEntry {
        code: "TERM-FATAL-011",
        input: "로그",
        canonical: "진단말",
    },
    TermEntry {
        code: "TERM-FATAL-012",
        input: "패치",
        canonical: "고침",
    },
];

pub const LEGACY_TERMS: [TermEntry; 5] = [
    TermEntry {
        code: "TERM-WARN-001",
        input: "변수",
        canonical: "이름/이름씨",
    },
    TermEntry {
        code: "TERM-WARN-002",
        input: "함수",
        canonical: "움직씨",
    },
    TermEntry {
        code: "TERM-WARN-003",
        input: "클래스",
        canonical: "이름씨",
    },
    TermEntry {
        code: "TERM-WARN-004",
        input: "이벤트",
        canonical: "알림씨",
    },
    TermEntry {
        code: "TERM-WARN-005",
        input: "살림",
        canonical: "바탕",
    },
];

pub const JOSA_ONLY: [&str; 18] = [
    "이", "가", "을", "를", "은", "는", "에", "로", "의", "도", "만", "와", "과", "에서", "에게",
    "께", "부터", "까지",
];

pub const RESERVED_WORDS: [&str; 10] = [
    "마디",
    "샘",
    "임자",
    "누리",
    "지킴이",
    "고장",
    "거울",
    "쓸감",
    "진단말",
    "고침",
];

pub fn find_fatal_term(term: &str) -> Option<TermEntry> {
    FATAL_TERMS
        .iter()
        .copied()
        .find(|entry| entry.input == term)
}

pub fn find_legacy_term(term: &str) -> Option<TermEntry> {
    LEGACY_TERMS
        .iter()
        .copied()
        .find(|entry| entry.input == term)
}

pub fn is_josa_only(term: &str) -> bool {
    JOSA_ONLY.iter().any(|entry| *entry == term)
}

pub fn is_reserved_word(term: &str) -> bool {
    RESERVED_WORDS.iter().any(|entry| *entry == term)
}
