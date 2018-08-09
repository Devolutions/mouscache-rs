use mouscache::*;

#[test]
fn test_memory_hash_functions() {
    let cache = memory();

    assert!(cache.hash_set("test_1", "field_1", "val").unwrap_or(false));
    let pairs = [("field_2", "val2"), ("field_3", "val3")];
    assert!(cache.hash_multiple_set("test_1", &pairs).unwrap_or(false));

    assert_eq!(Some("val2".to_string()), cache.hash_get::<String>("test_1", "field_2").unwrap_or(None));
    let fields = ["field_1", "field_3"];
    assert_eq!(vec![Some("val".to_string()), Some("val3".to_string())], cache.hash_multiple_get("test_1", &fields).unwrap_or(vec![]));

    assert!(cache.hash_delete("test_1", &["field_3"]).unwrap_or(false));
    assert!(!cache.hash_exists("test_1", "field_3").unwrap());
    assert!(cache.hash_set_if_not_exists("test_1", "field_3", "1").unwrap_or(false));
    assert!(!cache.hash_set_if_not_exists("test_1", "field_2", "0").unwrap());

    let keys = cache.hash_keys("test_1").unwrap_or(vec![]);

    assert!(keys.contains(&"field_1".to_string()));
    assert!(keys.contains(&"field_2".to_string()));
    assert!(keys.contains(&"field_3".to_string()));

    let values = cache.hash_values("test_1").unwrap_or(vec![]);

    assert!(values.contains(&"val".to_string()));
    assert!(values.contains(&"val2".to_string()));
    assert!(values.contains(&"1".to_string()));

    assert_eq!(3, cache.hash_len("test_1").unwrap_or(0));
}

#[test]
fn test_memory_set_functions() {
    let cache = memory();

    let set_1_members = ["1", "2", "3", "4", "5"];
    let set_2_members = ["3", "4", "5", "6", "7"];
    let set_3_members = ["5", "6", "7", "8", "9"];
    assert!(cache.set_add("set_1", &set_1_members).unwrap_or(false));
    assert!(cache.set_add("set_2", &set_2_members).unwrap_or(false));
    assert!(cache.set_add("set_3", &set_3_members).unwrap_or(false));

    assert_eq!(5, cache.set_card("set_1").unwrap_or(0));
    assert!(cache.set_rem("set_2", 3).unwrap_or(false));
    assert_eq!(4, cache.set_card("set_2").unwrap_or(0));

    let diff = cache.set_diff(&["set_1", "set_2"]).unwrap_or(vec![]);

    assert!(diff.contains(&"1".to_string()));
    assert!(diff.contains(&"2".to_string()));
    assert!(diff.contains(&"3".to_string()));

    assert_eq!(6, cache.set_unionstore("union", &["set_2", "set_3"]).unwrap_or(0));

    let union = cache.set_members("union").unwrap_or(vec![]);

    assert!(union.contains(&"4".to_string()));
    assert!(union.contains(&"5".to_string()));
    assert!(union.contains(&"6".to_string()));
    assert!(union.contains(&"7".to_string()));
    assert!(union.contains(&"8".to_string()));
    assert!(union.contains(&"9".to_string()));

    assert!(cache.set_move("union", "set_1", 9).unwrap_or(false));

    assert!(cache.set_ismember("set_1", 9).unwrap_or(false));

    let inter = cache.set_inter(&["union", "set_1"]).unwrap_or(vec![]);

    assert!(inter.contains(&"4".to_string()));
    assert!(inter.contains(&"5".to_string()));
}

#[test]
fn test_redis() {
    let cache = redis("127.0.0.1:6379", None, None).expect("dahh");

    let res = cache.set_members("this_set_should_not_exists");
    println!("test 1 {:?}", res);

    let res = cache.hash_get::<String>("this_hashset_does_not_exists", "neither_this_field");
    println!("test 2 {:?}", res);
    let res = cache.hash_set("test_1", "field_1", "val");
    println!("test 3 {:?}", res);
    let res = cache.hash_get::<String>("test_1", "field_1");
    println!("test 4 {:?}", res);
    let res = cache.hash_get::<String>("test_1", "field_2");
    println!("test 5 {:?}", res);

}