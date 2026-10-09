use monocle_bls_compat::{AGAVE_REVISION, CASE_NAMES, PROFILE_ID, case_result};
use serde_json::json;

fn main() {
    let cases: Vec<_> = CASE_NAMES
        .iter()
        .enumerate()
        .map(|(index, name)| json!({"name":name,"result":case_result(index as u32)}))
        .collect();
    println!(
        "{}",
        json!({"profile_id":PROFILE_ID,"agave_revision":AGAVE_REVISION,"cases":cases})
    );
    let expected = [1, 0, 0, 0, 0, 0];
    if (0..6).any(|index| cases[index]["result"] != expected[index]) {
        std::process::exit(1);
    }
}
