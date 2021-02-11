pub fn cd(cd_to: &str) -> Result<(), std::io::Error> {
    if let Err(err) =  std::env::set_current_dir(cd_to) {
        return Err(err);
    }
    Ok(())
}
