use bluenote::{IniProfileStore, DEFAULT_INI_FILE_PATH};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("DEFAULT_INI_FILE_PATH: {}", DEFAULT_INI_FILE_PATH);
    
    let ini_store = IniProfileStore::new(DEFAULT_INI_FILE_PATH);
    println!("Loading profile 'default' from: {}", DEFAULT_INI_FILE_PATH);
    
    match ini_store.get_profile("default")? {
        Some(profile) => {
            println!("Profile loaded successfully:");
            println!("  Server: {:?}", profile.server());
            println!("  User: {:?}", profile.user());
            println!("  Password: {:?}", profile.password());
            println!("  Insecure: {:?}", profile.insecure());
        }
        None => {
            println!("Profile 'default' not found");
        }
    }
    
    Ok(())
}
