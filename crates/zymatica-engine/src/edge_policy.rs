use crate::model::QuantMode;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoPriority {
    Balanced,
    Throughput,
    Parity,
}

impl AutoPriority {
    pub fn from_env() -> Self {
        match std::env::var("ZYMATICA_AUTO_PRIORITY")
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "throughput" | "speed" | "fast" => Self::Throughput,
            "parity" | "quality" | "exact" => Self::Parity,
            _ => Self::Balanced,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EdgeDeviceProfile {
    pub os: String,
    pub arch: String,
    pub total_memory_mb: Option<u64>,
    pub available_memory_mb: Option<u64>,
    pub cpu_temp_c: Option<f64>,
}

impl EdgeDeviceProfile {
    pub fn detect() -> Self {
        Self {
            os: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
            total_memory_mb: read_meminfo_mb("MemTotal:"),
            available_memory_mb: read_meminfo_mb("MemAvailable:"),
            cpu_temp_c: read_cpu_temp_c(),
        }
    }

    pub fn synthetic(
        os: impl Into<String>,
        arch: impl Into<String>,
        total_memory_mb: Option<u64>,
        available_memory_mb: Option<u64>,
        cpu_temp_c: Option<f64>,
    ) -> Self {
        Self {
            os: os.into(),
            arch: arch.into(),
            total_memory_mb,
            available_memory_mb,
            cpu_temp_c,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineDecision {
    pub mode: QuantMode,
    pub priority: AutoPriority,
    pub reason: String,
    pub estimated_peak_mb: u64,
    pub recommended_cache: bool,
}

impl EngineDecision {
    pub fn engine_name(&self) -> &'static str {
        self.mode.as_str()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ThermalQuantizationConfig {
    pub cool_below_c: f64,
    pub hot_above_c: f64,
    pub critical_above_c: f64,
    pub q8_min_available_mb: u64,
    pub q5_min_available_mb: u64,
}

impl Default for ThermalQuantizationConfig {
    fn default() -> Self {
        Self {
            cool_below_c: 62.0,
            hot_above_c: 78.0,
            critical_above_c: 86.0,
            q8_min_available_mb: estimated_peak_mb(QuantMode::Q8) + 512,
            q5_min_available_mb: estimated_peak_mb(QuantMode::Q5) + 384,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalQuantizationAction {
    Hold,
    Downgrade,
    Upgrade,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThermalQuantizationDecision {
    pub previous_mode: QuantMode,
    pub selected_mode: QuantMode,
    pub action: ThermalQuantizationAction,
    pub cpu_temp_c: Option<f64>,
    pub available_memory_mb: Option<u64>,
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct ThermalQuantizationController {
    current_mode: QuantMode,
    config: ThermalQuantizationConfig,
}

impl ThermalQuantizationController {
    pub fn new(initial_mode: QuantMode, config: ThermalQuantizationConfig) -> Self {
        Self {
            current_mode: initial_mode,
            config,
        }
    }

    pub fn current_mode(&self) -> QuantMode {
        self.current_mode
    }

    pub fn observe(&mut self, profile: &EdgeDeviceProfile) -> ThermalQuantizationDecision {
        let previous_mode = self.current_mode;
        let temp = profile.cpu_temp_c;
        let available = profile.available_memory_mb;
        let (selected_mode, reason) = match temp {
            Some(value) if value >= self.config.critical_above_c => (
                QuantMode::Q4,
                format!(
                    "critical thermal pressure {:.1}C >= {:.1}C; forcing q4 dispatch",
                    value, self.config.critical_above_c
                ),
            ),
            Some(value) if value >= self.config.hot_above_c => {
                let mode = downgrade_one(previous_mode);
                (
                    mode,
                    format!(
                        "thermal pressure {:.1}C >= {:.1}C; stepping precision down to {}",
                        value,
                        self.config.hot_above_c,
                        mode.as_str()
                    ),
                )
            }
            Some(value) if value <= self.config.cool_below_c => {
                let candidate = upgrade_one(previous_mode);
                let mode = highest_mode_with_memory(candidate, available, self.config);
                if mode == previous_mode {
                    (
                        mode,
                        format!(
                            "cool {:.1}C but available RAM {} MB does not permit higher precision",
                            value,
                            display_mb(available)
                        ),
                    )
                } else {
                    (
                        mode,
                        format!(
                            "cool {:.1}C <= {:.1}C with RAM headroom; stepping precision up to {}",
                            value,
                            self.config.cool_below_c,
                            mode.as_str()
                        ),
                    )
                }
            }
            Some(value) => (
                previous_mode,
                format!(
                    "temperature {:.1}C is inside hysteresis band [{:.1}C, {:.1}C]; holding {}",
                    value,
                    self.config.cool_below_c,
                    self.config.hot_above_c,
                    previous_mode.as_str()
                ),
            ),
            None => (
                highest_mode_with_memory(previous_mode, available, self.config),
                "temperature unavailable; holding memory-valid precision mode".to_string(),
            ),
        };

        self.current_mode = selected_mode;
        ThermalQuantizationDecision {
            previous_mode,
            selected_mode,
            action: thermal_action(previous_mode, selected_mode),
            cpu_temp_c: temp,
            available_memory_mb: available,
            reason,
        }
    }
}

pub fn decide_quant_mode(profile: &EdgeDeviceProfile, priority: AutoPriority) -> EngineDecision {
    let available = profile.available_memory_mb.unwrap_or(u64::MAX);
    let total = profile.total_memory_mb.unwrap_or(u64::MAX);
    let temp = profile.cpu_temp_c.unwrap_or(0.0);
    let is_arm_edge = profile.arch == "aarch64" || profile.arch == "arm" || profile.arch == "arm64";
    let thermal_pressure = temp >= 78.0;
    let constrained_ram = available < 1_800 || total <= 4_096;
    let moderate_ram = available < 2_800 || total <= 6_144;

    let (mode, reason) = if thermal_pressure {
        (
            QuantMode::Q4,
            format!(
                "selected q4 because CPU temperature is {:.1}C; lowering bandwidth and compute pressure",
                temp
            ),
        )
    } else if constrained_ram {
        (
            QuantMode::Q4,
            format!(
                "selected q4 because available RAM is {} MB and total RAM is {} MB",
                display_mb(profile.available_memory_mb),
                display_mb(profile.total_memory_mb)
            ),
        )
    } else {
        match priority {
            AutoPriority::Throughput => (
                QuantMode::Q4,
                "selected q4 because ZYMATICA_AUTO_PRIORITY=throughput favors highest tok/sec"
                    .to_string(),
            ),
            AutoPriority::Parity => {
                if moderate_ram {
                    (
                        QuantMode::Q5,
                        format!(
                            "selected q5 because parity was requested but available RAM is {} MB",
                            display_mb(profile.available_memory_mb)
                        ),
                    )
                } else {
                    (
                        QuantMode::Q8,
                        "selected q8 because parity was requested and RAM headroom is sufficient"
                            .to_string(),
                    )
                }
            }
            AutoPriority::Balanced => {
                if moderate_ram {
                    (
                        QuantMode::Q4,
                        format!(
                            "selected q4 because available RAM is {} MB; balanced policy reserves field headroom",
                            display_mb(profile.available_memory_mb)
                        ),
                    )
                } else if is_arm_edge {
                    (
                        QuantMode::Q5,
                        "selected q5 because ARM edge hardware has enough RAM and q5 is the best quality/speed balance"
                            .to_string(),
                    )
                } else {
                    (
                        QuantMode::Q5,
                        "selected q5 because balanced policy prefers the quality/speed middle path"
                            .to_string(),
                    )
                }
            }
        }
    };

    EngineDecision {
        mode,
        priority,
        reason,
        estimated_peak_mb: estimated_peak_mb(mode),
        recommended_cache: true,
    }
}

pub fn estimated_peak_mb(mode: QuantMode) -> u64 {
    match mode {
        QuantMode::Q8 => 2_300,
        QuantMode::Q5 => 1_600,
        QuantMode::Q4 => 1_350,
        QuantMode::Q3 => 950,
        QuantMode::Q1_58 => 650,
    }
}

fn downgrade_one(mode: QuantMode) -> QuantMode {
    match mode {
        QuantMode::Q8 => QuantMode::Q5,
        QuantMode::Q5 => QuantMode::Q4,
        QuantMode::Q4 => QuantMode::Q3,
        QuantMode::Q3 | QuantMode::Q1_58 => QuantMode::Q1_58,
    }
}

fn upgrade_one(mode: QuantMode) -> QuantMode {
    match mode {
        QuantMode::Q1_58 => QuantMode::Q3,
        QuantMode::Q3 => QuantMode::Q4,
        QuantMode::Q4 => QuantMode::Q5,
        QuantMode::Q5 | QuantMode::Q8 => QuantMode::Q8,
    }
}

fn highest_mode_with_memory(
    candidate: QuantMode,
    available: Option<u64>,
    config: ThermalQuantizationConfig,
) -> QuantMode {
    let available = available.unwrap_or(u64::MAX);
    match candidate {
        QuantMode::Q8 if available >= config.q8_min_available_mb => QuantMode::Q8,
        QuantMode::Q8 | QuantMode::Q5 if available >= config.q5_min_available_mb => QuantMode::Q5,
        QuantMode::Q8 | QuantMode::Q5 | QuantMode::Q4 if available >= 1000 => QuantMode::Q4,
        QuantMode::Q8 | QuantMode::Q5 | QuantMode::Q4 | QuantMode::Q3 if available >= 750 => {
            QuantMode::Q3
        }
        _ => QuantMode::Q1_58,
    }
}

fn thermal_action(previous_mode: QuantMode, selected_mode: QuantMode) -> ThermalQuantizationAction {
    let previous_rank = quant_mode_rank(previous_mode);
    let selected_rank = quant_mode_rank(selected_mode);
    if selected_rank < previous_rank {
        ThermalQuantizationAction::Downgrade
    } else if selected_rank > previous_rank {
        ThermalQuantizationAction::Upgrade
    } else {
        ThermalQuantizationAction::Hold
    }
}

fn quant_mode_rank(mode: QuantMode) -> u8 {
    match mode {
        QuantMode::Q1_58 => 0,
        QuantMode::Q3 => 1,
        QuantMode::Q4 => 2,
        QuantMode::Q5 => 3,
        QuantMode::Q8 => 4,
    }
}

fn display_mb(value: Option<u64>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn read_meminfo_mb(key: &str) -> Option<u64> {
    let raw = std::fs::read_to_string("/proc/meminfo").ok()?;
    raw.lines().find_map(|line| {
        let rest = line.strip_prefix(key)?;
        let kb = rest.split_whitespace().next()?.parse::<u64>().ok()?;
        Some(kb / 1024)
    })
}

fn read_cpu_temp_c() -> Option<f64> {
    let raw = std::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").ok()?;
    let milli_c = raw.trim().parse::<f64>().ok()?;
    Some(milli_c / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_selects_q4_when_memory_is_tight() {
        let profile =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(4096), Some(1400), None);
        let decision = decide_quant_mode(&profile, AutoPriority::Balanced);
        assert_eq!(decision.mode, QuantMode::Q4);
        assert!(decision.reason.contains("available RAM"));
    }

    #[test]
    fn auto_selects_q5_for_balanced_pi_with_headroom() {
        let profile =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(5200), Some(54.0));
        let decision = decide_quant_mode(&profile, AutoPriority::Balanced);
        assert_eq!(decision.mode, QuantMode::Q5);
    }

    #[test]
    fn auto_selects_q8_for_parity_when_ram_allows() {
        let profile =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(6200), Some(54.0));
        let decision = decide_quant_mode(&profile, AutoPriority::Parity);
        assert_eq!(decision.mode, QuantMode::Q8);
    }

    #[test]
    fn auto_selects_q4_under_thermal_pressure() {
        let profile =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(6200), Some(82.0));
        let decision = decide_quant_mode(&profile, AutoPriority::Parity);
        assert_eq!(decision.mode, QuantMode::Q4);
        assert!(decision.reason.contains("temperature"));
    }

    #[test]
    fn thermal_controller_downgrades_under_heat_and_recovers_when_cool() {
        let mut controller =
            ThermalQuantizationController::new(QuantMode::Q8, ThermalQuantizationConfig::default());
        let warm =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(6200), Some(80.0));
        let decision = controller.observe(&warm);
        assert_eq!(decision.previous_mode, QuantMode::Q8);
        assert_eq!(decision.selected_mode, QuantMode::Q5);
        assert_eq!(decision.action, ThermalQuantizationAction::Downgrade);

        let hot =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(6200), Some(88.0));
        let decision = controller.observe(&hot);
        assert_eq!(decision.selected_mode, QuantMode::Q4);
        assert_eq!(decision.action, ThermalQuantizationAction::Downgrade);

        let cool =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(8192), Some(6200), Some(55.0));
        let decision = controller.observe(&cool);
        assert_eq!(decision.selected_mode, QuantMode::Q5);
        assert_eq!(decision.action, ThermalQuantizationAction::Upgrade);
        let decision = controller.observe(&cool);
        assert_eq!(decision.selected_mode, QuantMode::Q8);
        assert_eq!(decision.action, ThermalQuantizationAction::Upgrade);
    }

    #[test]
    fn thermal_controller_refuses_upgrade_without_memory_headroom() {
        let mut controller =
            ThermalQuantizationController::new(QuantMode::Q4, ThermalQuantizationConfig::default());
        let cool =
            EdgeDeviceProfile::synthetic("linux", "aarch64", Some(4096), Some(1200), Some(48.0));
        let decision = controller.observe(&cool);
        assert_eq!(decision.selected_mode, QuantMode::Q4);
        assert_eq!(decision.action, ThermalQuantizationAction::Hold);
    }
}
