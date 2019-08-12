use super::*;
use crate::change_set::{ChangeSet, ChangeSetMut, ChangeOp, FieldChanges};
use crate::access_path::{AccessPath, Accesses};
use types::language_storage::ResourceKey;
use types::account_address::AccountAddress;
use types::account_config::account_struct_tag;


#[test]
fn test_change_op_merge_plus(){
    let mut change_op0 = ChangeOp::Plus(100);
    let change_op1 = ChangeOp::Plus(100);
    let change_op2 = ChangeOp::merge(&change_op0, &change_op1).unwrap();
    assert_eq!(change_op2.as_plus().unwrap(), 200);

    change_op0.merge_with(&change_op1).unwrap();
    assert_eq!(change_op0.as_plus().unwrap(), 200);
}

#[test]
fn test_change_set_merge(){
    let account_address = AccountAddress::random();
    let resource_key = ResourceKey::new(account_address, account_struct_tag());
    let mut accesses = Accesses::empty();
    //balance filed
    accesses.add_index_to_back(1);
    //coin value filed.
    accesses.add_index_to_back(0);
    let access_path = AccessPath::resource_access_path(&resource_key);

    let change_set0 = ChangeSetMut::new(vec![(access_path.clone(), FieldChanges::new(vec![(accesses.clone(),ChangeOp::Plus(100))]))]);
    let change_set1 = ChangeSetMut::new(vec![(access_path.clone(), FieldChanges::new(vec![(accesses.clone(),ChangeOp::Plus(100))]))]);
    let change_set2 = ChangeSetMut::merge(&change_set0, &change_set1).unwrap();
    let (access_path, field_changes) = &change_set2[0];
    println!("field_changes:{:#?}", field_changes);
    if let (_, ChangeOp::Plus(value)) = field_changes[0] {
        assert_eq!(value, 200);
        println!("merged value:{}", value)
    }else{
        debug_assert!(true, "unexpect change result.")
    }
}