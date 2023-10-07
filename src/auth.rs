use crate::util::is_installed;
use evscode::{error::Severity, E, R,glue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen::{JsCast, JsValue};
use log::info;
use node_sys::console;
// TODO: check how errors work w/o libsecret/gnome-keyring

#[derive(serde::Deserialize, serde::Serialize)]
struct Credentials {
	username: String,
	password: String,
}

pub async fn get_force_ask(site: &str) -> R<(String, String)> {
	let message = format!("Username at {}", site);
	let username = evscode::InputBox::new().prompt(&message).ignore_focus_out().show().await.ok_or_else(E::cancel)?;
	let message = format!("Password for {} at {}", username, site);
	let password =
		evscode::InputBox::new().prompt(&message).password().ignore_focus_out().show().await.ok_or_else(E::cancel)?;
	let kr = Keyring::new("credentials", site);
	if !kr
		.set(&serde_json::to_string(&Credentials { username: username.clone(), password: password.clone() }).unwrap())
		.await
	{
		E::error("failed to save password to a secure keyring, so it will not be remembered")
			.severity(Severity::Warning)
			.action_if(is_installed("kwalletd5").await?, "How to fix (KWallet)", help_fix_kwallet())
			.emit();
	}
	Ok((username, password))
}

pub async fn get_cached_or_ask(site: &str) -> R<(String, String)> {
	let kr = Keyring::new("credentials", site);
	match kr.get().await {
		Some(encoded) => {
			let creds: Credentials = serde_json::from_str(&encoded).unwrap();
			Ok((creds.username, creds.password))
		},
		None => get_force_ask(site).await,
	}
}

pub async fn get_if_cached(site: &str) -> Option<String> {
	Keyring::new("session", site).get().await
}

pub async fn save_cache(site: &str, value: &str) {
	Keyring::new("session", site).set(value).await; // ignore save fail
}

pub async fn has_any_saved(site: &str) -> bool {
	Keyring::new("session", site).get().await.is_some() || Keyring::new("credentials", site).get().await.is_some()
}

#[evscode::command(title = "ICIE Password reset from URL")]
async fn reset_from_url() -> R<()> {
	let url = evscode::InputBox::new()
		.prompt("Enter any contest/task URL from the site for which you want to reset the password")
		.placeholder("https://codeforces.com/contest/.../problem/...")
		.ignore_focus_out()
		.show()
		.await
		.ok_or_else(E::cancel)?;
	let site = crate::net::interpret_url(&url)?.0.site;
	Keyring::new("credentials", &site).delete().await;
	Keyring::new("session", &site).delete().await;
	Ok(())
}

#[evscode::command(title = "ICIE Password reset from list")]
async fn reset_from_list() -> R<()> {
	let credentials_list = Keyring::list().await;
	let credentials = evscode::QuickPick::new()
		.items(credentials_list.into_iter().map(|credentials| {
			let label = credentials.clone();
			evscode::quick_pick::Item::new(credentials, label)
		}))
		.show()
		.await
		.ok_or_else(E::cancel)?;
	Keyring::delete_entry(&credentials).await;
	Ok(())
}

async fn help_fix_kwallet() -> R<()> {
	evscode::open_external("https://github.com/pustaczek/icie/issues/14#issuecomment-516982482").await
}

struct Keyring {
	kind: &'static str,
	site: String,
}
impl Keyring {
	fn new(kind: &'static str, site: &str) -> Keyring {
		Keyring { kind, site: site.to_owned() }
	}

	async fn get(&self) -> Option<String> {
		let entry = format!("@{} {}", self.kind, self.site);
        
		
        let secret = glue::EXTENSION_CONTEXT
        .with(|ext_ctx| vscode_sys::ExtensionContext::get_secrets(ext_ctx.get().unwrap().unchecked_ref::<vscode_sys::ExtensionContext>()));
        
		let all_list = match secret.get_password(&entry).await {
			Some(val) => {
                //console::debug(&format!("requesting: {} ---{}",entry,val));
                //self.delete().await;
                    Some(val)
            },
            _ => None
		};        
        return all_list;
	}

	async fn set(&self, value: &str)-> bool {
		let entry = format!("@{} {}", self.kind, self.site);
        let entryall = "all";;
        
        let secret = glue::EXTENSION_CONTEXT
        .with(|ext_ctx| vscode_sys::ExtensionContext::get_secrets(ext_ctx.get().unwrap().unchecked_ref::<vscode_sys::ExtensionContext>()));
        
        let mut all_list = match secret.get_password(&entryall).await {
			Some(val) => val,
            _ => "".to_string()
		};
        all_list+=&("#".to_owned()+&entry);
        
        secret.set_password(&entryall,&all_list).await;
		secret.set_password(&entry, value).await;
        true
	}

	async fn delete(&self) {
		let entry = format!("@{} {}", self.kind, self.site);
		Keyring::delete_entry(&entry).await
	}

	async fn list() -> Vec<String> {
		let entryall = "all";
        
        let secret = glue::EXTENSION_CONTEXT
        .with(|ext_ctx| vscode_sys::ExtensionContext::get_secrets(ext_ctx.get().unwrap().unchecked_ref::<vscode_sys::ExtensionContext>()));
        
        let all_list = match secret.get_password(&entryall).await {
			Some(val) => val,
            _ => "".to_string()
		};
        let mut uniq:Vec<String>=all_list.split('#').collect::<Vec<_>>().iter().filter(|s| !s.is_empty()).map(|s| s.to_string()).collect();
        uniq.sort();
        uniq.dedup();
        uniq        
	}

	async fn delete_entry(entry: &str) {
        let secret = glue::EXTENSION_CONTEXT
        .with(|ext_ctx| vscode_sys::ExtensionContext::get_secrets(ext_ctx.get().unwrap().unchecked_ref::<vscode_sys::ExtensionContext>()));
        
		secret.delete_password(entry).await;
//        console::debug(&format!("deleting: {}",entry));
          
        let entryall = "all";
        
        let secret = glue::EXTENSION_CONTEXT
        .with(|ext_ctx| vscode_sys::ExtensionContext::get_secrets(ext_ctx.get().unwrap().unchecked_ref::<vscode_sys::ExtensionContext>()));
        
        let mut all_list = match secret.get_password(&entryall).await {
			Some(val) => val,
            _ => "".to_string()
		};
        let all_vals: Vec<_>=all_list.split('#').collect::<Vec<&str>>().iter().filter(|s| (**s)!=entry).map(|s| s.to_string()).collect();
        secret.set_password(&entryall,&all_vals.join("#")).await;
		        
	}
}
