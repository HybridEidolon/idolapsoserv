use super::Sqlite;
use super::super::Backend;
use ::db::account::Account;

#[test]
fn create_account() {
    let s = Sqlite::new(":memory:").unwrap();

    let mut a = Account::new("testuser", "testpassword", "pourthesalt");

    s.put_account(&mut a).unwrap();
}

#[test]
fn fetch_account_by_id() {
    let s = Sqlite::new(":memory:").unwrap();

    let mut a = Account::new("testuser", "testpassword", "pourthesalt");

    s.put_account(&mut a).unwrap();

    // Backend should have given this account a unique ID
    let id = a.id.unwrap();

    // Backend should have the account by that ID
    let a = s.get_account_by_id(id).unwrap().unwrap();

    assert_eq!(a.username, "testuser");
}

#[test]
fn fetch_account_by_username() {
    let s = Sqlite::new(":memory:").unwrap();

    let mut a = Account::new("testuser", "testpassword", "pourthesalt");

    s.put_account(&mut a).unwrap();

    let id = a.id.unwrap();

    let a = s.get_account_by_username("testuser").unwrap().unwrap();

    assert_eq!(a.id, Some(id));
}
