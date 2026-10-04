use indexmap::IndexSet;
use mio::Token;

use crate::connection::{Connection, Queued};
use crate::operation::Operation;

#[derive(Default)]
pub(crate) struct Queues {
    operations: Vec<Operation>,
    pending_writes: Vec<Token>,
    pending_reads: IndexSet<Token>,
    batch: Vec<Token>,
}

impl Queues {
    pub(crate) fn push(&mut self, operation: Operation) {
        self.operations.push(operation);
    }
    pub(crate) fn has_operations(&self) -> bool {
        !self.operations.is_empty()
    }
    pub(crate) fn take_operations(&mut self) -> Vec<Operation> {
        std::mem::take(&mut self.operations)
    }

    pub(crate) fn send(&mut self, conn: &mut Connection, message: &str) {
        match conn.queue_message(message) {
            Queued::NeedsFlush => self.pending_writes.push(conn.token()),
            Queued::AlreadyPending | Queued::Closing => {}
            Queued::Overflow => {
                println!(
                    "{:?} isn't reading (send queue full), disconnecting",
                    conn.token()
                );
                self.push(Operation::Disconnect(conn.token()));
            }
        }
    }
    pub(crate) fn write_later(&mut self, token: Token) {
        self.pending_writes.push(token);
    }

    pub(crate) fn read_later(&mut self, token: Token) {
        self.pending_reads.insert(token);
    }
    pub(crate) fn is_read_pending(&self, token: Token) -> bool {
        self.pending_reads.contains(&token)
    }
    pub(crate) fn has_pending_reads(&self) -> bool {
        !self.pending_reads.is_empty()
    }

    pub(crate) fn take_pending_reads(&mut self) -> Vec<Token> {
        let mut batch = std::mem::take(&mut self.batch);
        batch.clear();
        batch.extend(self.pending_reads.drain(..));
        batch
    }
    pub(crate) fn take_pending_writes(&mut self) -> Vec<Token> {
        let mut batch = std::mem::take(&mut self.batch);
        batch.clear();
        std::mem::swap(&mut batch, &mut self.pending_writes);
        batch
    }
    pub(crate) fn recycle(&mut self, batch: Vec<Token>) {
        self.batch = batch;
    }
}
