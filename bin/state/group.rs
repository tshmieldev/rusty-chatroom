use std::collections::{BTreeSet, HashMap};

use slotmap::{SlotMap, new_key_type};

use crate::state::AccountId;

new_key_type! { pub struct GroupId; }

pub struct Group {
    name: String,
    users: BTreeSet<AccountId>,
}

impl Group {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn contains(&self, user_id: AccountId) -> bool {
        self.users.contains(&user_id)
    }
    pub fn add(&mut self, user_id: AccountId) -> bool {
        self.users.insert(user_id)
    }
    pub fn remove(&mut self, user_id: AccountId) -> bool {
        self.users.remove(&user_id)
    }

    pub fn users(&self) -> impl Iterator<Item = AccountId> + '_ {
        self.users.iter().copied()
    }
}

#[derive(Default)]
pub struct Groups {
    store: SlotMap<GroupId, Group>,
    by_name: HashMap<String, GroupId>,
}

impl Groups {
    pub fn create(&mut self, name: &str) -> Option<GroupId> {
        if self.by_name.contains_key(name) {
            return None;
        }
        let id = self.store.insert(Group {
            name: String::from(name),
            users: BTreeSet::new(),
        });
        self.by_name.insert(String::from(name), id);
        Some(id)
    }
    pub fn find(&self, name: &str) -> Option<GroupId> {
        self.by_name.get(name).copied()
    }
    pub fn get(&self, id: GroupId) -> Option<&Group> {
        self.store.get(id)
    }
    pub fn get_mut(&mut self, id: GroupId) -> Option<&mut Group> {
        self.store.get_mut(id)
    }
}
