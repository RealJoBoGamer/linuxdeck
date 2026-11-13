//! Helpers for running official Elgato Stream Deck plugins on Linux
//! (Wine for Windows binaries, host Node.js for JS plugins).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use log::{debug, info, warn};

/// Shared Wine prefix used by LinuxDeck for Windows-only Stream Deck plugins.
pub fn shared_wine_prefix() -> PathBuf {
	crate::shared::config_dir().join("wineprefix")
}

/// True when a code path looks like a Windows plugin binary/script.
pub fn is_windows_code_path(code_path: &str) -> bool {
	let lower = code_path.to_lowercase();
	lower.ends_with(".exe")
		|| lower.ends_with(".dll")
		|| lower.ends_with(".bat")
		|| lower.ends_with(".cmd")
		|| lower.ends_with(".msi")
		|| lower.ends_with(".com")
}

fn candidate_exists(path: &Path) -> Option<PathBuf> {
	if path.is_file() {
		Some(path.to_path_buf())
	} else {
		None
	}
}

fn newest_matching(root: &Path, pattern_suffix: &str) -> Option<PathBuf> {
	let Ok(entries) = fs::read_dir(root) else {
		return None;
	};
	let mut matches: Vec<PathBuf> = entries
		.flatten()
		.map(|e| e.path())
		.filter(|p| p.is_dir())
		.collect();
	matches.sort();
	matches.reverse();
	for dir in matches {
		let candidate = dir.join(pattern_suffix);
		if let Some(found) = candidate_exists(&candidate) {
			return Some(found);
		}
	}
	None
}

/// Locate a usable Node.js >= 20 binary (PATH, nvm, fnm, volta, common dirs).
pub fn find_node() -> Option<PathBuf> {
	if let Ok(output) = Command::new("node").arg("--version").output()
		&& output.status.success()
	{
		let version = String::from_utf8_lossy(&output.stdout);
		if version_at_least(version.trim().trim_start_matches('v'), 20) {
			return Some(PathBuf::from("node"));
		}
	}

	if let Ok(home) = std::env::var("HOME") {
		let home = PathBuf::from(home);
		let nvm = newest_matching(&home.join(".nvm/versions/node"), "bin/node");
		if let Some(path) = nvm.filter(|p| node_version_ok(p)) {
			return Some(path);
		}
		let fnm = newest_matching(&home.join(".local/share/fnm/node-versions"), "installation/bin/node")
			.or_else(|| newest_matching(&home.join(".fnm/node-versions"), "installation/bin/node"));
		if let Some(path) = fnm.filter(|p| node_version_ok(p)) {
			return Some(path);
		}
		for rel in [
			".volta/bin/node",
			".local/bin/node",
			".asdf/shims/node",
		] {
			let path = home.join(rel);
			if node_version_ok(&path) {
				return Some(path);
			}
		}
	}

	for path in [
		"/usr/local/bin/node",
		"/usr/bin/node",
		"/opt/node/bin/node",
	] {
		let path = PathBuf::from(path);
		if node_version_ok(&path) {
			return Some(path);
		}
	}

	None
}

fn node_version_ok(path: &Path) -> bool {
	let Ok(output) = Command::new(path).arg("--version").output() else {
		return false;
	};
	if !output.status.success() {
		return false;
	}
	let version = String::from_utf8_lossy(&output.stdout);
	version_at_least(version.trim().trim_start_matches('v'), 20)
}

fn version_at_least(version: &str, major_min: u32) -> bool {
	let major = version
		.split('.')
		.next()
		.and_then(|s| s.parse::<u32>().ok())
		.unwrap_or(0);
	major >= major_min
}

/// Locate Wine (`wine64` preferred, then `wine`, then common paths).
pub fn find_wine() -> Option<PathBuf> {
	for name in ["wine64", "wine"] {
		let result = Command::new(name)
			.arg("--version")
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status();
		if matches!(result, Ok(status) if status.success()) {
			return Some(PathBuf::from(name));
		}
	}

	if let Ok(home) = std::env::var("HOME") {
		for rel in [
			".local/bin/wine64",
			".local/bin/wine",
			"wine/bin/wine64",
			"wine/bin/wine",
		] {
			let path = PathBuf::from(&home).join(rel);
			if wine_ok(&path) {
				return Some(path);
			}
		}
	}

	for path in [
		"/usr/bin/wine64",
		"/usr/bin/wine",
		"/usr/local/bin/wine64",
		"/usr/local/bin/wine",
		"/app/bin/wine",
	] {
		let path = PathBuf::from(path);
		if wine_ok(&path) {
			return Some(path);
		}
	}

	None
}

fn wine_ok(path: &Path) -> bool {
	path.is_file()
		&& Command::new(path)
			.arg("--version")
			.stdout(std::process::Stdio::null())
			.stderr(std::process::Stdio::null())
			.status()
			.map(|s| s.success())
			.unwrap_or(false)
}

/// Ensure the shared Wine prefix exists and mirrors Elgato's Windows plugin layout.
pub fn ensure_elgato_wine_layout(wine: &Path, prefix: &Path, plugins_dir: &Path) -> Result<()> {
	fs::create_dir_all(prefix).context("create wine prefix dir")?;

	if !prefix.join("system.reg").exists() {
		info!("Initialising LinuxDeck Wine prefix at {}", prefix.display());
		let status = Command::new(wine)
			.env("WINEPREFIX", prefix)
			.env("WINEDEBUG", "-all")
			.args(["wineboot", "--init"])
			.status()
			.context("wineboot --init")?;
		if !status.success() {
			warn!("wineboot --init exited with {status}; continuing anyway");
		}
	}

	let users_dir = prefix.join("drive_c/users");
	fs::create_dir_all(&users_dir).ok();

	let mut user_names = vec!["steamuser".to_owned(), "user".to_owned()];
	if let Ok(user) = std::env::var("USER") {
		user_names.insert(0, user);
	}

	for user in user_names {
		let elgato_plugins = users_dir
			.join(&user)
			.join("AppData/Roaming/Elgato/StreamDeck/Plugins");
		if let Err(error) = fs::create_dir_all(&elgato_plugins) {
			debug!("Could not create {}: {error}", elgato_plugins.display());
			continue;
		}

		// Also create the classic Stream Deck app folders some plugins probe.
		for sub in ["Cache", "Logs", "ProfilesV2"] {
			let _ = fs::create_dir_all(
				users_dir
					.join(&user)
					.join("AppData/Roaming/Elgato/StreamDeck")
					.join(sub),
			);
		}

		sync_plugin_links(&elgato_plugins, plugins_dir);
	}

	Ok(())
}

fn sync_plugin_links(elgato_plugins: &Path, plugins_dir: &Path) {
	let Ok(entries) = fs::read_dir(plugins_dir) else {
		return;
	};
	for entry in entries.flatten() {
		let src = entry.path();
		if !src.is_dir() {
			continue;
		}
		let name = entry.file_name();
		let dest = elgato_plugins.join(&name);
		if dest.exists() || dest.symlink_metadata().is_ok() {
			// Refresh broken symlinks.
			if dest.symlink_metadata().map(|m| m.file_type().is_symlink()).unwrap_or(false)
				&& !dest.exists()
			{
				let _ = fs::remove_file(&dest);
			} else {
				continue;
			}
		}
		#[cfg(unix)]
		{
			if let Err(error) = std::os::unix::fs::symlink(&src, &dest) {
				debug!(
					"Failed to link {} -> {}: {error}",
					dest.display(),
					src.display()
				);
			}
		}
		#[cfg(not(unix))]
		{
			let _ = crate::shared::copy_dir(&src, &dest);
		}
	}
}

/// Apply env vars that help Windows Stream Deck plugins under Wine.
pub fn apply_wine_env(command: &mut Command, prefix: &Path) {
	command.env("WINEPREFIX", prefix);
	command.env("WINEDEBUG", "-all");
	command.env("WINEDLLOVERRIDES", "winemenubuilder.exe=d");
}

#[derive(serde::Serialize, Clone)]
pub struct PluginRuntimeStatus {
	pub wine: Option<String>,
	pub wine_ok: bool,
	pub node: Option<String>,
	pub node_ok: bool,
	pub wine_prefix: String,
	pub hint: String,
}

pub fn runtime_status() -> PluginRuntimeStatus {
	let wine = find_wine();
	let node = find_node();
	let wine_ok = wine.is_some();
	let node_ok = node.is_some();
	let mut hints = Vec::new();
	if !wine_ok {
		hints.push("Install Wine (+ Wine Mono) for Windows-only Elgato plugins.".to_owned());
	}
	if !node_ok {
		hints.push("Install Node.js 20+ (nvm/fnm OK) for official Node Stream Deck plugins.".to_owned());
	}
	if hints.is_empty() {
		hints.push("Wine and Node.js look ready for Elgato Stream Deck plugins.".to_owned());
	}
	PluginRuntimeStatus {
		wine: wine.map(|p| p.display().to_string()),
		wine_ok,
		node: node.map(|p| p.display().to_string()),
		node_ok,
		wine_prefix: shared_wine_prefix().display().to_string(),
		hint: hints.join(" "),
	}
}

pub fn require_wine() -> Result<PathBuf> {
	find_wine().ok_or_else(|| {
		anyhow!(
			"Wine not found. Install Wine and Wine Mono to run Windows Elgato Stream Deck plugins. On SteamOS use Distrobox or install Wine on the host."
		)
	})
}

pub fn require_node() -> Result<PathBuf> {
	find_node().ok_or_else(|| {
		anyhow!(
			"Node.js 20+ not found on PATH (nvm/fnm checked). Install Node.js 20 or newer for official Stream Deck JS plugins."
		)
	})
}
