//! Sound packs: discovery and playback of <iso>.wav cues.

use std::path::{Path, PathBuf};

use crate::util;
use crate::win;

pub struct SoundLibrary {
    base: PathBuf,
    packs: Vec<String>,
}

fn has_wav(dir: &Path) -> bool {
    let Ok(rd) = std::fs::read_dir(dir) else {
        return false;
    };
    rd.flatten().any(|e| {
        e.path()
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("wav"))
            .unwrap_or(false)
    })
}

fn is_safe_pack_name(name: &str) -> bool {
    !name.is_empty() && !name.contains('/') && !name.contains('\\') && name != "." && name != ".."
}

impl SoundLibrary {
    pub fn new() -> Self {
        let exe = util::exe_dir();
        let mut base = exe.clone();
        let mut candidates = vec![exe.clone()];
        if let Some(parent) = exe.parent() {
            candidates.push(parent.to_path_buf());
            if let Some(grand) = parent.parent() {
                candidates.push(grand.to_path_buf());
            }
        }
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd);
        }
        for c in candidates {
            if c.join("sound_pack").is_dir() {
                base = c;
                break;
            }
        }
        let mut lib = Self {
            base,
            packs: Vec::new(),
        };
        lib.rescan();
        lib
    }

    pub fn rescan(&mut self) {
        self.packs.clear();
        let root = self.base.join("sound_pack");
        if let Ok(rd) = std::fs::read_dir(&root) {
            for entry in rd.flatten() {
                let path = entry.path();
                if !path.is_dir() || !has_wav(&path) {
                    continue;
                }
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if is_safe_pack_name(name) {
                        self.packs.push(name.to_string());
                    }
                }
            }
        }
        self.packs
            .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
    }

    pub fn packs(&self) -> &[String] {
        &self.packs
    }

    /// Returns a valid pack name, preferring `wanted`, then "01_default",
    /// then the first available pack.
    pub fn ensure_pack(&self, wanted: &str) -> String {
        if is_safe_pack_name(wanted) && self.packs.iter().any(|p| p.eq_ignore_ascii_case(wanted)) {
            return self
                .packs
                .iter()
                .find(|p| p.eq_ignore_ascii_case(wanted))
                .cloned()
                .unwrap_or_default();
        }
        if let Some(p) = self
            .packs
            .iter()
            .find(|p| p.eq_ignore_ascii_case("01_default"))
        {
            return p.clone();
        }
        self.packs.first().cloned().unwrap_or_default()
    }

    fn candidates(&self, pack: &str, iso: &str) -> Vec<PathBuf> {
        let iso = if iso.is_empty() {
            "default".to_string()
        } else {
            iso.to_lowercase()
        };
        let root = self.base.join("sound_pack");
        let mut v = Vec::new();
        if is_safe_pack_name(pack) {
            v.push(root.join(pack).join(format!("{}.wav", iso)));
            v.push(root.join(pack).join("default.wav"));
        }
        if !pack.eq_ignore_ascii_case("01_default") {
            v.push(root.join("01_default").join(format!("{}.wav", iso)));
            v.push(root.join("01_default").join("default.wav"));
        }
        // Old flat layout.
        v.push(self.base.join(format!("{}.wav", iso)));
        v.push(self.base.join("default.wav"));
        v
    }

    pub fn play(&self, pack: &str, iso: &str) {
        for path in self.candidates(pack, iso) {
            if path.is_file() {
                let w = win::wide_os(path.as_os_str());
                unsafe {
                    win::PlaySoundW(
                        w.as_ptr(),
                        std::ptr::null_mut(),
                        win::SND_ASYNC | win::SND_FILENAME | win::SND_NODEFAULT,
                    );
                }
                return;
            }
        }
        util::debug(&format!(
            "Звук для языка '{}' не найден (пакет '{})",
            iso, pack
        ));
    }
}
