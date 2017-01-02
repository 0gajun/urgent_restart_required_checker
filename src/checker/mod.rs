mod depends;

pub fn is_urgent_restart_required() -> bool {
    match depends::get_urgent_updates() {
        Some(_) => true,
        None => false
    }
}
