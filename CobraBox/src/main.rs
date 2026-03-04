use pelite::pe64::{Pe, PeFile, PeObject};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = Path::new("./cheat.exe");
    let data = fs::read(path)?;
    let file = PeFile::from_bytes(&data)?;
    let imports = file.imports()?;
    for imp in imports {
        print!("{}\n", imp.dll_name().unwrap())
    }
   

    
    Ok(())
}
