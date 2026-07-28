use anyhow::{bail, Context};
use std::{env, fs, path::PathBuf};

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub terminal: Vec<String>,
    pub file_manager: Vec<String>,
    pub browser: Vec<String>,
    pub launcher: Vec<String>,
    pub modifier: String,
    pub gaps: u32,
    pub border_width: u32,
    pub focused_border: u32,
    pub unfocused_border: u32,
    pub master_ratio: f32,
    pub workspaces: usize,
    pub autostart: Option<PathBuf>,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            terminal: vec!["kitty".into()],
            file_manager: vec!["thunar".into()],
            browser: vec!["firefox-esr".into()],
            launcher: vec!["boringwm-rofi".into()],
            modifier: "Mod4".into(),
            gaps: 8,
            border_width: 2,
            focused_border: 0x88ccff,
            unfocused_border: 0x333333,
            master_ratio: 0.6,
            workspaces: 9,
            autostart: None,
        }
    }
}

impl Config {
    pub fn path() -> Option<PathBuf> {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .map(|p| p.join("boringwm/config.toml"))
    }
    pub fn load() -> anyhow::Result<Self> {
        let Some(path) = Self::path() else {
            return Ok(Self::default());
        };
        if !path.exists() {
            let mut c = Self::default();
            c.set_default_autostart();
            return Ok(c);
        }
        let text =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let mut c = Self::parse(&text)
            .with_context(|| format!("invalid configuration in {}", path.display()))?;
        c.set_default_autostart();
        Ok(c)
    }
    fn set_default_autostart(&mut self) {
        if self.autostart.is_none() {
            self.autostart =
                env::var_os("HOME").map(|h| PathBuf::from(h).join(".config/boringwm/autostart.sh"));
        }
    }
    fn parse(input: &str) -> anyhow::Result<Self> {
        let mut c = Self::default();
        for (line_number, raw) in input.lines().enumerate() {
            let line = raw.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                bail!("line {}: expected key = value", line_number + 1)
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                "terminal" => c.terminal = parse_command(value)?,
                "file_manager" => c.file_manager = parse_command(value)?,
                "browser" => c.browser = parse_command(value)?,
                "launcher" => c.launcher = parse_command(value)?,
                "modifier" => c.modifier = parse_string(value)?,
                "gaps" => c.gaps = value.parse()?,
                "border_width" => c.border_width = value.parse()?,
                "focused_border" => c.focused_border = parse_color(value)?,
                "unfocused_border" => c.unfocused_border = parse_color(value)?,
                "master_ratio" => c.master_ratio = value.parse()?,
                "workspaces" => c.workspaces = value.parse()?,
                "autostart" => c.autostart = Some(PathBuf::from(parse_string(value)?)),
                _ => bail!("line {}: unknown field {key}", line_number + 1),
            }
        }
        if c.workspaces == 0 || c.workspaces > 9 {
            bail!("workspaces must be between 1 and 9")
        }
        if c.gaps > 100 || c.border_width > 20 {
            bail!("gaps must be <= 100 and border_width <= 20")
        }
        if !(0.2..=0.8).contains(&c.master_ratio) {
            bail!("master_ratio must be between 0.2 and 0.8")
        }
        if c.modifier != "Mod4" {
            bail!("only modifier = \"Mod4\" is currently supported")
        }
        Ok(c)
    }
}
fn parse_string(v: &str) -> anyhow::Result<String> {
    let v = v.trim();
    if v.len() < 2
        || !((v.starts_with('"') && v.ends_with('"')) || (v.starts_with('\'') && v.ends_with('\'')))
    {
        bail!("expected quoted string")
    }
    Ok(v[1..v.len() - 1].to_owned())
}
fn parse_command(v: &str) -> anyhow::Result<Vec<String>> {
    let v = v.trim();
    if !v.starts_with('[') || !v.ends_with(']') {
        bail!("command must be an array of quoted arguments")
    }
    let inner = &v[1..v.len() - 1];
    if inner.trim().is_empty() {
        bail!("command must not be empty")
    }
    inner
        .split(',')
        .map(|part| parse_string(part.trim()))
        .collect()
}
fn parse_color(v: &str) -> anyhow::Result<u32> {
    let s = parse_string(v)?;
    let digits = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix('#'))
        .unwrap_or(&s);
    Ok(u32::from_str_radix(digits, 16)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_valid() {
        Config::parse("").unwrap();
    }
    #[test]
    fn parses_commands_and_colors() {
        let c = Config::parse(
            "terminal = [\"xterm\", \"-name\", \"boring\"]\nfocused_border = \"#abcdef\"",
        )
        .unwrap();
        assert_eq!(c.terminal[1], "-name");
        assert_eq!(c.focused_border, 0xabcdef);
    }
    #[test]
    fn rejects_unknown_fields() {
        assert!(Config::parse("surprise = true").is_err());
    }
    #[test]
    fn rejects_invalid_values() {
        assert!(Config::parse("workspaces = 0").is_err());
        assert!(Config::parse("master_ratio = 0.9").is_err());
        assert!(Config::parse("terminal = []").is_err());
    }
}
