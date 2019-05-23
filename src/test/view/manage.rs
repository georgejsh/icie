use crate::{
	test::{
		self, view::{render::render, SCROLL_TO_FIRST_FAILED}, TestRun
	}, util
};
use evscode::{goodies::WebviewHandle, webview::Column, Webview, WebviewResultmap, R};
use std::{fs, path::PathBuf};

lazy_static::lazy_static! {
	pub static ref COLLECTION: WebviewResultmap<Option<PathBuf>, Vec<TestRun>> = WebviewResultmap::new(test::run, create, manage);
}

pub fn touch_input(webview: &Webview) {
	webview.post_message(json::object! {
		"tag" => "new_start",
	});
}

fn create(source: &Option<PathBuf>, runs: &Vec<TestRun>) -> R<Webview> {
	let title = util::fmt_verb("ICIE Test View", &source);
	let webview = evscode::Webview::new("icie.test.view", title, evscode::webview::Column::Beside)
		.enable_scripts()
		.retain_context_when_hidden()
		.create();
	webview.set_html(render(&runs)?);
	webview.reveal(Column::Beside);
	if *SCROLL_TO_FIRST_FAILED.get() {
		webview.post_message(json::object! {
			"tag" => "scroll_to_wa",
		});
	}
	Ok(webview)
}

fn manage(source: &Option<PathBuf>, _: &Vec<TestRun>, webview: WebviewHandle) -> R<Box<dyn FnOnce()+Send+'static>> {
	let webview = webview.lock()?;
	let stream = webview.listener().spawn().cancel_on(webview.disposer());
	let source = source.clone();
	Ok(Box::new(move || {
		for note in stream {
			match note["tag"].as_str() {
				Some("trigger_rr") => evscode::spawn({
					let in_path = PathBuf::from(note["in_path"].as_str().unwrap());
					let source = source.clone();
					move || crate::debug::rr(in_path, source)
				}),
				Some("trigger_gdb") => evscode::spawn({
					let in_path = PathBuf::from(note["in_path"].as_str().unwrap());
					let source = source.clone();
					move || crate::debug::gdb(in_path, source)
				}),
				Some("new_test") => evscode::spawn(move || crate::test::add(note["input"].as_str().unwrap(), note["desired"].as_str().unwrap())),
				Some("set_alt") => evscode::spawn({
					let source = source.clone();
					move || {
						let in_path = PathBuf::from(note["in_path"].as_str().unwrap());
						let out = note["out"].as_str().unwrap();
						fs::write(in_path.with_extension("alt.out"), format!("{}\n", out.trim()))?;
						COLLECTION.get_force(source)?;
						Ok(())
					}
				}),
				Some("del_alt") => evscode::spawn({
					let source = source.clone();
					move || {
						let in_path = PathBuf::from(note["in_path"].as_str().unwrap());
						fs::remove_file(in_path.with_extension("alt.out"))?;
						COLLECTION.get_force(source)?;
						Ok(())
					}
				}),
				_ => log::error!("unrecognied testview webview food `{}`", note.dump()),
			}
		}
	}))
}
