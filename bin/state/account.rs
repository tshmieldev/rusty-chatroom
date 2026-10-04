use std::collections::{BTreeSet, HashMap};

use slotmap::{SlotMap, new_key_type};

use crate::state::GroupId;

new_key_type! { pub struct AccountId; }

pub struct Account {
    username: String,
    password: String,
    groups: BTreeSet<GroupId>,
    pub history: Vec<String>,
}

impl Account {
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn check_password(&self, password: &str) -> bool {
        self.password == password
    }
    pub fn groups(&self) -> &BTreeSet<GroupId> {
        &self.groups
    }
    pub(in crate::state) fn join_group(&mut self, group_id: GroupId) -> bool {
        self.groups.insert(group_id)
    }
    pub(in crate::state) fn exit_group(&mut self, group_id: GroupId) -> bool {
        self.groups.remove(&group_id)
    }
}

#[derive(Default)]
pub struct Accounts {
    store: SlotMap<AccountId, Account>,
    by_name: HashMap<String, AccountId>,
}

impl Accounts {
    pub fn register(&mut self, username: &str, password: &str) -> Option<AccountId> {
        if self.by_name.contains_key(username) {
            return None;
        }
        let id = self.store.insert(Account {
            username: String::from(username),
            password: String::from(password),
            groups: BTreeSet::new(),
            history: Vec::new(),
        });
        self.by_name.insert(String::from(username), id);
        Some(id)
    }
    pub fn find(&self, username: &str) -> Option<AccountId> {
        self.by_name.get(username).copied()
    }
    pub fn get(&self, id: AccountId) -> Option<&Account> {
        self.store.get(id)
    }
    pub fn get_mut(&mut self, id: AccountId) -> Option<&mut Account> {
        self.store.get_mut(id)
    }
    pub fn delete_account(&mut self, id: AccountId) -> bool {
        if let Some(u) = self.store.remove(id) {
            self.by_name.remove(u.username());
            return true;
        }
        false
    }
}
