use std::{env, path::PathBuf};

pub fn get_default_install_dir() -> String {
    let dist_dir = {
        // 获取操作系统类型
        let os_type = env::consts::OS;
        let home_env_var = match os_type {
            "windows" => "USERPROFILE",
            _ => "HOME",
        };

        // 尝试获取相应的环境变量
        let home = env::var(home_env_var)
            .expect(&format!("{} environment variable not set", home_env_var));

        // 构建默认的 .xuanwu 路径
        let mut path = PathBuf::from(home);
        path.push(".xuanwu");
        path.to_string_lossy().into_owned()
    };

    dist_dir
}
