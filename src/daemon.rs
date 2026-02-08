use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use tracing::{info, warn};

fn get_pid_file() -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let run_dir = home.join(".claude-code-voice");
    fs::create_dir_all(&run_dir)?;
    Ok(run_dir.join("daemon.pid"))
}

pub async fn start_daemon(config: crate::config::Config) -> Result<()> {
    // Check if daemon is already running
    if is_running().await? {
        anyhow::bail!("Daemon is already running. Use 'stop' command first.");
    }

    #[cfg(unix)]
    {
        start_unix_daemon(config).await
    }

    #[cfg(windows)]
    {
        start_windows_daemon(config).await
    }
}

#[cfg(unix)]
async fn start_unix_daemon(config: crate::config::Config) -> Result<()> {
    use daemonize::Daemonize;

    let pid_file = get_pid_file()?;
    let stdout_file = get_log_file("stdout")?;
    let stderr_file = get_log_file("stderr")?;

    let stdout = fs::File::create(&stdout_file)?;
    let stderr = fs::File::create(&stderr_file)?;

    info!("Starting daemon...");
    info!("Logs: stdout={}, stderr={}", stdout_file.display(), stderr_file.display());

    let daemonize = Daemonize::new()
        .pid_file(pid_file)
        .working_directory(std::env::current_dir()?)
        .stdout(stdout)
        .stderr(stderr);

    match daemonize.start() {
        Ok(_) => {
            // This code runs in the daemon process
            info!("Daemon started successfully");
            crate::run_voice_input(config).await?;
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Failed to daemonize: {}", e)
        }
    }
}

#[cfg(windows)]
async fn start_windows_daemon(config: crate::config::Config) -> Result<()> {
    // On Windows, we'll spawn a detached process instead of a proper service
    let exe = std::env::current_exe()?;
    let pid_file = get_pid_file()?;

    let child = Command::new(exe)
        .arg("start")
        .arg("--foreground")
        .arg("--model")
        .arg(&config.model_size)
        .arg("--hotkey")
        .arg(&config.hotkey)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to spawn daemon process")?;

    // Write PID file
    fs::write(&pid_file, child.id().to_string())?;

    info!("Daemon started with PID: {}", child.id());
    Ok(())
}

pub async fn stop_daemon() -> Result<()> {
    let pid_file = get_pid_file()?;

    if !pid_file.exists() {
        anyhow::bail!("Daemon is not running (no PID file found)");
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = pid_str
        .trim()
        .parse()
        .context("Invalid PID in pid file")?;

    info!("Stopping daemon with PID: {}", pid);

    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        kill(Pid::from_raw(pid as i32), Signal::SIGTERM)
            .context("Failed to send SIGTERM to daemon")?;

        // Wait a bit for graceful shutdown
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Check if still running
        if is_running().await? {
            warn!("Daemon did not stop gracefully, sending SIGKILL");
            kill(Pid::from_raw(pid as i32), Signal::SIGKILL)
                .context("Failed to send SIGKILL to daemon")?;
        }
    }

    #[cfg(windows)]
    {
        use std::process;
        Command::new("taskkill")
            .args(["/PID", &pid.to_string(), "/F"])
            .output()
            .context("Failed to kill daemon process")?;
    }

    // Remove PID file
    fs::remove_file(&pid_file)?;

    info!("Daemon stopped");
    Ok(())
}

pub async fn get_status() -> Result<String> {
    if is_running().await? {
        let pid_file = get_pid_file()?;
        let pid = fs::read_to_string(&pid_file)?;
        Ok(format!("Daemon is running (PID: {})", pid.trim()))
    } else {
        Ok("Daemon is not running".to_string())
    }
}

async fn is_running() -> Result<bool> {
    let pid_file = get_pid_file()?;

    if !pid_file.exists() {
        return Ok(false);
    }

    let pid_str = fs::read_to_string(&pid_file)?;
    let pid: u32 = match pid_str.trim().parse() {
        Ok(p) => p,
        Err(_) => {
            // Invalid PID file, clean it up
            fs::remove_file(&pid_file)?;
            return Ok(false);
        }
    };

    // Check if process is actually running
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        // Send signal 0 to check if process exists (doesn't actually send a signal)
        match kill(Pid::from_raw(pid as i32), None) {
            Ok(_) => Ok(true),
            Err(_) => {
                // Process not running, clean up PID file
                fs::remove_file(&pid_file)?;
                Ok(false)
            }
        }
    }

    #[cfg(windows)]
    {
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {}", pid)])
            .output()
            .context("Failed to check process status")?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        let running = output_str.contains(&pid.to_string());

        if !running {
            fs::remove_file(&pid_file)?;
        }

        Ok(running)
    }
}

fn get_log_file(name: &str) -> Result<PathBuf> {
    let home = dirs::home_dir().context("Failed to get home directory")?;
    let log_dir = home.join(".claude-code-voice").join("logs");
    fs::create_dir_all(&log_dir)?;
    Ok(log_dir.join(format!("{}.log", name)))
}
