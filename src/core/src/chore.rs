use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::UserId;

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

  /// If Some, only these users see / can complete this chore.
  /// None means it's a common chore visible to everyone.
  pub assigned_to: Option<Vec<UserId>>,

  /// Chores that must be completed before this one is actionable.
  /// TODO: external-event dependencies are not yet modelled – see questions.md.
  pub depends_on: Vec<ChoreId>,

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

  /// Compute when this chore is next due, given the current time.
  /// Returns None if the chore is not currently due.
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
            if due <= now { Some(due) } else { Some(due) }
          }
        }
      }

      // Scheduled recurrence: next due date must be computed from the schedule string.
      // For the prototype this always returns now; proper cron parsing is future work.
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

  /// True if this chore has unresolved dependencies.
  pub fn blocked_by<'a>(&self, all: impl Iterator<Item = &'a Chore>) -> Vec<ChoreId> {
    let completed_ids: std::collections::HashSet<ChoreId> = all
      .filter(|c| !c.completions.is_empty())
      .map(|c| c.id)
      .collect();
    self.depends_on.iter()
      .filter(|dep| !completed_ids.contains(dep))
      .copied()
      .collect()
  }
}
