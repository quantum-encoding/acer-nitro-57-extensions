use anyhow::{Context, Result};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn, error};
use zbus::{interface, ConnectionBuilder};

const EC_IO_PATH: &str = "/sys/kernel/debug/ec/ec0/io";

// Hardware Safety Lock - Supported models
const SUPPORTED_MODELS: &[&str] = &[
    "Nitro AN515-57",
];

// DMI paths for hardware identification
const DMI_PRODUCT_NAME: &str = "/sys/class/dmi/id/product_name";
const DMI_BOARD_VENDOR: &str = "/sys/class/dmi/id/board_vendor";

// EC Register addresses for Acer Nitro AN515-57
const REG_MANUAL_CONTROL: u64 = 3;
const REG_GPU_FAN_MODE: u64 = 33;
const REG_CPU_FAN_MODE: u64 = 34;
const REG_CPU_FAN_READ: u64 = 19;
const REG_GPU_FAN_READ: u64 = 21;
const REG_CPU_FAN_WRITE: u64 = 55;
const REG_GPU_FAN_WRITE: u64 = 58;

// Control values
const VAL_MANUAL_CONTROL_ENABLE: u8 = 17;
const VAL_CPU_FAN_MANUAL: u8 = 12;
const VAL_GPU_FAN_MANUAL: u8 = 48;
const VAL_MANUAL_CONTROL_DISABLE: u8 = 0;
const VAL_CPU_FAN_AUTO: u8 = 4;
const VAL_GPU_FAN_AUTO: u8 = 16;

#[derive(Debug, Clone, Copy)]
enum FanProfile {
    Silent,
    Balanced,
    MaxPower,
    Auto,
}

impl FanProfile {
    fn cpu_speed(&self) -> u8 {
        match self {
            FanProfile::Silent => 25,
            FanProfile::Balanced => 50,
            FanProfile::MaxPower => 100,
            FanProfile::Auto => 50, // Will be reset to auto mode
        }
    }

    fn gpu_speed(&self) -> u8 {
        match self {
            FanProfile::Silent => 25,
            FanProfile::Balanced => 50,
            FanProfile::MaxPower => 100,
            FanProfile::Auto => 50,
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
        error!("  - Hardware damage");
        error!("  - System instability");
        error!("  - Thermal runaway");
        error!("  - Permanent component failure");
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

/// Validate fan speed value is within safe range
fn validate_fan_speed(speed: u8) -> Result<u8> {
    if speed > 100 {
        anyhow::bail!(
            "Invalid fan speed: {}. Must be 0-100.",
            speed
        );
    }
    Ok(speed)
}

struct EcController {
    file: Arc<Mutex<File>>,
}

impl EcController {
    fn new() -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(EC_IO_PATH)
            .context("Failed to open EC interface. Ensure ec_sys module is loaded with write_support=1")?;

        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }

    async fn read_register(&self, register: u64) -> Result<u8> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(register))?;
        let mut buf = [0u8; 1];
        file.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    async fn write_register(&self, register: u64, value: u8) -> Result<()> {
        let mut file = self.file.lock().await;
        file.seek(SeekFrom::Start(register))?;
        file.write_all(&[value])?;
        file.flush()?;
        Ok(())
    }

    async fn initialize_manual_control(&self) -> Result<()> {
        info!("Initializing manual fan control");
        self.write_register(REG_MANUAL_CONTROL, VAL_MANUAL_CONTROL_ENABLE).await?;
        self.write_register(REG_CPU_FAN_MODE, VAL_CPU_FAN_MANUAL).await?;
        self.write_register(REG_GPU_FAN_MODE, VAL_GPU_FAN_MANUAL).await?;
        info!("Manual fan control enabled");
        Ok(())
    }

    async fn restore_auto_control(&self) -> Result<()> {
        info!("Restoring automatic fan control");
        self.write_register(REG_CPU_FAN_MODE, VAL_CPU_FAN_AUTO).await?;
        self.write_register(REG_GPU_FAN_MODE, VAL_GPU_FAN_AUTO).await?;
        self.write_register(REG_MANUAL_CONTROL, VAL_MANUAL_CONTROL_DISABLE).await?;
        info!("Automatic fan control restored");
        Ok(())
    }

    async fn set_fan_speeds(&self, cpu_speed: u8, gpu_speed: u8) -> Result<()> {
        // Validate inputs
        let cpu = validate_fan_speed(cpu_speed)?;
        let gpu = validate_fan_speed(gpu_speed)?;

        info!("Setting fan speeds: CPU={}%, GPU={}%", cpu, gpu);
        self.write_register(REG_CPU_FAN_WRITE, cpu).await?;
        self.write_register(REG_GPU_FAN_WRITE, gpu).await?;
        Ok(())
    }

    async fn get_fan_speeds(&self) -> Result<(u8, u8)> {
        let cpu = self.read_register(REG_CPU_FAN_READ).await?;
        let gpu = self.read_register(REG_GPU_FAN_READ).await?;
        Ok((cpu, gpu))
    }
}

struct BoreasService {
    ec: Arc<EcController>,
    current_profile: Arc<Mutex<FanProfile>>,
}

#[interface(name = "org.jesternet.Boreas")]
impl BoreasService {
    async fn set_fan_profile(&self, profile: &str) -> zbus::fdo::Result<String> {
        let profile_enum = match profile.to_lowercase().as_str() {
            "silent" => FanProfile::Silent,
            "balanced" => FanProfile::Balanced,
            "maxpower" | "max_power" | "max" => FanProfile::MaxPower,
            "auto" => FanProfile::Auto,
            _ => {
                return Err(zbus::fdo::Error::InvalidArgs(
                    format!("Unknown profile: {}. Valid: silent, balanced, maxpower, auto", profile)
                ));
            }
        };

        info!("Setting fan profile to: {:?}", profile_enum);

        if matches!(profile_enum, FanProfile::Auto) {
            if let Err(e) = self.ec.restore_auto_control().await {
                error!("Failed to restore auto control: {}", e);
                return Err(zbus::fdo::Error::Failed(format!("EC error: {}", e)));
            }
        } else {
            if let Err(e) = self.ec.initialize_manual_control().await {
                error!("Failed to initialize manual control: {}", e);
                return Err(zbus::fdo::Error::Failed(format!("EC error: {}", e)));
            }

            if let Err(e) = self.ec.set_fan_speeds(
                profile_enum.cpu_speed(),
                profile_enum.gpu_speed()
            ).await {
                error!("Failed to set fan speeds: {}", e);
                return Err(zbus::fdo::Error::Failed(format!("EC error: {}", e)));
            }
        }

        *self.current_profile.lock().await = profile_enum;

        Ok(format!("Fan profile set to: {}", profile))
    }

    async fn get_fan_speeds(&self) -> zbus::fdo::Result<(u8, u8)> {
        match self.ec.get_fan_speeds().await {
            Ok(speeds) => Ok(speeds),
            Err(e) => {
                error!("Failed to read fan speeds: {}", e);
                Err(zbus::fdo::Error::Failed(format!("EC error: {}", e)))
            }
        }
    }

    async fn get_current_profile(&self) -> String {
        let profile = *self.current_profile.lock().await;
        format!("{:?}", profile)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Boreas Fan Control Daemon starting...");
    info!("Version: 1.0.0");
    info!("Project: https://github.com/yourrepo/boreas");

    // CRITICAL: Verify hardware compatibility before proceeding
    verify_hardware()?;

    let ec = Arc::new(EcController::new()?);

    let service = BoreasService {
        ec: ec.clone(),
        current_profile: Arc::new(Mutex::new(FanProfile::Auto)),
    };

    info!("Connecting to system D-Bus...");
    let _conn = ConnectionBuilder::system()?
        .name("org.jesternet.Boreas")?
        .serve_at("/org/jesternet/Boreas", service)?
        .build()
        .await?;

    info!("Boreas daemon ready on D-Bus: org.jesternet.Boreas");
    info!("Available profiles: silent, balanced, maxpower, auto");

    // Keep running
    std::future::pending::<()>().await;

    Ok(())
}
