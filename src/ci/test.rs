use crate::ci::{
	exec::{Executable, ExitKind}, task::Task, util::R
};
use std::{fmt, time::Duration};

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
	Accepted { alternative: bool },
	WrongAnswer,
	RuntimeError,
	TimeLimitExceeded,
	IgnoredNoOut,
}

impl Verdict {
	pub fn success(&self) -> bool {
		match self {
			Verdict::Accepted { .. } => true,
			_ => false,
		}
	}
}

#[derive(Debug)]
pub struct Outcome {
	pub verdict: Verdict,
	pub out: String,
	pub stderr: String,
	pub time: Duration,
}

impl Outcome {
	pub fn success(&self) -> bool {
		self.verdict.success()
	}
}

pub fn simple_test(exec: &Executable, input: &str, desired: Option<&str>, alternative: Option<&str>, task: &Task) -> R<Outcome> {
	let run = exec.run(input, &task.environment)?;
	let verdict = match run.exit_kind {
		ExitKind::Normal => {
			if run.status.success() {
				if let Some(desired) = desired {
					if task.checker.judge(input, desired, &run.stdout) {
						Verdict::Accepted { alternative: false }
					} else if let Some(alternative) = alternative {
						if task.checker.judge(input, alternative, &run.stdout) {
							Verdict::Accepted { alternative: true }
						} else {
							Verdict::WrongAnswer
						}
					} else {
						Verdict::WrongAnswer
					}
				} else {
					Verdict::IgnoredNoOut
				}
			} else {
				Verdict::RuntimeError
			}
		},
		ExitKind::TimeLimitExceeded => Verdict::TimeLimitExceeded,
	};
	Ok(Outcome { verdict, out: run.stdout, stderr: run.stderr, time: run.time })
}

impl fmt::Display for Verdict {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		use Verdict::*;
		write!(f, "{}", match self {
			Accepted { .. } => "Accept",
			WrongAnswer => "Wrong Answer",
			RuntimeError => "Runtime Error",
			TimeLimitExceeded => "Time Limit Exceeded",
			IgnoredNoOut => "Ignored (no out)",
		})
	}
}
