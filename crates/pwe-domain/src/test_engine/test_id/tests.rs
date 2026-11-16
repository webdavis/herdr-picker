use super::*;

fn read(text: &str) -> TestId {
    text.parse().unwrap()
}

#[test]
fn an_id_with_a_case_prints_back_exactly_as_it_was_read() {
    let text = "pytest:tests/test_math.py::TestAdd::test_sum[1-2]";

    assert_eq!(read(text).to_string(), text);
}

#[test]
fn an_id_names_its_runner_file_suite_name_and_case() {
    assert_eq!(
        read("pytest:tests/test_math.py::TestAdd::test_sum[1-2]"),
        TestId {
            runner: "pytest".to_string(),
            file: "tests/test_math.py".to_string(),
            suite: "TestAdd".to_string(),
            name: "test_sum".to_string(),
            case: Some("1-2".to_string()),
        }
    );
}

#[test]
fn a_suite_keeps_every_module_of_its_path() {
    let id = read("cargo:src/lib.rs::capture::tests::a_check_runs_in_its_folder");

    assert_eq!(
        (id.suite.as_str(), id.name.as_str()),
        ("capture::tests", "a_check_runs_in_its_folder")
    );
}

#[test]
fn a_test_outside_any_suite_has_an_empty_suite_and_prints_back_exactly() {
    let id = read("cargo:tests/it.rs::outside");

    assert_eq!(
        (id.suite.as_str(), id.to_string()),
        ("", "cargo:tests/it.rs::outside".to_string())
    );
}

#[test]
fn brackets_in_the_file_name_are_part_of_the_file() {
    let id = read("vitest:app/[id]/page.test.ts::renders");

    assert_eq!((id.file.as_str(), id.case), ("app/[id]/page.test.ts", None));
}

#[test]
fn a_case_may_hold_the_separator() {
    let text = "pytest:tests/test_a.py::test_path[a::b]";

    assert_eq!(
        (read(text).case, read(text).to_string()),
        (Some("a::b".to_string()), text.to_string())
    );
}

#[test]
fn an_id_without_a_runner_is_refused() {
    assert_eq!(
        "tests/it.rs::outside".parse::<TestId>(),
        Err(TestIdError::NoRunner)
    );
}

#[test]
fn an_id_without_a_test_name_is_refused() {
    assert_eq!(
        "cargo:tests/it.rs".parse::<TestId>(),
        Err(TestIdError::NoName)
    );
}

#[test]
fn an_id_with_an_empty_file_is_refused() {
    assert_eq!(
        "cargo:::outside".parse::<TestId>(),
        Err(TestIdError::NoFile)
    );
}
