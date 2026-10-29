use ddonirang_core::Fixed64;

fn main() {
    let actual = Fixed64::determinism_vector_v1();
    let expected = Fixed64::DETERMINISM_VECTOR_V1_EXPECTED;

    let mut hasher = blake3::Hasher::new();
    for value in actual {
        hasher.update(&value.to_le_bytes());
    }

    let values = actual
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let expected_values = expected
        .iter()
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let status = if actual == expected { "pass" } else { "fail" };

    println!("schema=ddn.fixed64.determinism_vector.v1");
    println!("status={status}");
    println!("blake3={}", hasher.finalize().to_hex());
    println!("raw_i64={values}");
    println!("expected_raw_i64={expected_values}");

    if status != "pass" {
        std::process::exit(2);
    }
}
