use std::{
    io,
    convert::AsRef,
    collections::HashMap,
    path::Path,
    os::unix::fs::PermissionsExt,
};

pub fn recursive_dir_read<P: AsRef<Path>>(path: P) -> Result<Vec<String>, String>{
    use std::{
        fs::{
            read_dir,
        },
        thread,
        sync::{
            Arc,
            Mutex
        }
    };

    let mut threads_vec: Vec<thread::JoinHandle<_>> = vec![];

    let path = path.as_ref();

    let paths_arc: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    let entries = match read_dir(path){
        Ok(entries) => entries,
        Err(err) => return Err(err.to_string())
    };

    for entry in entries {
        let entry = entry.unwrap();

        let path = entry.path();

        if path.is_dir() {
            let arc_clone = paths_arc.clone();

            let handle = thread::spawn(move || {
                match recursive_dir_read(&path){
                    Ok(mut paths) => {
                        let mut vec = match arc_clone.lock(){
                            Ok(lock) => lock,
                            Err(_) => return
                        };
                        
                        vec.append(&mut paths);
                    },
                    Err(_) => {}
                };
            });
            
            threads_vec.push(handle);

            continue;
        }
        let metadata = entry.metadata();

        if let Err(err) = metadata {
            return Err(err.to_string());
        }

        let metadata = metadata.unwrap();
        let permissions = metadata.permissions().mode();

        if metadata.is_file() && permissions & 0o111 != 0 {
            let path_str = path.to_str().unwrap();
            
            let mut vec = match paths_arc.lock(){
                Ok(lock) => lock,
                Err(err) => return Err(err.to_string())
            };
            
            vec.push(path_str.to_owned());
        }
    }
    
    for thread_handle in threads_vec {
        if thread_handle.join().is_err(){
            return Err("Error while trying to join thread".to_owned());
        };
    }
    
    let paths_vec = Arc::try_unwrap(paths_arc).unwrap().into_inner().unwrap();

    Ok(paths_vec)
}

pub fn get_commands_from_path() -> io::Result<HashMap<String, String>> {
    use std::env;

    let path_var = env::var("PATH").unwrap();

    let mut commands_map: HashMap<String, String> = HashMap::new();

    let paths_arc: Vec<&str> = path_var.split(":")
        .collect();

    for path_str in paths_arc {

        let path = Path::new(path_str);

        let executable_paths = match recursive_dir_read(path) {
            Ok(vec) => vec,

            Err(_err) => vec![]
        };

        for executable_path in executable_paths {
            let parts: Vec<&str> = executable_path.split("/").collect();

            if let Some(last) = parts.last(){
                commands_map.insert(last.to_string(), executable_path);
                continue;
            }

            commands_map.insert(executable_path.clone(), executable_path);
        }
    }

    Ok(commands_map)
}
