use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap, VecDeque},
};

use slotmap::{SlotMap, new_key_type};

use crate::state::AccountId;

new_key_type! { pub struct GroupId; }

pub type History = VecDeque<HistoryEntry>;

pub struct HistoryEntry {
    pub user: AccountId,
    pub message: Cow<'static, str>,
}

pub struct Group {
    name: String,
    users: BTreeSet<AccountId>,
    history: History,
}

impl Group {
    const MAX_HISTORY_LEN: usize = 32;

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
    pub fn get_history(&self) -> &History {
        &self.history
    }
    pub fn add_history_entry(&mut self, user_id: AccountId, message: &str) {
        self.history.push_back(HistoryEntry {
            user: user_id,
            message: message.to_string().into(),
        });
        if self.history.len() > Self::MAX_HISTORY_LEN {
            self.history.pop_front();
        }
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
            history: History::new(),
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
