use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, PermissionsExt};

//note: what if user does echo "hello world'
pub fn echo(args: &[&str]) {
    if args.is_empty() {
        println!();
        return;
    }
    
    let mut output = Vec::new();
    for arg in args {
        let mut processed = *arg;
        if (processed.starts_with('"') && processed.ends_with('"')) ||
           (processed.starts_with('\'') && processed.ends_with('\'')) {
            processed = &processed[1..processed.len()-1];
        }
        output.push(processed);
    }
    
    println!("{}", output.join(" "));
}

pub fn cd(args: &[&str], current_dir: &PathBuf) -> Option<PathBuf> {
    let target = if args.is_empty() {
        env::var("HOME").ok().map(PathBuf::from).unwrap_or_else(|| {
            #[cfg(unix)]
            { PathBuf::from("/") }
            #[cfg(not(unix))]
            { env::current_dir().unwrap_or_else(|_| PathBuf::from(".")) }
        })
    } else {
        let path = args[0];
        #[cfg(unix)]
        {
            if path.starts_with('/') {
                PathBuf::from(path)
            } else {
                current_dir.join(path)
            }
        }
        #[cfg(not(unix))]
        {
            if path.starts_with('/') || path.starts_with('\\') {
                PathBuf::from(path)
            } else {
                current_dir.join(path)
            }
        }
    };
    
    match env::set_current_dir(&target) {
        Ok(_) => Some(target),
        Err(e) => {
            eprintln!("cd: {}: {}", target.display(), e);
            None
        }
    }
}

/// Print working directory
pub fn pwd(current_dir: &PathBuf) {
    println!("{}", current_dir.display());
}

/// List directory contents
pub fn ls(args: &[&str], current_dir: &PathBuf) {
    let mut show_long = false;
    let mut show_all = false;
    let mut show_type = false;
    let mut target_dir = current_dir.clone();
    
    // Parse flags
    let mut paths_start = 0;
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with('-') {
            paths_start = i + 1;
            for c in arg.chars().skip(1) {
                match c {
                    'l' => show_long = true,
                    'a' => show_all = true,
                    'F' => show_type = true,
                    _ => {}
                }
            }
        } else {
            paths_start = i;
            break;
        }
    }
    
    // Get target directory
    if paths_start < args.len() {
        let path = args[paths_start];
        #[cfg(unix)]
        {
            if path.starts_with('/') {
                target_dir = PathBuf::from(path);
            } else {
                target_dir = current_dir.join(path);
            }
        }
        #[cfg(not(unix))]
        {
            // On Windows, handle both / and \ as path separators
            if path.starts_with('/') || path.starts_with('\\') {
                target_dir = PathBuf::from(path);
            } else {
                target_dir = current_dir.join(path);
            }
        }
    }
    
    // List directory
    match fs::read_dir(&target_dir) {
        Ok(entries) => {
            let mut items: Vec<_> = entries
                .filter_map(|e| e.ok())
                .collect();
            
            // Filter hidden files if -a not specified
            if !show_all {
                items.retain(|e| {
                    e.file_name().to_string_lossy().starts_with('.') == false
                });
            }
            
            // Sort alphabetically
            items.sort_by_key(|e| e.file_name());
            
            if show_long {
                // Calculate total blocks (simplified)
                #[cfg(unix)]
                let total: u64 = items.iter()
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| file_size_blocks(&m))
                    .sum();
                #[cfg(not(unix))]
                let total: u64 = items.iter()
                    .filter_map(|e| e.metadata().ok())
                    .map(|m| file_size_blocks(&m))
                    .sum();
                println!("total {}", total);
                
                for entry in items {
                    if let Ok(metadata) = entry.metadata() {
                        print_entry_long(&entry, &metadata, show_type);
                    }
                }
            } else {
                let items_len = items.len();
                for entry in items {
                    let name = entry.file_name();
                    let name_str = name.to_string_lossy();
                    if show_type {
                        if let Ok(metadata) = entry.metadata() {
                            let suffix = if metadata.is_dir() {
                                "/"
                            } else if is_executable(&metadata) {
                                "*"
                            } else {
                                ""
                            };
                            print!("{}{}  ", name_str, suffix);
                        } else {
                            print!("{}  ", name_str);
                        }
                    } else {
                        print!("{}  ", name_str);
                    }
                }
                if items_len > 0 {
                    println!();
                }
            }
        }
        Err(e) => {
            eprintln!("ls: {}: {}", target_dir.display(), e);
        }
    }
}

//note: review whatever the fuck this is
fn print_entry_long(entry: &fs::DirEntry, metadata: &fs::Metadata, show_type: bool) {
    // Permissions
    #[cfg(unix)]
    let mode = metadata.permissions().mode();
    #[cfg(not(unix))]
    let mode = 0o644; // Default mode for Windows
    let perms = format_permissions(mode, metadata.is_dir());
    print!("{} ", perms);
    
    // Number of links
    #[cfg(unix)]
    let nlink = metadata.nlink();
    #[cfg(not(unix))]
    let nlink = 1;
    print!("{:>3} ", nlink);
    
    // Owner and group (simplified - would need to look up UID/GID)
    #[cfg(unix)]
    {
        print!("{:>5} {:>5} ", metadata.uid(), metadata.gid());
    }
    #[cfg(not(unix))]
    {
        print!("{:>5} {:>5} ", 0, 0);
    }
    
    // Size
    print!("{:>8} ", metadata.len());
    
    // Modified time
    if let Ok(modified) = metadata.modified() {
        if let Ok(duration) = modified.duration_since(SystemTime::UNIX_EPOCH) {
            let secs = duration.as_secs();
            let time = secs % 86400;
            let hour = time / 3600;
            let min = (time % 3600) / 60;
            let sec = time % 60;
            let day = secs / 86400;
            let month_days = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut year = 1970;
            let mut days = day;
            while days >= if is_leap_year(year) { 366 } else { 365 } {
                days -= if is_leap_year(year) { 366 } else { 365 };
                year += 1;
            }
            let mut month = 0;
            for (i, &md) in month_days.iter().enumerate() {
                let days_in_month = if i == 1 && is_leap_year(year) { md + 1 } else { md };
                if days < days_in_month {
                    month = i;
                    break;
                }
                days -= days_in_month;
            }
            let month_names = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                              "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
            print!("{} {:2} {:02}:{:02}:{:02} ", month_names[month], days + 1, hour, min, sec);
        }
    }
    
    // Name
    let name = entry.file_name();
    let name_str = name.to_string_lossy();
    if show_type {
        let suffix = if metadata.is_dir() {
            "/"
        } else if is_executable(metadata) {
            "*"
        } else {
            ""
        };
        println!("{}{}", name_str, suffix);
    } else {
        println!("{}", name_str);
    }
}
//note: review whatever the fuck this is
#[cfg(unix)]
fn file_size_blocks(metadata: &fs::Metadata) -> u64 {
    (metadata.size() + 511) / 512
}
//note: review whatever the fuck this is
#[cfg(not(unix))]
fn file_size_blocks(metadata: &fs::Metadata) -> u64 {
    (metadata.len() + 511) / 512
}
//note: review whatever the fuck this is
#[cfg(unix)]
fn is_executable(metadata: &fs::Metadata) -> bool {
    metadata.permissions().mode() & 0o111 != 0
}
//note: review whatever the fuck this is
#[cfg(not(unix))]
fn is_executable(_metadata: &fs::Metadata) -> bool {
    false
}

fn format_permissions(mode: u32, is_dir: bool) -> String {
    let file_type = if is_dir {
        'd'
    } else {
        '-' // regular file (we don't detect symlinks here)
    };
    
    let mut perms = String::from(file_type);
    // Extract rwx bits for owner, group, and others
    for shift in (0..3).rev() {
        let bits = (mode >> (shift * 3)) & 0o7;
        perms.push(if bits & 0o4 != 0 { 'r' } else { '-' });
        perms.push(if bits & 0o2 != 0 { 'w' } else { '-' });
        perms.push(if bits & 0o1 != 0 { 'x' } else { '-' });
    }
    perms
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Cat command - display file contents
pub fn cat(args: &[&str], current_dir: &PathBuf) {
    if args.is_empty() {
        // Read from stdin
        let mut buffer = String::new();
        loop {
            match io::stdin().read_line(&mut buffer) {
                Ok(0) => break,
                Ok(_) => continue,
                Err(_) => break,
            }
        }
        print!("{}", buffer);
        return;
    }
    
    for arg in args {
        let file_path = {
            #[cfg(unix)]
            {
                if arg.starts_with('/') {
                    PathBuf::from(arg)
                } else {
                    current_dir.join(arg)
                }
            }
            #[cfg(not(unix))]
            {
                if arg.starts_with('/') || arg.starts_with('\\') {
                    PathBuf::from(arg)
                } else {
                    current_dir.join(arg)
                }
            }
        };
        
        match fs::File::open(&file_path) {
            Ok(mut file) => {
                let mut contents = String::new();
                if let Ok(_) = file.read_to_string(&mut contents) {
                    print!("{}", contents);
                } else {
                    eprintln!("cat: {}: Error reading file", file_path.display());
                }
            }
            Err(e) => {
                eprintln!("cat: {}: {}", file_path.display(), e);
            }
        }
    }
}

/// Copy command
pub fn cp(args: &[&str], current_dir: &PathBuf) {
    if args.len() < 2 {
        eprintln!("cp: missing file operand");
        return;
    }
    
    let sources: Vec<_> = args[..args.len()-1].iter().collect();
    let dest = args[args.len()-1];
    let sources_len = sources.len();
    
    let dest_path = {
        #[cfg(unix)]
        {
            if dest.starts_with('/') {
                PathBuf::from(dest)
            } else {
                current_dir.join(dest)
            }
        }
        #[cfg(not(unix))]
        {
            if dest.starts_with('/') || dest.starts_with('\\') {
                PathBuf::from(dest)
            } else {
                current_dir.join(dest)
            }
        }
    };
    
    // Check if destination is a directory
    let dest_is_dir = dest_path.is_dir();
    
    for source in sources {
        let source_path = {
            #[cfg(unix)]
            {
                if source.starts_with('/') {
                    PathBuf::from(source)
                } else {
                    current_dir.join(source)
                }
            }
            #[cfg(not(unix))]
            {
                if source.starts_with('/') || source.starts_with('\\') {
                    PathBuf::from(source)
                } else {
                    current_dir.join(source)
                }
            }
        };
        
        let final_dest = if dest_is_dir {
            dest_path.join(source_path.file_name().unwrap_or_else(|| source.as_ref()))
        } else {
            if sources_len > 1 {
                eprintln!("cp: target '{}' is not a directory", dest);
                return;
            }
            dest_path.clone()
        };
        
        match copy_recursive(&source_path, &final_dest) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("cp: {}: {}", source_path.display(), e);
            }
        }
    }
}

fn copy_recursive(source: &Path, dest: &Path) -> io::Result<()> {
    if source.is_dir() {
        fs::create_dir_all(dest)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            let dest_path = dest.join(path.file_name().unwrap());
            copy_recursive(&path, &dest_path)?;
        }
    } else {
        fs::copy(source, dest)?;
    }
    Ok(())
}

/// Remove command
pub fn rm(args: &[&str], current_dir: &PathBuf) {
    if args.is_empty() {
        eprintln!("rm: missing operand");
        return;
    }
    
    let mut recursive = false;
    let mut paths_start = 0;
    
    // Parse flags
    for (i, arg) in args.iter().enumerate() {
        if arg.starts_with('-') {
            paths_start = i + 1;
            if arg.contains('r') || arg.contains('R') {
                recursive = true;
            }
        } else {
            paths_start = i;
            break;
        }
    }
    
    for arg in &args[paths_start..] {
        let path = if arg.starts_with('/') {
            PathBuf::from(arg)
        } else {
            current_dir.join(arg)
        };
        
        if path.is_dir() {
            if recursive {
                match fs::remove_dir_all(&path) {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("rm: {}: {}", path.display(), e);
                    }
                }
            } else {
                eprintln!("rm: {}: is a directory", path.display());
            }
        } else {
            match fs::remove_file(&path) {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("rm: {}: {}", path.display(), e);
                }
            }
        }
    }
}

/// Move/rename command
pub fn mv(args: &[&str], current_dir: &PathBuf) {
    if args.len() < 2 {
        eprintln!("mv: missing file operand");
        return;
    }
    
    let sources: Vec<_> = args[..args.len()-1].iter().collect();
    let dest = args[args.len()-1];
    let sources_len = sources.len();
    
    let dest_path = {
        #[cfg(unix)]
        {
            if dest.starts_with('/') {
                PathBuf::from(dest)
            } else {
                current_dir.join(dest)
            }
        }
        #[cfg(not(unix))]
        {
            if dest.starts_with('/') || dest.starts_with('\\') {
                PathBuf::from(dest)
            } else {
                current_dir.join(dest)
            }
        }
    };
    
    // Check if destination is a directory
    let dest_is_dir = dest_path.is_dir();
    
    for source in sources {
        let source_path = {
            #[cfg(unix)]
            {
                if source.starts_with('/') {
                    PathBuf::from(source)
                } else {
                    current_dir.join(source)
                }
            }
            #[cfg(not(unix))]
            {
                if source.starts_with('/') || source.starts_with('\\') {
                    PathBuf::from(source)
                } else {
                    current_dir.join(source)
                }
            }
        };
        
        let final_dest = if dest_is_dir {
            dest_path.join(source_path.file_name().unwrap_or_else(|| source.as_ref()))
        } else {
            if sources_len > 1 {
                eprintln!("mv: target '{}' is not a directory", dest);
                return;
            }
            dest_path.clone()
        };
        
        match fs::rename(&source_path, &final_dest) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("mv: {}: {}", source_path.display(), e);
            }
        }
    }
}

/// Make directory command
pub fn mkdir(args: &[&str], current_dir: &PathBuf) {
    if args.is_empty() {
        eprintln!("mkdir: missing operand");
        return;
    }
    
    for arg in args {
        let path = {
            #[cfg(unix)]
            {
                if arg.starts_with('/') {
                    PathBuf::from(arg)
                } else {
                    current_dir.join(arg)
                }
            }
            #[cfg(not(unix))]
            {
                if arg.starts_with('/') || arg.starts_with('\\') {
                    PathBuf::from(arg)
                } else {
                    current_dir.join(arg)
                }
            }
        };
        
        match fs::create_dir_all(&path) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("mkdir: {}: {}", path.display(), e);
            }
        }
    }
}

