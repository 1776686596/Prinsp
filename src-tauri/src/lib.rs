use arboard::Clipboard;
use base64::{engine::general_purpose::STANDARD, Engine};
use image::{GrayImage, ImageFormat, Pixel, RgbImage};
use imageproc::contrast::{otsu_level, threshold};
use imageproc::distance_transform::Norm;
use imageproc::filter::median_filter;
use imageproc::morphology::close;
use rusty_tesseract::{Args, Image as TessImage};
use std::collections::HashMap;
use std::io::Cursor;
use std::process::Command;
use std::sync::{mpsc, Mutex, OnceLock};
use std::thread;
use std::time::Duration;
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewWindow,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, ShortcutState};
use xcap::Monitor;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CaptureBackend {
    Grim,
    Xcap,
    GnomeScreenshot,
}

static PREFERRED_BACKEND: OnceLock<Mutex<Option<CaptureBackend>>> = OnceLock::new();

fn preferred_backend_state() -> &'static Mutex<Option<CaptureBackend>> {
    PREFERRED_BACKEND.get_or_init(|| Mutex::new(None))
}

fn set_preferred_backend(backend: CaptureBackend) {
    if let Ok(mut guard) = preferred_backend_state().lock() {
        *guard = Some(backend);
    }
}

fn get_preferred_backend() -> Option<CaptureBackend> {
    preferred_backend_state()
        .lock()
        .ok()
        .and_then(|guard| *guard)
}

fn command_exists(cmd: &str) -> bool {
    Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd} >/dev/null 2>&1"))
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ensure_tesseract_installed() -> Result<(), String> {
    if command_exists("tesseract") {
        Ok(())
    } else {
        Err("未找到 tesseract，可先安装：sudo apt install tesseract-ocr tesseract-ocr-chi-sim（或对应发行版包名）".to_string())
    }
}

fn preselect_backend() {
    // 简单的环境探测，避免第一次截图时走到不可用后端造成长时间阻塞
    if get_preferred_backend().is_some() {
        return;
    }

    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    if is_wayland && command_exists("grim") {
        set_preferred_backend(CaptureBackend::Grim);
        return;
    }

    if std::env::var("DISPLAY").is_ok() {
        set_preferred_backend(CaptureBackend::Xcap);
    }
}

#[tauri::command]
fn hide_window(window: WebviewWindow) -> Result<(), String> {
    window.hide().map_err(|e| e.to_string())
}

#[tauri::command]
fn register_global_shortcut(app: AppHandle, shortcut: String) -> Result<(), String> {
    let manager = app.global_shortcut();
    manager
        .unregister_all()
        .map_err(|e| format!("unregister_all: {e}"))?;

    let normalized = normalize_shortcut(&shortcut);
    let shortcut_label = normalized.clone();
    manager
        .on_shortcut(normalized.as_str(), move |handle, _shortcut, _event| {
            // 全局快捷键触发后通知前端开始截图
            let _ = handle.emit("start-capture", ());
        })
        .map_err(|e| format!("register {shortcut_label}: {e}"))?;

    Ok(())
}

fn normalize_shortcut(input: &str) -> String {
    // 转为插件要求的小写形式，并去掉多余空格
    input
        .split('+')
        .map(|p| p.trim().to_lowercase())
        .collect::<Vec<_>>()
        .join("+")
}

#[tauri::command]
fn show_window_fullscreen(window: WebviewWindow) -> Result<(), String> {
    window.set_fullscreen(true).map_err(|e| e.to_string())?;
    window.set_decorations(false).map_err(|e| e.to_string())?;
    window.show().map_err(|e| e.to_string())?;
    window.set_focus().map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_window(window: WebviewWindow) -> Result<(), String> {
    window.set_fullscreen(false).map_err(|e| e.to_string())?;
    window.set_decorations(true).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn capture_screen_hidden(window: WebviewWindow) -> Result<String, String> {
    // 隐藏窗口
    window.hide().map_err(|e| e.to_string())?;
    // 等待窗口完全隐藏
    thread::sleep(Duration::from_millis(200));
    // 截图
    let result = capture_screen();
    result
}

#[tauri::command]
fn capture_screen() -> Result<String, String> {
    let mut last_err = String::new();
    let mut order = Vec::new();

    if let Some(preferred) = get_preferred_backend() {
        order.push(preferred);
    }

    for backend in [CaptureBackend::Grim, CaptureBackend::Xcap, CaptureBackend::GnomeScreenshot] {
        if !order.contains(&backend) {
            order.push(backend);
        }
    }

    for backend in order {
        let result = match backend {
            CaptureBackend::Grim => capture_with_timeout("grim", Duration::from_secs(2), capture_with_grim),
            CaptureBackend::Xcap => capture_with_timeout("xcap", Duration::from_secs(2), capture_with_xcap),
            CaptureBackend::GnomeScreenshot => capture_with_gnome_screenshot(),
        };

        match result {
            Ok(data) => {
                set_preferred_backend(backend);
                return Ok(data);
            }
            Err(err) => last_err = err,
        }
    }

    Err(last_err)
}

fn capture_with_timeout<F>(name: &str, timeout: Duration, capture: F) -> Result<String, String>
where
    F: FnOnce() -> Result<String, String> + Send + 'static,
{
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(capture());
    });

    match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(_) => Err(format!("{name} 截图超时（超过 {:?}）", timeout)),
    }
}

fn capture_with_xcap() -> Result<String, String> {
    let monitors = Monitor::all().map_err(|e| e.to_string())?;
    let monitor = monitors.into_iter().next().ok_or("No monitor found")?;
    let image = monitor.capture_image().map_err(|e| e.to_string())?;

    let mut buf = Cursor::new(Vec::new());
    image
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(STANDARD.encode(buf.into_inner()))
}

fn capture_with_grim() -> Result<String, String> {
    let output = Command::new("grim")
        .arg("-")
        .output()
        .map_err(|e| format!("grim: {}", e))?;

    if !output.status.success() {
        return Err(format!("grim: {}", String::from_utf8_lossy(&output.stderr)));
    }

    Ok(STANDARD.encode(&output.stdout))
}

fn capture_with_gnome_screenshot() -> Result<String, String> {
    let tmp_file = "/tmp/prinsp_screenshot.png";
    let _ = std::fs::remove_file(tmp_file);

    let mut child = Command::new("gnome-screenshot")
        .arg("-f")
        .arg(tmp_file)
        .spawn()
        .map_err(|e| format!("gnome-screenshot: {}", e))?;

    // 等待最多2秒
    for _ in 0..20 {
        match child.try_wait() {
            Ok(Some(status)) => {
                if !status.success() {
                    return Err("gnome-screenshot failed".to_string());
                }
                break;
            }
            Ok(None) => thread::sleep(Duration::from_millis(100)),
            Err(e) => return Err(format!("gnome-screenshot: {}", e)),
        }
    }

    let data = std::fs::read(tmp_file).map_err(|e| format!("read file: {}", e))?;
    let _ = std::fs::remove_file(tmp_file);

    Ok(STANDARD.encode(&data))
}

/// 颜色通道增强：对彩色文字（如红色）提升与背景的对比度
fn channel_emphasized_gray(img: &RgbImage) -> GrayImage {
    let (w, h) = img.dimensions();
    let n = (w as u64) * (h as u64);

    // 计算各通道均值
    let mut sum = [0u64; 3];
    for p in img.pixels() {
        let channels = p.channels();
        sum[0] += channels[0] as u64;
        sum[1] += channels[1] as u64;
        sum[2] += channels[2] as u64;
    }
    let mean = [
        (sum[0] / n) as f32,
        (sum[1] / n) as f32,
        (sum[2] / n) as f32,
    ];

    // 计算各通道对比度
    let mut contrast = [0f32; 3];
    for p in img.pixels() {
        let channels = p.channels();
        contrast[0] += (channels[0] as f32 - mean[0]).abs();
        contrast[1] += (channels[1] as f32 - mean[1]).abs();
        contrast[2] += (channels[2] as f32 - mean[2]).abs();
    }

    // 选择对比度最高的通道
    let best = contrast
        .iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .map(|(i, _)| i)
        .unwrap_or(0);

    // 计算增强后的灰度值并找出范围
    let mut values: Vec<f32> = Vec::with_capacity((w * h) as usize);
    for p in img.pixels() {
        let channels = p.channels();
        let r = channels[0] as f32;
        let g = channels[1] as f32;
        let b = channels[2] as f32;
        // 对红色通道最高的情况，使用 R - 0.5G - 0.5B 增强红色文字
        let v = if best == 0 {
            r - 0.5 * g - 0.5 * b
        } else if best == 1 {
            g - 0.5 * r - 0.5 * b
        } else {
            b - 0.5 * r - 0.5 * g
        };
        values.push(v);
    }

    // 线性拉伸到 0-255
    let min_v = values.iter().cloned().fold(f32::INFINITY, f32::min);
    let max_v = values.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let span = (max_v - min_v).max(1.0);

    let mut out = GrayImage::new(w, h);
    for (i, v) in values.iter().enumerate() {
        let norm = ((v - min_v) / span * 255.0).clamp(0.0, 255.0) as u8;
        let x = (i as u32) % w;
        let y = (i as u32) / w;
        out.put_pixel(x, y, image::Luma([norm]));
    }
    out
}

/// 图像预处理：颜色增强→放大→去噪→自适应二值化→闭运算
fn preprocess_for_ocr(dyn_img: &image::DynamicImage) -> GrayImage {
    let rgb = dyn_img.to_rgb8();
    let (w, h) = rgb.dimensions();

    // 颜色增强的灰度转换
    let enhanced_gray = channel_emphasized_gray(&rgb);

    // 2倍放大，提升小字识别率
    let resized = image::imageops::resize(&enhanced_gray, w * 2, h * 2, image::imageops::FilterType::Lanczos3);

    // 中值滤波去噪（保边缘）
    let denoised = median_filter(&resized, 1, 1);

    // Otsu 自适应阈值二值化
    let thr = otsu_level(&denoised);
    let binary = threshold(&denoised, thr, imageproc::contrast::ThresholdType::Binary);

    // 闭运算填补细笔画断裂
    close(&binary, Norm::L1, 1)
}

/// 后处理：规范空白，保留段落结构
fn postprocess_ocr_text(text: &str) -> String {
    let mut result = Vec::new();
    let mut prev_empty = false;

    for line in text.lines() {
        // 仅压缩连续空格，保留行内容
        let trimmed = line.trim();
        if trimmed.is_empty() {
            // 保留单个空行作为段落分隔
            if !prev_empty && !result.is_empty() {
                result.push(String::new());
            }
            prev_empty = true;
        } else {
            // 压缩连续空格但保留单个空格
            let normalized: String = trimmed
                .chars()
                .fold((String::new(), false), |(mut s, was_space), c| {
                    if c.is_whitespace() {
                        if !was_space {
                            s.push(' ');
                        }
                        (s, true)
                    } else {
                        s.push(c);
                        (s, false)
                    }
                })
                .0;
            result.push(normalized);
            prev_empty = false;
        }
    }

    // 移除末尾空行
    while result.last().map_or(false, |s| s.is_empty()) {
        result.pop();
    }

    result.join("\n")
}

#[tauri::command]
fn ocr_image(base64_data: String) -> Result<String, String> {
    ensure_tesseract_installed()?;

    let data = STANDARD.decode(&base64_data).map_err(|e| e.to_string())?;
    let dyn_img = image::load_from_memory(&data).map_err(|e| e.to_string())?;

    let processed = preprocess_for_ocr(&dyn_img);
    let processed_dyn = image::DynamicImage::ImageLuma8(processed);
    let img = TessImage::from_dynamic_image(&processed_dyn).map_err(|e| e.to_string())?;

    let mut args = Args::default();
    args.lang = "chi_sim+eng".into(); // 中文优先
    args.dpi = Some(350); // 中文对分辨率更敏感
    args.psm = Some(7); // 单行文本（适合标题类）
    args.oem = Some(1); // 仅 LSTM 引擎
    args.config_variables = {
        let mut vars = HashMap::new();
        vars.insert("preserve_interword_spaces".into(), "1".into());
        vars.insert("textord_heavy_nr".into(), "1".into());
        vars.insert("textord_min_linesize".into(), "2.5".into());
        vars.insert("textord_space_size_is_variable".into(), "1".into());
        // 关闭词典，提升生僻字/特殊符号识别
        vars.insert("load_system_dawg".into(), "F".into());
        vars.insert("load_freq_dawg".into(), "F".into());
        vars
    };

    let raw_text = rusty_tesseract::image_to_string(&img, &args).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("Failed loading language") || msg.contains("traineddata") {
            "Tesseract 语言数据缺失，请安装 tesseract-ocr-chi-sim 并确认 TESSDATA_PREFIX 配置".to_string()
        } else {
            msg
        }
    })?;

    Ok(postprocess_ocr_text(&raw_text))
}

#[tauri::command]
fn copy_text_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_text(text).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn copy_to_clipboard(base64_data: String) -> Result<(), String> {
    let data = STANDARD.decode(&base64_data).map_err(|e| e.to_string())?;
    let img = image::load_from_memory(&data).map_err(|e| e.to_string())?;
    let rgba = img.to_rgba8();

    let img_data = arboard::ImageData {
        width: rgba.width() as usize,
        height: rgba.height() as usize,
        bytes: rgba.into_raw().into(),
    };

    let mut clipboard = Clipboard::new().map_err(|e| e.to_string())?;
    clipboard.set_image(img_data).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            capture_screen,
            capture_screen_hidden,
            register_global_shortcut,
            copy_to_clipboard,
            copy_text_to_clipboard,
            hide_window,
            show_window_fullscreen,
            restore_window,
            ocr_image
        ])
        .setup(|app| {
            preselect_backend();

            // 注册全局快捷键插件
            #[cfg(desktop)]
            {
                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_shortcuts(["ctrl+shift+a"])?
                        .with_handler(|app, shortcut, event| {
                            if event.state == ShortcutState::Pressed {
                                if shortcut.matches(Modifiers::CONTROL | Modifiers::SHIFT, Code::KeyA) {
                                    if let Some(window) = app.get_webview_window("main") {
                                        let _ = window.emit("start-capture", ());
                                    }
                                }
                            }
                        })
                        .build(),
                )?;
            }

            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let capture =
                MenuItem::with_id(app, "capture", "截图 (Ctrl+Shift+A)", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&capture, &quit])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("PrinSp 截图工具")
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        if let Some(window) = tray.app_handle().get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("start-capture", ());
                        }
                    }
                    _ => {}
                })
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "capture" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                            let _ = window.emit("start-capture", ());
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            if let Some(window) = app.get_webview_window("main") {
                let _ = window.emit("start-capture", ());
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
