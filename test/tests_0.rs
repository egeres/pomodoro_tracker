
#[cfg(test)]
mod tests_0 {

    use super::*;
    use app::load_json;

    #[test]
    fn test_load_json() {
        load_json("data/my_json_test.json")
    }

}
