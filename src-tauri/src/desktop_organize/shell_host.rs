use std::fs;
use std::time::{Duration, Instant};

use super::types::ShellMenuEntry;

const SHELL_MENU_HOST_ARG: &str = "--lingscape-shell-menu-host";
const SHELL_MENU_TIMEOUT: Duration = Duration::from_secs(10);

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
                let bytes = fs::read(&output_path)
                    .map_err(|e| format!("读取 Shell 菜单结果失败: {e}"));
                let _ = fs::remove_file(&output_path);
                let _ = fs::remove_file(&status_path);
                if !status.success() {
                    return Err(format!("Shell 菜单进程异常退出: {status}"));
                }
                let result: Result<Vec<ShellMenuEntry>, String> = serde_json::from_slice(
                    &bytes?,
                )
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
                let stage = fs::read_to_string(&status_path)
                    .unwrap_or_else(|_| "未知阶段".into());
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

    let result = (|| -> Result<Vec<ShellMenuEntry>, String> {
        let mode = args.get(2).ok_or_else(|| "缺少菜单模式".to_string())?;
        let path = args.get(3).ok_or_else(|| "缺少菜单路径".to_string())?;
        let menu_path: Vec<u32> = serde_json::from_str(
            args.get(4).ok_or_else(|| "缺少二级菜单路径".to_string())?,
        )
        .map_err(|e| format!("解析二级菜单路径失败: {e}"))?;
        let path = (!path.is_empty()).then_some(path.as_str());

        #[cfg(windows)]
        {
            let hwnd = crate::shell_menu::create_host_window()?;
            crate::shell_menu::pump_messages();
            let result = match mode.as_str() {
                "root" => crate::shell_menu::list_shell_context_menu(hwnd, path),
                "submenu" => {
                    crate::shell_menu::list_shell_context_submenu(hwnd, path, &menu_path)
                }
                "invoke" => {
                    let command_id = *menu_path.first().ok_or_else(|| "缺少命令 ID".to_string())?;
                    let sub_path = if menu_path.len() > 1 {
                        &menu_path[1..]
                    } else {
                        &[]
                    };
                    crate::shell_menu::invoke_shell_context_command(
                        hwnd,
                        path,
                        command_id,
                        sub_path,
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
