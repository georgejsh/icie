//! Integrated terminal support.

use crate::internal::executor::{send_object, HANDLE_FACTORY};
use std::path::PathBuf;

/// Builder object for an integrated terminal.
#[must_use]
pub struct Builder {
	cwd: Option<PathBuf>,
	env: Option<Vec<(String, String)>>,
	name: Option<String>,
	shell_args: Option<Vec<String>>,
	shell_path: Option<PathBuf>,
	strict_env: bool,
}

/// Builder for configuring integrated terminals. See [module documentation](index.html) for details.
impl Builder {
	/// Set the current working directory.
	pub fn cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
		self.cwd = Some(cwd.into());
		self
	}

	/// Add multiple environment variables.
	pub fn envs(mut self, env: impl IntoIterator<Item=(impl AsRef<str>, impl AsRef<str>)>) -> Self {
		let hm = self.env.get_or_insert_with(Vec::new);
		hm.extend(env.into_iter().map(|(a, b)| (a.as_ref().to_owned(), b.as_ref().to_owned())));
		self
	}

	/// Add an environment variable.
	pub fn env(mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Self {
		let hm = self.env.get_or_insert_with(Vec::new);
		hm.push((key.as_ref().to_owned(), value.as_ref().to_owned()));
		self
	}

	/// Set the name visible in the terminal selection.
	pub fn name(mut self, name: impl AsRef<str>) -> Self {
		self.name = Some(name.as_ref().to_owned());
		self
	}

	/// Add multiple arguments to the shell.
	pub fn shell_args(mut self, shell_args: impl IntoIterator<Item=impl AsRef<str>>) -> Self {
		let sa = self.shell_args.get_or_insert_with(Vec::new);
		sa.extend(shell_args.into_iter().map(|a| a.as_ref().to_owned()));
		self
	}

	/// Add an argument to the shell.
	pub fn shell_arg(mut self, arg: impl AsRef<str>) -> Self {
		let sa = self.shell_args.get_or_insert_with(Vec::new);
		sa.push(arg.as_ref().to_owned());
		self
	}

	/// Set the shell executable path.
	pub fn shell_path(mut self, shell_path: impl Into<PathBuf>) -> Self {
		self.shell_path = Some(shell_path.into());
		self
	}

	/// Remove inherited environment variables.
	pub fn strict_env(mut self) -> Self {
		self.strict_env = true;
		self
	}

	/// Spawn the terminal session.
	pub fn create(self) -> Terminal {
		let hid = HANDLE_FACTORY.generate();
		let env = self.env.map(|env| {
			let mut obj = json::object! {};
			for (k, v) in env {
				obj[k] = json::from(v);
			}
			obj
		});
		send_object(json::object! {
			"tag" => "terminal_create",
			"hid" => hid,
			"cwd" => self.cwd.as_ref().map(|p| p.to_str().unwrap()),
			"env" => env,
			"name" => self.name,
			"shellArgs" => self.shell_args,
			"shellPath" => self.shell_path.as_ref().map(|p| p.to_str().unwrap()),
			"strictEnv" => self.strict_env,
		});
		Terminal { hid }
	}
}

/// Integrated terminal provided by the VS Code API.
///
/// See [module documentation](index.html) for more details.
pub struct Terminal {
	hid: u64,
}

impl Terminal {
	/// Create a new builder to configure the terminal.
	pub fn new() -> Builder {
		Builder { cwd: None, env: None, name: None, shell_args: None, shell_path: None, strict_env: false }
	}

	/// Write a text line to the terminal.
	/// VS Code will add a newline by itself.
	pub fn write(&self, text: impl AsRef<str>) {
		send_object(json::object! {
			"tag" => "terminal_write",
			"hid" => self.hid,
			"text" => text.as_ref(),
			"addNewLine" => true,
		});
	}

	/// Make the terminal visible without changing the focus.
	pub fn reveal(&self) {
		self.raw_show(true);
	}

	/// Make the terminal visible and focus it.
	pub fn focus(&self) {
		self.raw_show(false);
	}

	fn raw_show(&self, preserve_focus: bool) {
		send_object(json::object! {
			"tag" => "terminal_show",
			"hid" => self.hid,
			"preserveFocus" => preserve_focus,
		})
	}
}
