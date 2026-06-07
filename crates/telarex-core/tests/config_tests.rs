use telarex_core::config::TelaRexConfig;

#[test]
fn test_config_save_and_load_roundtrip() {
    let dir = std::env::temp_dir().join("telarex_test_config");
    let _ = std::fs::create_dir_all(&dir);
    let config_path = dir.join("test_config.toml");

    // Create and save a config with modified values
    let mut config = TelaRexConfig::default();
    config.editor.tab_size = 2;
    config.editor.vim_mode = true;
    config.editor.auto_save = true;
    config.recent_projects.push("/test/path".to_string());

    let toml_str = toml::to_string_pretty(&config).unwrap();
    std::fs::write(&config_path, &toml_str).unwrap();

    // Load and verify
    let loaded_str = std::fs::read_to_string(&config_path).unwrap();
    let loaded: TelaRexConfig = toml::from_str(&loaded_str).unwrap();
    assert_eq!(loaded.editor.tab_size, 2);
    assert!(loaded.editor.vim_mode);
    assert!(loaded.editor.auto_save);
    assert_eq!(loaded.recent_projects.len(), 1);
    assert_eq!(loaded.recent_projects[0], "/test/path");
    assert_eq!(loaded.keymaps.global.len(), 7);

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn test_config_default_creates_valid_toml() {
    let config = TelaRexConfig::default();
    let toml_str = toml::to_string_pretty(&config).unwrap();
    assert!(toml_str.contains("tab_size"));
    assert!(toml_str.contains("version"));
    assert!(toml_str.contains("ctrl-q"));
}

#[test]
fn test_config_add_recent_project_persists_order() {
    let mut config = TelaRexConfig::default();
    config.add_recent_project("/first".to_string());
    config.add_recent_project("/second".to_string());
    config.add_recent_project("/third".to_string());
    assert_eq!(config.recent_projects, vec!["/third", "/second", "/first"]);
}

#[test]
fn test_config_add_recent_project_evicts_oldest() {
    let mut config = TelaRexConfig::default();
    for i in 0..10 {
        config.add_recent_project(format!("/path_{}", i));
    }
    // Adding one more should evict the oldest (/path_0 moves to position 9, /path_10 is first)
    config.add_recent_project("/path_new".to_string());
    assert_eq!(config.recent_projects.len(), 10);
    assert_eq!(config.recent_projects[0], "/path_new");
    // /path_0 should have been evicted (truncate at 10 removes oldest from end)
    assert_eq!(config.recent_projects[9], "/path_1");
}
