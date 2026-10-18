// Experiment only: create a tinshift operation whose JSON grid PROJ downloads.
use std::ffi::{CStr, CString};

#[test]
fn fetch_json_grid() {
    std::thread::spawn(|| unsafe {
        let ctx = proj_sys::proj_context_create();
        proj_sys::proj_log_level(ctx, proj_sys::PJ_LOG_LEVEL_PJ_LOG_DEBUG);
        assert_eq!(proj_sys::proj_context_set_enable_network(ctx, 1), 1);
        let def = CString::new("+proj=tinshift +file=fi_nls_ykj_etrs35fin.json").unwrap();
        let pj = proj_sys::proj_create(ctx, def.as_ptr());
        let errno = proj_sys::proj_context_errno(ctx);
        if errno != 0 {
            let msg = CStr::from_ptr(proj_sys::proj_context_errno_string(ctx, errno));
            eprintln!("errno {errno}: {msg:?}");
        }
        assert!(!pj.is_null());
    })
    .join()
    .unwrap();
}

#[test]
fn local_json_grid() {
    std::fs::create_dir_all("/grids").unwrap();
    std::fs::write(
        "/grids/fi_nls_ykj_etrs35fin.json",
        include_bytes!("data/fi_nls_ykj_etrs35fin.json"),
    )
    .unwrap();
    std::thread::spawn(|| unsafe {
        let ctx = proj_sys::proj_context_create();
        let path = CString::new("/grids").unwrap();
        let paths = [path.as_ptr()];
        proj_sys::proj_context_set_search_paths(ctx, 1, paths.as_ptr());
        let def = CString::new("+proj=tinshift +file=fi_nls_ykj_etrs35fin.json").unwrap();
        let pj = proj_sys::proj_create(ctx, def.as_ptr());
        let errno = proj_sys::proj_context_errno(ctx);
        if errno != 0 {
            let msg = CStr::from_ptr(proj_sys::proj_context_errno_string(ctx, errno));
            eprintln!("errno {errno}: {msg:?}");
        }
        assert!(!pj.is_null());
    })
    .join()
    .unwrap();
}
