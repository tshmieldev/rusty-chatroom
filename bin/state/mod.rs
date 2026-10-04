mod account;
mod group;

use std::borrow::Cow;

use czat::Server;
use mio::Token;
use slotmap::SecondaryMap;

use crate::{
    handlers::commands::Commands,
    state::{account::Accounts, group::Groups},
};

pub use account::{Account, AccountId};
pub use group::{Group, GroupId};

#[derive(Default)]
pub struct ConnectionState {
    account: Option<AccountId>,
}

impl ConnectionState {
    pub fn account(&self) -> Option<AccountId> {
        self.account
    }
}

#[derive(Default)]
pub struct AppState {
    pub accounts: Accounts,
    pub groups: Groups,
    pub commands: Commands,
    online: SecondaryMap<AccountId, Token>,
}

pub struct Chat;

impl Server for Chat {
    type ConnState = ConnectionState;
    type AppState = AppState;
}

impl AppState {
    pub fn set_online(&mut self, conn: &mut ConnectionState, id: AccountId, token: Token) {
        conn.account = Some(id);
        self.online.insert(id, token);
    }
    pub fn set_offline(&mut self, conn: &mut ConnectionState) -> Option<AccountId> {
        let id = conn.account.take()?;
        self.online.remove(id);
        Some(id)
    }
    pub fn session(&self, id: AccountId) -> Option<Token> {
        self.online.get(id).copied()
    }
    pub fn user_by_name(&self, username: &str) -> Option<AccountId> {
        self.accounts.find(username)
    }
    pub fn user(&self, user_id: AccountId) -> Option<&Account> {
        self.accounts.get(user_id)
    }
    pub fn group(&self, group_id: GroupId) -> Option<&Group> {
        self.groups.get(group_id)
    }
    pub fn add_user_to_group(
        &mut self,
        user_id: AccountId,
        group_id: GroupId,
    ) -> Result<bool, Cow<'static, str>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or("Group doesn't exist.")?;
        let user = self
            .accounts
            .get_mut(user_id)
            .ok_or("User doesn't exist.")?;
        Ok(user.join_group(group_id) && group.add(user_id))
    }
    pub fn remove_user_from_group(
        &mut self,
        user_id: AccountId,
        group_id: GroupId,
    ) -> Result<bool, Cow<'static, str>> {
        let group = self
            .groups
            .get_mut(group_id)
            .ok_or("Group doesn't exist.")?;
        let user = self
            .accounts
            .get_mut(user_id)
            .ok_or("User doesn't exist.")?;

        Ok(user.exit_group(group_id) && group.remove(user_id))
    }
    pub fn remove_user_from_all_groups(
        &mut self,
        user_id: AccountId,
    ) -> Result<(), Cow<'static, str>> {
        let user = self
            .accounts
            .get_mut(user_id)
            .ok_or("User doesn't exist.")?;
        let groups = user.groups().clone();

        for group_id in groups.into_iter() {
            let group = self
                .groups
                .get_mut(group_id)
                .ok_or("Group doesn't exist.")?;
            group.remove(user_id);
            user.exit_group(group_id);
        }
        Ok(())
    }
}
