use serde_json::json;

use crate::{AppContext, transactions_log::FullTransactionLog};

pub enum DomainEvent {
    TransactionCreated(FullTransactionLog),
}

pub struct DomainEventDispatcher;

impl DomainEventDispatcher {
    pub fn dispatch_events(context: &AppContext, events: Vec<DomainEvent>) -> anyhow::Result<()> {
        for event in events {
            Self::dispatch(context, event)?;
        }
        Ok(())
    }

    fn dispatch(context: &AppContext, event: DomainEvent) -> anyhow::Result<()> {
        match event {
            DomainEvent::TransactionCreated(log) => {
                context
                    .event_manager
                    .emit_all("new_transaction", json!(log))?;
            }
        }
        Ok(())
    }
}
