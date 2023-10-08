use crate::{
	compile, executable::{Environment, Executable}, service::Service, terminal, terminal::BashTerminal, test, util, util::{fs, path::Path, SourceTarget}
};
use evscode::{E, R};

pub const GDB: Service = Service {
	human_name: "GDB",
	exec_linuxmac: Some("gdb"),
	exec_windows: None,
	package_apt: Some("gdb"),
	package_brew: Some("gdb"),
	package_pacman: Some("gdb"),
	tutorial_url_windows: None,
	supports_linux: true,
	supports_windows: false,
	supports_macos: true,
    compile_flags: None,
    run_flags: None
};

pub const RR: Service = Service {
	human_name: "RR",
	exec_linuxmac: Some("rr"),
	exec_windows: None,
	package_apt: Some("rr"),
	package_brew: None,
	package_pacman: None,
	tutorial_url_windows: None,
	supports_linux: true,
	supports_windows: false,
	supports_macos: false,
    compile_flags: None,
    run_flags: None
};

pub async fn gdb(in_path: &Path, source: SourceTarget) -> R<()> {
	let gdb = GDB.find_command().await?;
	terminal::debugger("GDB", in_path, &[
		&gdb,
		"-q",
		compile::executable_path(source)?.as_str(),
		"-ex",
		&format!("set args < {}", util::bash_escape(in_path.as_str())),
	])
	.await
}

pub async fn rr(in_path: &Path, source: SourceTarget) -> R<()> {
	let rr = RR.find_command().await?;
	let rr_exec = Executable::new_name(rr.clone());
	let input = fs::read_to_string(in_path).await?;
	let exec_path = compile::executable_path(source)?;
	let args = ["record", exec_path.as_str()];
	let environment = Environment { time_limit: test::time_limit(), cwd: None };
	let record_out = rr_exec.run(&input, &args, &environment).await?;
	if record_out.stderr.contains("/proc/sys/kernel/perf_event_paranoid") {
		return Err(E::error(
			"RR is not configured properly (this is to be expected), kernel.perf_event_paranoid must be <= 1",
		)
		.action("🔐 Auto-configure", configure_kernel_perf_event_paranoid()));
	}
	terminal::debugger("RR", in_path, &[&rr, "replay", "--", "-q"]).await
}

async fn configure_kernel_perf_event_paranoid() -> R<()> {
	terminal::Internal
		.spawn_bash(
			"ICIE Auto-configure RR",
			"echo 'kernel.perf_event_paranoid=1' | pkexec tee -a /etc/sysctl.conf && echo 1 | pkexec tee -a \
			 /proc/sys/kernel/perf_event_paranoid",
		)
		.await
}
