use crate::{
	executable::Executable, terminal, util::{self, is_installed, OS}
};
use evscode::{E, R};

pub struct Service {
	pub human_name: &'static str,
	pub exec_linuxmac: Option<&'static str>,
	pub exec_windows: Option<&'static str>,
	pub package_apt: Option<&'static str>,
	pub package_brew: Option<&'static str>,
	pub package_pacman: Option<&'static str>,
	pub tutorial_url_windows: Option<&'static str>,
	pub supports_linux: bool,
	pub supports_windows: bool,
	pub supports_macos: bool,
    pub compile_flags: Option<&'static [&'static str]>,
    pub run_flags: Option<&'static [&'static str]>
}

impl Service {
	pub async fn find_executable(&'static self) -> R<Executable> {
		self.find_command().await.map(Executable::new_name)
	}

	pub async fn find_command(&'static self) -> R<String> {
		let command = self.get_exec().ok_or_else(|| E::error(self.fmt_supported_platforms()))?;
		if !is_installed(command).await? {
			return Err(self.not_installed().await?);
		}
		Ok(command.to_owned())
	}

	fn fmt_supported_platforms(&self) -> String {
		let mut platforms = Vec::new();
		if self.supports_linux {
			platforms.push("Linux");
		}
		if self.supports_windows {
			platforms.push("Windows");
		}
		if self.supports_macos {
			platforms.push("macOS");
		}
		format!("{} is only supported on {}", self.human_name, util::fmt::list(&platforms))
	}

	pub async fn not_installed(&'static self) -> R<E> {
		let mut e = E::error(format!("{} is not installed", self.human_name));
		match OS::query()? {
			OS::Linux => {
				if let Some(package) = self.package_apt {
					if is_installed("apt").await? {
						e = e.action("🔐 Auto-install (apt)".to_owned(), async move { apt_install(package).await });
					}
				}
				if let Some(package) = self.package_pacman {
					if is_installed("pacman").await? {
						e = e.action("🔐 Auto-install (pacman)".to_owned(), async move { pacman_s(package).await });
					}
				}
			},
			OS::Windows => {
				if let Some(tutorial) = self.tutorial_url_windows {
					e = e.action("📄 How to install?".to_owned(), async move { tutorial_show(tutorial).await });
				}
			},
			OS::MacOS => {
				if let Some(package) = self.package_brew {
					if is_installed("brew").await? {
						e = e.action("🔐 Auto install (brew)".to_owned(), async move { brew_install(package).await });
					}
				}
			},
		}
		Ok(e)
	}

	fn get_exec(&self) -> Option<&'static str> {
		match OS::query() {
			Ok(OS::Linux) | Ok(OS::MacOS) => self.exec_linuxmac,
			Ok(OS::Windows) => self.exec_windows,
			Err(_) => None,
		}
	}
}

async fn apt_install(package: &'static str) -> R<()> {
	terminal::install(package, &["pkexec", "apt", "install", "-y", package]).await
}

async fn brew_install(package: &'static str) -> R<()> {
	terminal::install(package, &["brew", "install", package]).await
}

async fn pacman_s(package: &'static str) -> R<()> {
	terminal::install(package, &["pkexec", "pacman", "-S", package]).await
}

async fn tutorial_show(url: &'static str) -> R<()> {
	evscode::open_external(url).await
}
