use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use super::types::ShellMenuEntry;

const SHELL_MENU_HOST_ARG: &str = "--lingscape-shell-menu-host";
const SHELL_MENU_TIMEOUT: Duration = Duration::from_secs(10);
static PERSISTENT_HOST: OnceLock<Mutex<Option<PersistentShellHost>>> = OnceLock::new();

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellHostRequest {
    mode: String,
    path: Option<String>,
    menu_path: Vec<u32>,
}

struct PersistentShellHost {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl PersistentShellHost {
    fn spawn() -> Result<Self, String> {
        use std::process::{Command, Stdio};

        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg(SHELL_MENU_HOST_ARG)
            .arg("server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("启动常驻 Shell 菜单进程失败: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "常驻 Shell 菜单进程 stdin 不可用".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "常驻 Shell 菜单进程 stdout 不可用".to_string())?;
        Ok(Self {
            child,
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn request(
        &mut self,
        request: &ShellHostRequest,
    ) -> Result<Result<Vec<ShellMenuEntry>, String>, String> {
        if let Some(status) = self
            .child
            .try_wait()
            .map_err(|e| format!("检查常驻 Shell 菜单进程失败: {e}"))?
        {
            return Err(format!("常驻 Shell 菜单进程已退出: {status}"));
        }
        let line = serde_json::to_string(request)
            .map_err(|e| format!("序列化常驻 Shell 菜单请求失败: {e}"))?;
        self.stdin
            .write_all(line.as_bytes())
            .and_then(|_| self.stdin.write_all(b"\n"))
            .and_then(|_| self.stdin.flush())
            .map_err(|e| format!("写入常驻 Shell 菜单请求失败: {e}"))?;

        let mut response = String::new();
        let n = self
            .stdout
            .read_line(&mut response)
            .map_err(|e| format!("读取常驻 Shell 菜单响应失败: {e}"))?;
        if n == 0 {
            return Err("常驻 Shell 菜单进程已断开".into());
        }
        serde_json::from_str::<Result<Vec<ShellMenuEntry>, String>>(&response)
            .map_err(|e| format!("解析常驻 Shell 菜单响应失败: {e}"))
    }
}

pub(crate) fn shell_host_stage(stage: &str) {
    if let Some(path) = std::env::var_os("LINGSCAPE_SHELL_MENU_STATUS") {
        let _ = fs::write(path, stage);
    }
}

#[cfg(windows)]
pub(crate) fn run_shell_menu_host(
    mode: &str,
    path: Option<&str>,
    menu_path: &[u32],
) -> Result<Vec<ShellMenuEntry>, String> {
    // Root + submenu share the persistent host so blank-desktop cascade
    // preloads do not spawn one process per submenu.
    if mode == "root" || mode == "submenu" {
        return run_persistent_shell_menu_host(mode, path, menu_path);
    }
    run_one_shot_shell_menu_host(mode, path, menu_path)
}

#[cfg(windows)]
fn run_persistent_shell_menu_host(
    mode: &str,
    path: Option<&str>,
    menu_path: &[u32],
) -> Result<Vec<ShellMenuEntry>, String> {
    let request = ShellHostRequest {
        mode: mode.to_string(),
        path: path.map(str::to_string),
        menu_path: menu_path.to_vec(),
    };
    let lock = PERSISTENT_HOST.get_or_init(|| Mutex::new(None));
    let mut guard = lock
        .lock()
        .map_err(|_| "常驻 Shell 菜单进程锁失败".to_string())?;

    for attempt in 0..2 {
        if guard.is_none() {
            *guard = Some(PersistentShellHost::spawn()?);
        }
        let Some(host) = guard.as_mut() else {
            continue;
        };
        match host.request(&request) {
            Ok(result) => return result,
            Err(e) if attempt == 0 => {
                tracing::info!("[desktop-organize] persistent shell menu host restarting: {e}");
                if let Some(mut dead) = guard.take() {
                    let _ = dead.child.kill();
                    let _ = dead.child.wait();
                }
            }
            Err(e) => return Err(e),
        }
    }
    Err("常驻 Shell 菜单进程不可用".into())
}

#[cfg(windows)]
fn run_one_shot_shell_menu_host(
    mode: &str,
    path: Option<&str>,
    menu_path: &[u32],
) -> Result<Vec<ShellMenuEntry>, String> {
    use std::process::{Command, Stdio};

    let output_file = tempfile::Builder::new()
        .prefix("lingscape-shell-menu-")
        .suffix(".json")
        .tempfile()
        .map_err(|e| format!("创建 Shell 菜单临时文件失败: {e}"))?;
    let output_path = output_file.path().to_path_buf();
    let status_path = output_path.with_extension("status");
    let menu_path_json =
        serde_json::to_string(menu_path).map_err(|e| format!("序列化菜单路径失败: {e}"))?;
    let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
        .arg(SHELL_MENU_HOST_ARG)
        .arg(mode)
        .arg(path.unwrap_or(""))
        .arg(menu_path_json)
        .arg(&output_path)
        .env("LINGSCAPE_SHELL_MENU_STATUS", &status_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("启动 Shell 菜单进程失败: {e}"))?;

    let timeout = if mode == "native" || mode == "invoke" {
        Duration::from_secs(300)
    } else {
        SHELL_MENU_TIMEOUT
    };
    let deadline = Instant::now() + timeout;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let bytes =
                    fs::read(&output_path).map_err(|e| format!("读取 Shell 菜单结果失败: {e}"));
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&status_path);
                if !status.success() {
                    return Err(format!("Shell 菜单进程异常退出: {status}"));
                }
                let result: Result<Vec<ShellMenuEntry>, String> =
                    serde_json::from_slice(&bytes?)
                        .map_err(|e| format!("解析 Shell 菜单结果失败: {e}"))?;
                return result;
            }
            Ok(None) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            Ok(None) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&output_path);
                let stage = fs::read_to_string(&status_path).unwrap_or_else(|_| "未知阶段".into());
                let _ = fs::remove_file(&status_path);
                tracing::info!("[desktop-organize] shell menu timeout stage={stage}");
                return Err(format!("菜单加载超时（{stage}）"));
            }
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&status_path);
                return Err(format!("等待 Shell 菜单进程失败: {e}"));
            }
        }
    }
}

/// Handle the isolated Shell-menu subprocess before Tauri starts.
pub fn maybe_run_shell_menu_host() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) != Some(SHELL_MENU_HOST_ARG) {
        return false;
    }

    if args.get(2).map(String::as_str) == Some("server") {
        run_shell_menu_server();
        return true;
    }

    let result = (|| -> Result<Vec<ShellMenuEntry>, String> {
        let mode = args.get(2).ok_or_else(|| "缺少菜单模式".to_string())?;
        let path = args.get(3).ok_or_else(|| "缺少菜单路径".to_string())?;
        let menu_path: Vec<u32> =
            serde_json::from_str(args.get(4).ok_or_else(|| "缺少二级菜单路径".to_string())?)
                .map_err(|e| format!("解析二级菜单路径失败: {e}"))?;
        let path = (!path.is_empty()).then_some(path.as_str());

        #[cfg(windows)]
        {
            let hwnd = crate::shell_menu::create_host_window()?;
            crate::shell_menu::pump_messages();
            let result = match mode.as_str() {
                "root" => crate::shell_menu::list_shell_context_menu(hwnd, path),
                "submenu" => crate::shell_menu::list_shell_context_submenu(hwnd, path, &menu_path),
                "invoke" => {
                    let command_id = *menu_path.first().ok_or_else(|| "缺少命令 ID".to_string())?;
                    let sub_path = if menu_path.len() > 1 {
                        &menu_path[1..]
                    } else {
                        &[]
                    };
                    crate::shell_menu::invoke_shell_context_command(
                        hwnd, path, command_id, sub_path,
                    )
                    .map(|_| Vec::new())
                }
                "native" => crate::shell_menu::show_native_shell_context_menu(hwnd, path)
                    .map(|_| Vec::new()),
                _ => Err("未知菜单模式".into()),
            };
            crate::shell_menu::pump_messages();
            crate::shell_menu::destroy_host_window(hwnd);
            result
        }
        #[cfg(not(windows))]
        {
            let _ = (mode, path, menu_path);
            Err("桌面整理仅支持 Windows".into())
        }
    })();

    if let Some(output) = args.get(5) {
        if let Ok(json) = serde_json::to_vec(&result) {
            let _ = fs::write(output, json);
        }
    }
    true
}

fn run_shell_menu_server() {
    let result = (|| -> Result<(), String> {
        #[cfg(windows)]
        {
            let hwnd = crate::shell_menu::create_host_window()?;
            crate::shell_menu::pump_messages();
            let stdin = std::io::stdin();
            let mut stdout = std::io::stdout();
            for line in stdin.lock().lines() {
                let result = match line {
                    Ok(line) => handle_shell_menu_server_line(hwnd, &line),
                    Err(e) => Err(format!("读取常驻 Shell 菜单请求失败: {e}")),
                };
                if serde_json::to_writer(&mut stdout, &result).is_err() {
                    break;
                }
                if stdout
                    .write_all(b"\n")
                    .and_then(|_| stdout.flush())
                    .is_err()
                {
                    break;
                }
                crate::shell_menu::pump_messages();
            }
            crate::shell_menu::destroy_host_window(hwnd);
            Ok(())
        }
        #[cfg(not(windows))]
        {
            Err("桌面整理仅支持 Windows".into())
        }
    })();
    if let Err(e) = result {
        let _ = writeln!(std::io::stderr(), "{e}");
    }
}

#[cfg(windows)]
fn handle_shell_menu_server_line(
    hwnd: windows::Win32::Foundation::HWND,
    line: &str,
) -> Result<Vec<ShellMenuEntry>, String> {
    let request: ShellHostRequest =
        serde_json::from_str(line).map_err(|e| format!("解析常驻 Shell 菜单请求失败: {e}"))?;
    match request.mode.as_str() {
        "root" => crate::shell_menu::list_shell_context_menu(hwnd, request.path.as_deref()),
        "submenu" => crate::shell_menu::list_shell_context_submenu(
            hwnd,
            request.path.as_deref(),
            &request.menu_path,
        ),
        _ => Err("常驻 Shell 菜单进程不支持该模式".into()),
    }
}
