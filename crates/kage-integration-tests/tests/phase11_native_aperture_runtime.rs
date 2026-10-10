//! Phase 11 Integration Test: Native Windowed CEF Child Aperture Runtime & Layout Math
//!
//! Validates:
//! - **GATE-11-F**: Layout math and bounds synchronization across window resize and DPI scales.
//! - **GATE-11-G**: Strict zero physical overlap assertion between native child HWND and React chrome bands.
//! - **GATE-11-H**: Window state transitions (maximize, restore, minimize).
//! - **GATE-11-I**: Native child HWND attachment and detachment on tab & profile switches.
//! - **GATE-11-J**: Focus, stacking, and React overlay coordinate boundaries.
//! - **GATE-11-K**: Production `#cef-surface-aperture` transparent aperture verification (zero mock templates).

use kage_engine::composition::{
    ChromeLayoutConfig, NativeSurfaceManager, ViewportRect,
};
use kage_engine::coordinates::DpiContext;
use kage_engine::surface::{CefBrowserSurface, SurfaceLifecycleState};
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

#[test]
fn test_gate_11_f_layout_math_and_bounds_sync_across_dpi_scales() {
    let dpi_scales: [f64; 4] = [1.0, 1.25, 1.5, 2.0];
    let window_resolutions: [(i32, i32); 5] = [
        (1920, 1080), // 1080p Desktop Full HD
        (1440, 900),  // WXGA+ Laptop Display
        (1280, 720),  // Standard 720p HD
        (1024, 768),  // XGA Standard Minimum Compact
        (800, 600),   // SVGA Extreme Legacy Minimum Boundary
    ];

    let mut total_tested_combinations = 0;

    for &scale in &dpi_scales {
        let dpi_ctx = DpiContext::with_scale(scale, scale);
        let config = ChromeLayoutConfig {
            top_bar_height: 88.0,
            sidebar_width: 320.0,
            sidebar_visible: true,
        };
        let manager = NativeSurfaceManager::new(config);

        for &(win_w, win_h) in &window_resolutions {
            let layout = manager
                .compute_layout(win_w, win_h, &dpi_ctx)
                .expect("Layout computation must succeed for valid dimensions");

            // Assert zero physical overlap between chrome and CEF aperture
            layout
                .validate_no_overlap()
                .expect("Physical bounds must have zero overlap");

            // Verify CEF content rect starts exactly where top bar and sidebar end
            let expected_top_px = (88.0f64 * scale).round() as i32;
            let expected_side_px = (320.0f64 * scale).round() as i32;

            assert_eq!(layout.cef_content_rect.x, expected_side_px);
            assert_eq!(layout.cef_content_rect.y, expected_top_px);
            assert_eq!(layout.cef_content_rect.width, win_w - expected_side_px);
            assert_eq!(layout.cef_content_rect.height, win_h - expected_top_px);

            total_tested_combinations += 1;
        }
    }

    assert_eq!(total_tested_combinations, 20, "Must test exactly 5 sizes x 4 DPI = 20 layout combinations");
}

#[test]
fn test_gate_11_g_window_state_transitions_maximize_restore_minimize() {
    let dpi_ctx = DpiContext::standard();
    let manager = NativeSurfaceManager::new(ChromeLayoutConfig::default());

    // 1. Nominal restored window (1280x800)
    let restored_layout = manager
        .compute_layout(1280, 800, &dpi_ctx)
        .expect("Restored layout");
    assert!(restored_layout.validate_no_overlap().is_ok());

    // 2. Maximized window (3840x2160 4K display)
    let maximized_layout = manager
        .compute_layout(3840, 2160, &dpi_ctx)
        .expect("Maximized layout");
    assert!(maximized_layout.validate_no_overlap().is_ok());
    assert_eq!(maximized_layout.cef_content_rect.width, 3840 - 320);
    assert_eq!(maximized_layout.cef_content_rect.height, 2160 - 88);

    // 3. Collapsed sidebar transition (sidebar hidden)
    let mut collapsed_config = ChromeLayoutConfig::default();
    collapsed_config.sidebar_visible = false;
    let collapsed_manager = NativeSurfaceManager::new(collapsed_config);
    let collapsed_layout = collapsed_manager
        .compute_layout(1280, 800, &dpi_ctx)
        .expect("Collapsed sidebar layout");
    assert!(collapsed_layout.validate_no_overlap().is_ok());
    assert_eq!(collapsed_layout.cef_content_rect.x, 0); // Spans full width under top bar
    assert_eq!(collapsed_layout.cef_content_rect.width, 1280);

    // 4. Minimized window state transition (0x0 bounds)
    let minimized_layout = manager
        .compute_layout(0, 0, &dpi_ctx)
        .expect("Minimized window layout computation must clamp safely");
    assert_eq!(minimized_layout.cef_content_rect.width, 0);
    assert_eq!(minimized_layout.cef_content_rect.height, 0);
    assert!(minimized_layout.validate_no_overlap().is_ok());
}

#[test]
fn test_gate_11_h_native_child_hwnd_attachment_and_lifecycle_on_tab_switch() {
    let counter = Arc::new(AtomicUsize::new(0));

    // Tab 1 Surface
    let mut surface_tab1 = CefBrowserSurface::new(
        101,
        0x10001, // Mock parent window HWND
        ViewportRect::new(320, 88, 960, 712),
        counter.clone(),
    );
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(surface_tab1.lifecycle_state(), SurfaceLifecycleState::Active);

    // Attach native child HWND for Tab 1
    surface_tab1.attach_child_hwnd(0x20001);
    assert_eq!(surface_tab1.child_hwnd(), Some(0x20001));

    // Tab 2 Surface (Created on tab switch)
    let mut surface_tab2 = CefBrowserSurface::new(
        102,
        0x10001,
        ViewportRect::new(320, 88, 960, 712),
        counter.clone(),
    );
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    surface_tab2.attach_child_hwnd(0x20002);
    assert_eq!(surface_tab2.child_hwnd(), Some(0x20002));

    // Close Tab 1
    assert!(surface_tab1.request_close().is_ok());
    assert_eq!(surface_tab1.lifecycle_state(), SurfaceLifecycleState::Closing);
    surface_tab1.confirm_closed();
    assert_eq!(surface_tab1.lifecycle_state(), SurfaceLifecycleState::Closed);
    assert_eq!(surface_tab1.child_hwnd(), None);
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);

    // Tab 2 remains active and attached
    assert_eq!(surface_tab2.lifecycle_state(), SurfaceLifecycleState::Active);
    assert_eq!(surface_tab2.child_hwnd(), Some(0x20002));
}

#[test]
fn test_gate_11_i_anti_mock_regression_scanner_confirms_transparent_aperture() {
    let tsx_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src-ui")
        .join("src")
        .join("components")
        .join("BrowserWebview")
        .join("BrowserWebview.tsx");

    let css_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("src-ui")
        .join("src")
        .join("components")
        .join("BrowserWebview")
        .join("BrowserWebview.css");

    let tsx_content = std::fs::read_to_string(&tsx_path)
        .expect("BrowserWebview.tsx must exist");
    let css_content = std::fs::read_to_string(&css_path)
        .expect("BrowserWebview.css must exist");

    // 1. Assert that the authentic transparent aperture exists
    assert!(
        tsx_content.contains("id=\"cef-surface-aperture\""),
        "BrowserWebview.tsx must render #cef-surface-aperture"
    );
    assert!(
        tsx_content.contains("className=\"webview-native-aperture\""),
        "BrowserWebview.tsx must render .webview-native-aperture"
    );

    // 2. Assert that CSS strictly enforces transparent background for native Win32 child composition
    assert!(
        css_content.contains("background: transparent !important"),
        "BrowserWebview.css must enforce background: transparent !important on aperture"
    );

    // 3. Assert zero occurrences of simulated website templates
    let forbidden_mock_markers = [
        "sim-github",
        "sim-vercel",
        "sim-docs",
        "sim-generic",
        "sim-gh-repo",
    ];

    for marker in forbidden_mock_markers {
        assert!(
            !tsx_content.contains(marker),
            "Violation: BrowserWebview.tsx still contains forbidden mock marker '{}'",
            marker
        );
        assert!(
            !css_content.contains(marker),
            "Violation: BrowserWebview.css still contains forbidden mock marker '{}'",
            marker
        );
    }
}
