use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{UserId, event::{EventId, ExternalEvent}};

pub type ChoreId = Uuid;

/// How and when a chore recurs (or doesn't).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChoreKind {
  /// Done once; disappears from the pending list after completion.
  OneTime,

  /// Becomes pending again N seconds after last completion.
  RecurringAfterCompletion { delay_secs: u64 },

  /// Becomes pending on a fixed cron schedule (UTC).
  /// Prototype uses a simple "HH:MM weekday/daily/weekly" string;
  /// full cron syntax TBD once the design is clarified.
  RecurringScheduled { schedule: String },

  /// Must be done before the deadline.
  WithDeadline { deadline: DateTime<Utc> },
}

/// A single recorded completion of a chore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
  pub completed_at: DateTime<Utc>,
  pub completed_by: UserId,
}

/// A chore in the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chore {
  pub id: ChoreId,
  pub title: String,
  pub kind: ChoreKind,

  // --- permission fields (Q4) ---
  /// Who can see this chore. None = everyone.
  pub visible_to: Option<Vec<UserId>>,

  /// Primary assignee. None = no specific assignee.
  pub assignee: Option<UserId>,

  /// Who may mark this chore done. None = everyone.
  pub can_complete: Option<Vec<UserId>>,

  // --- dependency fields ---
  /// Other chores that must be completed before this one is actionable.
  pub depends_on: Vec<ChoreId>,

  /// External events that must be triggered before this one is actionable (Q3).
  pub depends_on_events: Vec<EventId>,

  pub created_at: DateTime<Utc>,
  pub created_by: UserId,

  /// Full completion history (most-recent last).
  pub completions: Vec<Completion>,
}

impl Chore {
  /// Most-recent completion, if any.
  pub fn last_completion(&self) -> Option<&Completion> {
    self.completions.last()
  }

  /// True if `user` can see this chore.
  pub fn visible_to_user(&self, user: UserId) -> bool {
    match &self.visible_to {
      None => true,
      Some(list) => list.contains(&user),
    }
  }

  /// True if `user` is allowed to complete this chore.
  pub fn completable_by(&self, user: UserId) -> bool {
    match &self.can_complete {
      None => true,
      Some(list) => list.contains(&user),
    }
  }

  /// Compute when this chore is next due, given the current time.
  /// Returns None if the chore has no pending work.
  pub fn next_due(&self, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    match &self.kind {
      ChoreKind::OneTime => {
        if self.completions.is_empty() { Some(now) } else { None }
      }

      ChoreKind::RecurringAfterCompletion { delay_secs } => {
        match self.last_completion() {
          None => Some(now), // never done → due immediately
          Some(c) => {
            let due = c.completed_at + chrono::Duration::seconds(*delay_secs as i64);
            Some(due) // always return the due date (may be past or future)
          }
        }
      }

      // Scheduled recurrence: proper cron parsing is future work.
      ChoreKind::RecurringScheduled { .. } => Some(now),

      ChoreKind::WithDeadline { deadline } => {
        if *deadline > now && self.completions.is_empty() {
          Some(*deadline)
        } else {
          None
        }
      }
    }
  }

  /// True if this chore is blocked by unmet chore or event dependencies.
  pub fn is_blocked(
    &self,
    all_chores: &HashMap<ChoreId, Chore>,
    all_events: &HashMap<EventId, ExternalEvent>,
  ) -> bool {
    // Check chore dependencies: each dep must have at least one completion.
    let completed_chores: HashSet<ChoreId> = all_chores.values()
      .filter(|c| !c.completions.is_empty())
      .map(|c| c.id)
      .collect();
    let chore_blocked = self.depends_on.iter().any(|dep| !completed_chores.contains(dep));

    // Check event dependencies: each dep must be triggered.
    let event_blocked = self.depends_on_events.iter()
      .any(|eid| !all_events.get(eid).map_or(false, |e| e.triggered));

    chore_blocked || event_blocked
  }
}
