mod checker;

pub fn check() {
    println!("{}", checker::is_urgent_restart_required());
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
