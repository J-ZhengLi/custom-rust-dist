use crate::UpdateConfiguration;

pub fn check_manager_upgrade() -> bool {
    let config: UpdateConfiguration = UpdateConfiguration;
    let upgradeable = config.check_upgrade().unwrap_or(false);
    if upgradeable {
        println!(
            "A new manager version has been detected. You can update it via using `--self-update`"
        )
    }
    upgradeable
}
