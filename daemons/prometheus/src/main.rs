use anyhow::{Context, Result};
use glob::glob;
use std::fs;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use zbus::{interface, ConnectionBuilder};

const CPU_GOVERNOR_PATH: &str = "/sys/devices/system/cpu/cpu*/cpufreq/scaling_governor";
const CPU_EPP_PATH: &str = "/sys/devices/system/cpu/cpu*/cpufreq/energy_performance_preference";
const TURBO_PATH: &str = "/sys/devices/system/cpu/intel_pstate/no_turbo";

// Hardware Safety Lock - Supported models
const SUPPORTED_MODELS: &[&str] = &[
    "Nitro AN515-57",
];

// DMI paths for hardware identification
const DMI_PRODUCT_NAME: &str = "/sys/class/dmi/id/product_name";

#[derive(Debug, Clone, Copy)]
enum PerformanceProfile {
    Silent,
    Balanced,
    WarSpeed,
}

impl PerformanceProfile {
    fn governor(&self) -> &str {
        match self {
            PerformanceProfile::Silent => "powersave",
            PerformanceProfile::Balanced => "powersave",
            PerformanceProfile::WarSpeed => "performance",
        }
    }

    fn epp(&self) -> &str {
        match self {
            PerformanceProfile::Silent => "power",
            PerformanceProfile::Balanced => "balance_performance",
            PerformanceProfile::WarSpeed => "performance",
        }
    }

    fn turbo_enabled(&self) -> bool {
        match self {
            PerformanceProfile::Silent => false,
            PerformanceProfile::Balanced => true,
            PerformanceProfile::WarSpeed => true,
        }
    }
}

/// Verify hardware compatibility before allowing operation
fn verify_hardware() -> Result<()> {
    info!("Performing hardware compatibility check...");

    // Read product name
    let product_name = fs::read_to_string(DMI_PRODUCT_NAME)
        .context("Failed to read DMI product name. Are you running on physical hardware?")?
        .trim()
        .to_string();

    info!("Detected hardware: {}", product_name);

    // Check if this hardware is supported
    let is_supported = SUPPORTED_MODELS.iter().any(|model| product_name.contains(model));

    if !is_supported {
        error!("HARDWARE SAFETY LOCK ENGAGED");
        error!("Detected model: {}", product_name);
        error!("This daemon is designed ONLY for: {:?}", SUPPORTED_MODELS);
        error!("");
        error!("Running this daemon on unsupported hardware may cause:");
        error!("  - CPU instability");
        error!("  - Thermal issues");
        error!("  - System crashes");
        error!("  - Frequency scaling problems");
        error!("");
        error!("If you believe your hardware should be supported, please:");
        error!("  1. Verify your exact model number");
        error!("  2. Open an issue at: https://github.com/yourrepo/boreas");
        error!("  3. DO NOT bypass this safety check");

        anyhow::bail!(
            "Hardware safety check failed. Detected: '{}'. Supported: {:?}",
            product_name,
            SUPPORTED_MODELS
        );
    }

    info!("✓ Hardware compatibility verified: {}", product_name);
    Ok(())
}

struct CpuController;

impl CpuController {
    fn new() -> Result<Self> {
        // Verify we have access to CPU control interfaces
        if !std::path::Path::new(TURBO_PATH).exists() {
            anyhow::bail!("Intel P-State driver not available");
        }
        Ok(Self)
    }

    fn set_governor(&self, governor: &str) -> Result<()> {
        info!("Setting CPU governor to: {}", governor);

        let paths: Vec<_> = glob(CPU_GOVERNOR_PATH)
            .context("Failed to glob governor paths")?
            .filter_map(Result::ok)
            .collect();

        if paths.is_empty() {
            anyhow::bail!("No CPU governor control files found");
        }

        let count = paths.len();
        for path in paths {
            fs::write(&path, governor)
                .with_context(|| format!("Failed to write to {:?}", path))?;
        }

        info!("Governor set for {} CPUs", count);
        Ok(())
    }

    fn set_epp(&self, epp: &str) -> Result<()> {
        info!("Setting Energy Performance Preference to: {}", epp);

        let paths: Vec<_> = glob(CPU_EPP_PATH)
            .context("Failed to glob EPP paths")?
            .filter_map(Result::ok)
            .collect();

        if paths.is_empty() {
            warn!("No EPP control files found (may not be supported)");
            return Ok(());
        }

        let count = paths.len();
        for path in paths {
            if let Err(e) = fs::write(&path, epp) {
                warn!("Failed to write EPP to {:?}: {}", path, e);
            }
        }

        info!("EPP set for {} CPUs", count);
        Ok(())
    }

    fn set_turbo(&self, enabled: bool) -> Result<()> {
        let value = if enabled { "0" } else { "1" }; // 0 = turbo enabled, 1 = disabled
        info!("Setting turbo boost: {}", if enabled { "enabled" } else { "disabled" });

        fs::write(TURBO_PATH, value)
            .context("Failed to write turbo setting")?;

        Ok(())
    }

    fn apply_profile(&self, profile: PerformanceProfile) -> Result<()> {
        info!("Applying performance profile: {:?}", profile);

        self.set_governor(profile.governor())?;
        self.set_epp(profile.epp())?;
        self.set_turbo(profile.turbo_enabled())?;

        info!("Performance profile {:?} applied successfully", profile);
        Ok(())
    }

    fn get_current_governor(&self) -> Result<String> {
        let paths: Vec<_> = glob(CPU_GOVERNOR_PATH)
            .context("Failed to glob governor paths")?
            .filter_map(Result::ok)
            .collect();

        if let Some(path) = paths.first() {
            let governor = fs::read_to_string(path)
                .context("Failed to read governor")?
                .trim()
                .to_string();
            Ok(governor)
        } else {
            anyhow::bail!("No governor paths found")
        }
    }

    fn get_current_epp(&self) -> Result<String> {
        let paths: Vec<_> = glob(CPU_EPP_PATH)
            .context("Failed to glob EPP paths")?
            .filter_map(Result::ok)
            .collect();

        if let Some(path) = paths.first() {
            let epp = fs::read_to_string(path)
                .context("Failed to read EPP")?
                .trim()
                .to_string();
            Ok(epp)
        } else {
            Ok("not_supported".to_string())
        }
    }
}

struct PrometheusService {
    cpu: Arc<CpuController>,
    current_profile: Arc<Mutex<Option<PerformanceProfile>>>,
}

#[interface(name = "org.jesternet.Prometheus")]
impl PrometheusService {
    async fn set_performance_profile(&self, profile: &str) -> zbus::fdo::Result<String> {
        let profile_enum = match profile.to_lowercase().as_str() {
            "silent" => PerformanceProfile::Silent,
            "balanced" => PerformanceProfile::Balanced,
            "warspeed" | "war_speed" => PerformanceProfile::WarSpeed,
            _ => {
                return Err(zbus::fdo::Error::InvalidArgs(
                    format!("Unknown profile: {}. Valid: silent, balanced, warspeed", profile)
                ));
            }
        };

        info!("Setting performance profile to: {:?}", profile_enum);

        if let Err(e) = self.cpu.apply_profile(profile_enum) {
            error!("Failed to apply performance profile: {}", e);
            return Err(zbus::fdo::Error::Failed(format!("CPU control error: {}", e)));
        }

        *self.current_profile.lock().await = Some(profile_enum);

        Ok(format!("Performance profile set to: {}", profile))
    }

    async fn get_current_profile(&self) -> String {
        if let Some(profile) = *self.current_profile.lock().await {
            format!("{:?}", profile)
        } else {
            "Unknown".to_string()
        }
    }

    async fn get_cpu_status(&self) -> zbus::fdo::Result<(String, String)> {
        match (self.cpu.get_current_governor(), self.cpu.get_current_epp()) {
            (Ok(gov), Ok(epp)) => Ok((gov, epp)),
            (Err(e), _) | (_, Err(e)) => {
                error!("Failed to read CPU status: {}", e);
                Err(zbus::fdo::Error::Failed(format!("CPU read error: {}", e)))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Prometheus Performance Control Daemon starting...");
    info!("Version: 1.0.0");
    info!("Project: https://github.com/yourrepo/boreas");

    // CRITICAL: Verify hardware compatibility before proceeding
    verify_hardware()?;

    let cpu = Arc::new(CpuController::new()?);

    let service = PrometheusService {
        cpu: cpu.clone(),
        current_profile: Arc::new(Mutex::new(None)),
    };

    info!("Connecting to system D-Bus...");
    let _conn = ConnectionBuilder::system()?
        .name("org.jesternet.Prometheus")?
        .serve_at("/org/jesternet/Prometheus", service)?
        .build()
        .await?;

    info!("Prometheus daemon ready on D-Bus: org.jesternet.Prometheus");
    info!("Available profiles: silent, balanced, warspeed");

    // Keep running
    std::future::pending::<()>().await;

    Ok(())
}
